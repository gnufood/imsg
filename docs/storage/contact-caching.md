# Contact Caching

Contact caching in imsg addresses a fundamental asymmetry in the Phone Book Access Profile (PBAP): the device controls the phonebook, and the client must pull contacts on demand. Unlike MAP messages, which arrive push-style via MNS notifications, PBAP offers no change notifications—the client must decide when to pull, what to pull, and how to reconcile the pulled data with what it already has. The contact cache bridges this gap, providing fast local reads, reliable matching against message threads, and intelligent refresh decisions that avoid unnecessary network traffic.

## Why Cache Contacts at All?

PBAP operates over a Bluetooth OBEX transport that is comparatively slow and battery-intensive. Every contact pull requires the client to list the phonebook (receiving handles), then pull each vCard individually. A phonebook with hundreds of contacts can take several seconds to fully synchronize. For a GUI that wants to show contact names alongside message threads, waiting for a PBAP pull on every screen render is impractical.

The cache solves this by storing contacts in a local SQLCipher database. Once synchronized, the GUI can read contact names instantly without touching the Bluetooth radio. The cache also enables the `--lookup` feature, where a user can find which contact owns a given phone number—a lookup that would otherwise require a live PBAP search operation.

However, caching introduces its own problem: the cache can become stale. The device may add, modify, or delete contacts between syncs. The system must therefore decide when to refresh, how to detect staleness, and what to do when the device's phonebook identity fundamentally changes.

## Keying: UID Over Handle

PBAP exposes two identifiers for each contact: a **handle** (like `1.vcf`, `2.vcf`) and a **UID** (the vCard's `UID` property). The handle is volatile—reordering the contacts on the device can change which handle points to which contact. The UID is durable, assigned by the device and stable across reorderings.

imsg keys contacts by UID, not handle. This is explicit in the `ContactRow` structure, which stores `uid` as the primary key:

```rust
pub struct ContactRow {
    pub uid: String,
    pub display_name: Option<String>,
    pub phones: Vec<PhoneField>,
}
```

The `upsert_contacts` operation uses `ON CONFLICT(uid) DO UPDATE`, so a contact whose handle changed but whose UID persists is updated in place rather than creating a duplicate. This design choice ensures that contact identity survives device-side reorganizations.

## Phone Number Normalization: The E.164 Design

Phone numbers appear in many formats: `+1 555 000 0001`, `0015550000001`, `(555) 000-0001`, `5550000001`, and so on. When a user searches for a contact by phone number, or when the system joins contacts against message threads, format differences must not cause mismatches. A message from `+15550001` should match a contact listed as `(555) 000-0001`.

imsg solves this through **normalization at ingress**, not at match time. Every phone number that enters the system—whether from a PBAP vCard or a MAP message—is normalized to **E.164** format (a globally unambiguous canonical form like `+15550001`) and stored alongside the original raw value.

### The PhoneField Type

The `PhoneField` type in `imsg-formats` encapsulates both forms:

```rust
pub struct PhoneField {
    raw: String,
    e164: Option<String>,
}
```

When created, the raw value is parsed via the `phonenumber` crate's regional metadata. If parsing succeeds and produces a valid E.164 number, `e164` holds the canonical form; otherwise it remains `None` and the raw value is used for all operations.

The key methods are:
- `raw()` — the device-reported form, preserved exactly
- `e164()` — the canonical form if resolved
- `canonical()` — E.164 when available, otherwise raw (used for matching)
- `display()` — the value to show by default

This design reflects a deliberate choice: normalization is best-effort, never a gate. A number that cannot be resolved to E.164 is not rejected—it is stored as-is and will match only on exact string equality. This handles edge cases like short codes, internal extensions, and numbers from regions without clear country codes.

### Why Ingress Normalization?

The alternative—normalizing at display time or match time—leads to scattered call sites and inconsistent behavior. If the CLI normalizes at render but the GUI normalizes at display, the same data shows differently depending on the client. If matching normalizes on every query, the database cannot use indexes effectively.

By normalizing once at the point where data enters the system (the PBAP vCard parser for contacts, the MAP message ingress for messages), the canonical form flows downstream as a plain string. Every read, join, and match operates on consistent data without re-parsing. The design is captured in the internal document `PHONE_NORMALIZATION.md`, which details the reasoning and the specific ingress chokepoints.

### The Storage Schema

The `contact_phones` table stores both forms:

```sql
CREATE TABLE contact_phones (
    address TEXT PRIMARY KEY,
    address_e164 TEXT,
    uid TEXT REFERENCES contacts(uid)
);
```

The `address` column holds the raw value and is the primary key, ensuring uniqueness. The `address_e164` column holds the canonical form when available. Reads reconstruct `PhoneField` via `from_parts`, which trusts the stored value rather than re-parsing (avoiding metadata drift between the parsing library version and the stored result).

Matching uses `COALESCE(address_e164, address)`, which prefers the canonical form but falls back to raw when normalization failed. This single expression handles all combinations of resolved and unresolved numbers.

## Cache Invalidation: Device Version Tracking

The most subtle aspect of contact caching is knowing when to refresh. Pulling the entire phonebook on every session is wasteful; pulling never leaves the cache stale. PBAP provides version counters that let the client detect changes without pulling.

### PbapMeta: The Version Watermark

The `PbapMeta` structure stores three values from the device:

```rust
pub struct PbapMeta {
    pub database_id: Option<String>,
    pub primary_version: Option<String>,
    pub secondary_version: Option<String>,
}
```

- **DatabaseIdentifier**: A 16-byte identifier that changes whenever the device's phonebook database is rebuilt (factory reset, major contact app update, SIM swap). When this changes, all previously cached UIDs are potentially invalid—the device may reuse old UID values for entirely different contacts.
- **PrimaryVersionCounter**: Changes when any contact in the phonebook is added, modified, or deleted.
- **SecondaryVersionCounter**: Changes when contact properties that don't affect identity (like a new photo) are updated.

The sync logic in `session::contacts::sync` compares these values:

```rust
if remote.database_id.is_some() && remote == cached {
    // Watermarks match — cache is current
    return Ok(SyncReport::UpToDate);
}
let cleared = remote.database_id.is_some() && remote.database_id != cached.database_id;
if cleared {
    store.clear_contacts().await?;  // Wipe the cache
}
```

If the `DatabaseIdentifier` changed, the cache is wiped entirely before the refresh. The `Refresh` struct reports whether a wipe occurred, allowing callers to log or surface that information.

If only the version counters changed, the cache is refreshed in place—new contacts are upserted, existing contacts are updated, and contacts no longer present on the device remain in the cache (deletion is not currently propagated; this is a deliberate scope limitation).

### Why This Matters

Without version tracking, the client would have no way to know whether the cache is current. It would either pull unnecessarily (wasting time and battery) or fail to notice important changes. The `DatabaseIdentifier` check is particularly important because it catches scenarios where the device's contact database has been fundamentally rebuilt—the cached UIDs are no longer meaningful.

The design also handles the first-sync case gracefully. When no prior metadata exists (`cached.database_id` is `None`), every call performs a full refresh. This ensures the cache is populated on first use without special-casing the initial sync.

## The Sync Flow

The complete sync flow, implemented in `session::contacts::sync::sync_contacts`, proceeds as follows:

1. **Fetch metadata**: The client requests the phonebook metadata (not the contacts themselves), receiving the `DatabaseIdentifier` and version counters.
2. **Compare**: If the metadata matches the cached values, return `SyncReport::UpToDate` and update the `contacts_synced_at` timestamp.
3. **Check identity change**: If the `DatabaseIdentifier` differs, wipe the contact cache.
4. **Pull contacts**: List all handles, skip the owner card (`0.vcf`), pull each vCard, and extract UID, display name, and phone numbers.
5. **Upsert**: Write all contacts to the store in a single transaction, replacing each contact's full phone set.
6. **Persist metadata**: Store the new `PbapMeta` values for next comparison.

The flow is designed to be idempotent: running it multiple times with no changes produces `UpToDate` after the first refresh, and running it with an unchanged device is cheap (one metadata request, no vCard pulls).

## Interaction with Message Threads

The contact cache exists primarily to support the thread list, where users expect to see contact names rather than raw phone numbers. The `threads()` query in the store performs a `LEFT JOIN` against `contact_phones`, matching on the canonical phone form:

```sql
SELECT ... FROM threads
LEFT JOIN contact_phones ON COALESCE(address_e164, address) = threads.address
LEFT JOIN contacts ON contact_phones.uid = contacts.uid
```

This join is why E.164 normalization is critical: without it, a message from `+15550001` would not match a contact listed as `(555) 000-0001`, and the thread would display the raw number instead of the contact name. The normalization design ensures this join works reliably across formatting variations.

## Design Trade-offs

Several trade-offs shaped this design:

- **UID vs. handle**: Choosing UID as the key adds complexity (parsing vCards to extract the UID property) but provides durable identity. Handle-based keys would be simpler but would break on any device-side reordering.
- **Full replacement vs. delta**: The current design replaces each contact's entire phone set on every sync. PBAP provides no per-number change signal, so a number that disappears from the device is not automatically deleted from the cache. This is a known limitation—deletion propagation would require additional tracking.
- **Best-effort normalization**: Rejecting unresolvable numbers would be stricter but would lose data. Storing both forms and falling back to raw ensures no device-reported information is discarded.
- **Cache wipe on DatabaseIdentifier change**: This is conservative. In practice, some UIDs might survive a database rebuild, but detecting which ones would require additional device-side capabilities. Wiping ensures correctness at the cost of a full resync.

## Related Documentation

- The [Message Storage](message-storage.md) page covers the parallel caching strategy for MAP messages.
- The [Sync Cursors](sync-cursors.md) page explains the analogous versioning mechanism for message folder synchronization.
- The [Outbox Lifecycle](outbox-lifecycle.md) page covers outgoing message handling, which also normalizes phone numbers at ingress.
- The internal document `internal/PHONE_NORMALIZATION.md` contains the full design reasoning for the normalization approach.
- The internal document `internal/PBAP_CONTACTS.md` covers the original contact caching design and implementation stages.