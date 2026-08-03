# Roadmap

> Last updated: 2026-07-31

---

## Tags

Domain-scoped — a small set of broad prefixes, each spanning multiple sections below, numbered
sequentially within the prefix (not per-section) as sub-items ship:

| Prefix | Domain |
|---|---|
| `PLATFORM` | Internal engineering — error handling, logging, and shared infrastructure |
| `TRANSPORT` | Connectivity and protocol support |
| `MSG` | Messaging and contacts features |
| `STORE` | Persistence and data security |
| `SVC` | Service management |
| `CLIENTS` | Alternative frontends |
| `API` | Public API and SDKs |
| `DISC` | Built-in device discovery — fully shipped, kept only so the tag in Completed can be looked up |

Cross-cutting — fixes that don't map to a roadmap line item:

| Prefix | Meaning |
|---|---|
| `OBS` | Observability — logging/tracing/diagnostics fixes |
| `GAP` | Protocol/device compliance gap — workaround for something the spec implies but the device doesn't honor |
| `ISS` | Tracked issue — scoped bug or gap, not large enough to warrant its own roadmap section |

---

## Planned

### Standardized error + logging (CLI + GUI)

The CLI and GUI each handle errors their own way, and the GUI's approach requires a manual update
for every new kind of error, which already lags behind new features. Logging setup is also
duplicated between the two entrypoints instead of shared.

- [ ] **PLATFORM-01** — Unify or share the error-conversion boundary between CLI and GUI
- [ ] **PLATFORM-02** — Extract one shared logging setup used by both entrypoints

### Hub and spoke topology

The hub relay connects to the phone per-connection today, rather than through the daemon's
persistent session handling, so the hub-and-spoke path gets none of the benefit of the
persistent-session work already shipped for direct connections.

- [ ] **TRANSPORT-01** — Extend persistent session handling to the hub side of the hub-and-spoke setup
- [ ] **TRANSPORT-02** — Extend persistent session handling to the spoke side of the hub-and-spoke setup

### Message attachments

MAP supports advertising MMS, but Apple's SDP record doesn't advertise it, and the Attachment
flag isn't respected by the device — attachment support can't be driven off protocol metadata
alone. Messages are also UTF-8 constrained.

- [ ] **MSG-01** — Investigate a non-metadata-driven detection/transfer path for MMS content

### Export

- [ ] **MSG-02** — Add an export command (format/target TBD)

### Persistence layer

Local encrypted message database so the app works without the phone connected. Fully shipped —
see STORE-01/02/03 in Completed.

### Built-in device discovery

Eliminates having to manually configure the device by its Bluetooth address. Fully shipped —
see DISC-01 in Completed.

### Full TUI client

Interactive TUI for the CLI (the desktop GUI covers the graphical case separately).

- [ ] **CLIENTS-01** — Scrollable thread list (arrow-key navigation)
- [ ] **CLIENTS-02** — Message reader pane (selected thread contents)
- [ ] **CLIENTS-03** — Compose / reply pane
- [ ] **CLIENTS-04** — Unified layout with live MNS updates

### Fuzzy + semantic search of conversations

No search of any kind exists today — the existing commands only filter by folder, sender, or date.

- [ ] **MSG-03** — Fuzzy text match over local message bodies (typo-tolerant)
- [ ] **MSG-04** — Semantic search — embedding-based similarity search over the local store,
      for meaning-based queries rather than exact/fuzzy text match

### Advanced security

Post-store-crate hardening.

- [ ] **STORE-04** — Per-record encryption — independent key per message
- [ ] **STORE-05** — Audit log — track decryption access (who read what, when)
- [ ] **STORE-06** — Key rotation — re-key DB without full rewrite

### Library + SDKs

Stable public API for building on top of imsg.

- [ ] **API-01** — Define and stabilize a public Rust API surface, with real semver discipline,
      separate from the internal workspace crates
- [ ] **API-02** — Version the broker's wire protocol so a long-running daemon and an upgraded
      CLI can detect a mismatch instead of silently talking past each other
- [ ] **API-03** — C FFI layer for cross-language binding
- [ ] **API-04** — Python SDK (PyO3 / maturin)
- [ ] **API-05** — Go SDK (cgo)

### Web UI

Browser-based interface as an alternative to the CLI/TUI.

- [ ] **CLIENTS-05** — Serves the local message DB over a local HTTP server
- [ ] **CLIENTS-06** — Thread list, message view, compose / reply in the browser
- [ ] **CLIENTS-07** — Real-time updates via SSE or WebSocket from the daemon

### Service management

Covers getting the daemon service installed, controlled, and kept in sync with what's configured.
Installing it as a system-wide (rather than per-user) service requires elevated privileges that
neither the GUI nor the underlying service-management layer currently has a way to request — the
GUI's system-install option was pulled for this reason, and system-level install/uninstall remains
CLI-only, run manually with elevated privileges. Separately, some settings changes made in the GUI are saved
but not picked up by an already-running daemon, leaving the user with no indication that a manual
restart is required.

- [ ] **SVC-01** — Ship a packaged installer for the major Linux distributions that can install
      the daemon as a system service with proper elevation, rather than the current unprivileged
      shell install
- [ ] **SVC-02** — Have the GUI install and control the system service through the operating
      system's native authorization flow instead of a custom elevation path
- [ ] **SVC-03** — Decide whether the system service should be able to start before login or only
      within a logged-in session, then restore system-level install/uninstall in the GUI
      accordingly
- [ ] **SVC-04** — Tell the user when a settings change requires a daemon restart to take
      effect, and offer a one-click way to do it
- [ ] **SVC-05** — Fix daemon restart so it actually restarts an already-running daemon
      instead of leaving it untouched
- [ ] **SVC-06** — Share the daemon restart/reconnect logic between the CLI and GUI instead of
      maintaining it twice

### Message previews

No preview rendering exists for message content beyond plain text today.

- [ ] **LINK-01** — Detect links in message text and render them as clickable
- [ ] **LINK-02** — Show a preview (title/image/description) for a detected link

### Contacts

The GUI shows a generic avatar for every contact today.

- [ ] **CONTACT-01** — Investigate syncing contact photos from the phone alongside the existing
      contact data
- [ ] **CONTACT-02** — Show a contact's synced photo in place of the generic avatar

### Phone number handling

Phone numbers written without a country code aren't normalized to a canonical form today, so
they can silently fail to match against numbers that do carry one. A design for deriving and
applying a default region has already been scoped.

- [ ] **PHONE-01** — Derive a default region for normalization from the device's own phone
      number, with a safe fallback and a user-configurable override
- [ ] **PHONE-02** — Validate a manually configured region up front so a typo is caught
      immediately instead of causing silent normalization failures later
- [ ] **PHONE-03** — Cache the derived region so it isn't re-derived on every use
- [ ] **PHONE-04** — Apply phone-number normalization consistently from one shared place
      instead of separately in each frontend

---

## Research

Items that need protocol investigation or feasibility work before they land on the Planned list.

### Additional transport protocols

Beyond RFCOMM (Classic Bluetooth) and iroh (hub/spoke QUIC).

- [ ] **XPORT-01 (BLE)** — investigate ANCS for notification delivery; MAP requires Classic Bluetooth, so full message sync over BLE is unproven
- [ ] **XPORT-02 (TCP)** — partially scaffolded internally but not yet wired into any caller; plain TCP for same-machine or LAN use without iroh's relay overhead
- [ ] **XPORT-03 (iAP)** — iPod Accessory Protocol for USB-connected iOS devices; potentially lower latency than RFCOMM

### Additional Bluetooth profiles

Expand beyond MAP (messaging) and PBAP (contacts).

- [ ] **BTP-01 (HFP)** — Hands-Free Profile for call history and call control
- [ ] **BTP-02 (OBEX FTP)** — File Transfer Profile for browsing and pulling files from the phone

---

## Known issues

---

## Completed

- [x] **OBS-02** — Proxy teardown log noise — `70f9b44`
- [x] **GAP-06** — Undelete error surface — `a36219b`
- [x] **STORE-01** — Encrypted SQLCipher message store — `68fef48`
- [x] **STORE-02** — Keyring-backed 256-bit DB key — `3dd359e`
- [x] **DISC-01** — Built-in device discovery (scan, picker, persist) — `1a584de`
- [x] **STORE-03** — Persistence layer: `watch` retired for the persistent `daemon` (`ca7d21c`), eager MNS body fetch (`91597b9`)

---

## Issues

Scoped bugs and gaps found during testing that don't warrant their own roadmap section. Tagged
`ISS-NN` here; moves to Completed with a commit hash once fixed.

- [ ] **ISS-01** — GUI channel overrides let the same channel be set for both messaging and
      contacts, which is always invalid, and a partial save can leave one written and the other
      not
- [ ] **ISS-02** — Automatic channel detection doesn't retry on a transient failure, unlike other
      connection attempts in the app
- [ ] **ISS-03** — The GUI treats a broken or unreadable config file the same as no device being
      configured, silently sending the user through first-time setup instead of showing an error
- [ ] **ISS-04** — Contact sync can report as never having run immediately after a successful
      sync, and can cache garbled contact names; not yet root-caused
- [ ] **ISS-05** — A sent message occasionally files into the wrong folder, which then causes the
      default delete action to target the wrong place; not yet root-caused
- [ ] **ISS-06** — The conversation view doesn't resolve or display the other party's contact name
- [ ] **ISS-07** — On first launch, device discovery for the picker runs only after the splash
      animation finishes instead of alongside it, adding avoidable delay
