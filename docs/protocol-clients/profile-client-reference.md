# MAP and PBAP Client API Reference

This reference describes the Message Access Profile (MAP) and Phone Book Access Profile (PBAP) client interfaces, which provide Bluetooth OBEX-based access to device message stores and phonebooks. Both profiles operate over RFCOMM and use OBEX as their transport layer.

## MAP Client

The MAP client (`MapClient<T>`) provides access to a device's message store. It operates over an async transport implementing `AsyncRead + AsyncWrite + Unpin`.

### Session Lifecycle

#### Connection

```rust
pub async fn connect(stream: T) -> Result<Self, MapError>
```

Establishes a MAP session by sending the MAP UUID (`bb582b40-420c-11db-b0de-0800200c9a66`) as the OBEX `Target` header. The server must respond with OK and include a `ConnectionId` header. The transport is wrapped in OBEX framing automatically.

**Errors:** Returns `MapError` if the transport fails, packet encoding fails, the server rejects the connection, or the response omits the `ConnectionId` header.

#### Disconnection

```rust
pub async fn disconnect(self) -> Result<(), MapError>
```

Sends OBEX DISCONNECT and awaits the server acknowledgement. Consumes `self` — the session is unusable after this call regardless of outcome. Does not close the underlying stream; the stream is dropped when `self` is consumed.

**Errors:** Returns `MapError` if the request cannot be encoded, the transport fails, or the server returns a non-OK response.

### Folder Navigation

MAP uses a hierarchical folder structure rooted at `telecom/msg`. The client maintains a depth counter (0 = root, maximum 3) confirmed by the server.

#### Setting Current Folder

```rust
pub async fn set_folder(&mut self, folder: Folder) -> Result<(), MapError>
```

Navigates to a message folder. If already inside a subfolder, backs up to root first, then navigates `telecom` → `msg` → folder. iOS requires one SETPATH per level; a single slash-joined path is rejected.

**Folder values:** `Folder::Inbox`, `Folder::Outbox`, `Folder::Sent`, `Folder::Deleted`, `Folder::Draft`, or custom folder names.

**Errors:** Returns `MapError` if any SETPATH fails to encode, the transport closes, or the server returns a non-OK response for any step.

#### Reset to Root

```rust
pub async fn reset_to_root(&mut self) -> Result<(), MapError>
```

Navigates to root by issuing backup SETPATH operations for each depth level. No-op when already at root (`depth == 0`). Does not navigate anywhere after resetting.

**Errors:** Returns `MapError` if any backup SETPATH fails to encode, the transport closes, or the server returns a non-OK response.

### Message Listing

```rust
pub async fn list_messages(
    &mut self,
    filter: &ListMessagesFilter,
) -> Result<Vec<MessageEntry>, MapError>
```

Retrieves a list of messages from the current folder. The caller must navigate to the target folder via `set_folder` before calling this. Sends a `GetMessagesListing` GET with no Name header — the device lists the current OBEX working directory. Accumulates body chunks across CONTINUE responses before parsing.

**Errors:** Returns `MapError` if the transport fails, the server rejects the request, or the response XML is malformed.

### Message Retrieval

```rust
pub async fn get_message(&mut self, handle: &str) -> Result<BMessage, MapError>
```

Fetches a single message by handle. Sends a `GetMessage` GET with `Type: x-bt/message` and `Charset=UTF-8`. Accumulates body chunks across CONTINUE responses before parsing.

**Input constraints:** `handle` must not be empty and must not contain CR or LF — it lands verbatim in the OBEX Name header.

**Errors:** Returns `MapError::InvalidInput` if `handle` is empty or contains CR or LF. Returns `MapError` if the transport fails, the server rejects the request, the response body exceeds 4 MiB, is not valid UTF-8, or the bMessage is malformed.

### Message Sending

```rust
pub async fn push_message(&mut self, phone: &str, text: &str) -> Result<String, MapError>
```

Sends `text` as an outbound SMS to `phone` via MAP `PushMessage`. Returns the opaque handle assigned by the remote, suitable for passing to `set_message_status_read` or `set_message_status_deleted`. The caller must have navigated to outbox via `set_folder(Folder::Outbox)` first.

**Input constraints:** `phone` must not contain CR or LF.

**Errors:** Returns `MapError` if encoding fails, the transport closes, the server rejects the request, or the OK response contains no Name header.

### Status Operations

```rust
pub async fn set_message_status_read(
    &mut self,
    handle: &str,
    status: MessageStatus,
) -> Result<(), MapError>

pub async fn set_message_status_deleted(
    &mut self,
    handle: &str,
    deleted: bool,
) -> Result<(), MapError>
```

Marks the message identified by `handle` as read/unread or deleted/undeleted via MAP `SetMessageStatus`. The caller must navigate to the containing folder via `set_folder` first.

**MessageStatus values:**
- `MessageStatus::Read` — wire value `0x01`
- `MessageStatus::Unread` — wire value `0x00`

**Input constraints:** `handle` must not be empty and must not contain CR or LF.

**Errors:** Returns `MapError::InvalidInput` if `handle` is empty or contains CR or LF. Returns `MapError::ServerError` if the remote returns a non-OK response. Returns `MapError::Obex` or `MapError::Transport` on lower-layer failure.

### Folder Listing

```rust
pub async fn get_folder_listing(&mut self) -> Result<FolderListing, MapError>
```

Returns the MAP folder listing for the current object store level via `GetFolderListing` GET with Type `x-obex/folder-listing`. Folders are in device-reported document order.

**Errors:** Returns `MapError::ServerError` if the remote returns a non-OK response. Returns `MapError::FolderListing` if the response body is malformed XML. Returns `MapError::Obex` or `MapError::Transport` on lower-layer failure.

### Notification Registration

```rust
pub async fn set_notification_registration(&mut self, enable: bool) -> Result<(), MapError>
```

When `enable` is `true`, the phone will connect to the MNS channel and push event reports. When `false`, it stops. The caller must keep the MAP session alive while notifications are active.

**Errors:** Returns `MapError::ServerError` if the remote returns a non-OK response. Returns `MapError::Obex` or `MapError::Transport` on lower-layer failure.

### Hold

```rust
pub async fn hold(&mut self) -> Result<(), MapError>
```

Reads from the transport until the remote closes the stream, discarding all received packets. Returns `Ok(())` on clean close. Does not send OBEX DISCONNECT and does not parse received packet opcodes.

**Errors:** Returns `MapError::Transport` on a framing error from the underlying codec.

## MAP Message Types

### ListMessagesFilter

```rust
pub struct ListMessagesFilter {
    pub max_count: u16,           // Maximum entries to return; capped by device
    pub offset: u16,              // Zero-based index of first entry
    pub read_status: Option<ReadStatus>,
    pub originating_address: Option<String>,
    pub period_begin: Option<String>,  // Format: YYYYMMDDTHHMMSS[±HHMM]
    pub period_end: Option<String>,    // Format: YYYYMMDDTHHMMSS[±HHMM]
}
```

**Default:** Fetches up to 1024 messages with no filters applied.

**Constraints:** String filters must not contain null bytes and must not exceed 254 UTF-8 bytes.

### ReadStatus

```rust
pub enum ReadStatus {
    Unread,  // Wire value 0x01; excludes read messages
    Read,    // Wire value 0x02; excludes unread messages
}
```

### MessageEntry

```rust
pub struct MessageEntry {
    pub handle: String,           // Opaque device-assigned message handle (hex string)
    pub subject: String,          // Message subject or first line of body
    pub datetime: String,         // Timestamp in MAP format
    pub sender_name: String,      // Sender display name; empty if unavailable
    pub sender_addressing: String, // Sender address (phone number or email)
    pub recipient_name: String,   // Recipient display name; empty for received
    pub recipient_addressing: String, // Recipient address; empty for received
    pub msg_type: String,         // Message type: SMS_GSM, SMS_CDMA, EMAIL, MMS
    pub size: u32,                // Body size in bytes as reported by device
    pub read: bool,               // Device-reported read state
    pub sent: bool,               // true for outbound messages; false for received
}
```

Absent attributes default to zero/false/empty.

## MAP Application Parameters

### MessageTypes Bitmask

```rust
pub struct MessageTypes: u8 {
    const SMS_GSM  = 0x01;
    const SMS_CDMA = 0x02;
    const EMAIL    = 0x04;
    const MMS      = 0x08;
    const IM       = 0x10;
}
```

### FilterMessageType Bitmask

```rust
pub struct FilterMessageType: u8 {
    const SMS_GSM  = 0x01;
    const SMS_CDMA = 0x02;
    const EMAIL    = 0x04;
    const MMS      = 0x08;
}
```

### Application Parameter Tags

| Tag | Name | Description |
|-----|------|-------------|
| `0x01` | `MAX_LIST_COUNT` | Maximum entries to return |
| `0x02` | `LIST_START_OFFSET` | Zero-based starting index |
| `0x03` | `FILTER_MESSAGE_TYPE` | Bitmask of message types to include |
| `0x04` | `FILTER_PERIOD_BEGIN` | Earliest timestamp filter |
| `0x05` | `FILTER_PERIOD_END` | Latest timestamp filter |
| `0x06` | `FILTER_READ_STATUS` | Read/unread filter |
| `0x07` | `FOLDER_LISTING_SIZE` | Response-only: total folder count |
| `0x08` | `FILTER_ORIGINATOR` | Sender address filter |
| `0x0A` | `TRANSPARENT` | If set, message not copied to sent folder |
| `0x0B` | `RETRY` | If set, retry delivery on transient failure |
| `0x0D` | `NEW_MESSAGE` | Response-only: 1 if unread messages exist |
| `0x0E` | `NOTIFICATION_STATUS` | 1 = enable MNS, 0 = disable |
| `0x12` | `MESSAGES_LISTING_SIZE` | Response-only: total message count |
| `0x14` | `CHARSET` | Character set (0x01 = UTF-8) |
| `0x17` | `STATUS_INDICATOR` | Status property to update |
| `0x18` | `STATUS_VALUE` | New value for status property |

### Status Indicator Values

- `INDICATOR_READ_STATUS` (`0x00`) — selects read/unread state
- `INDICATOR_DELETED_STATUS` (`0x01`) — selects deleted/present state

## PBAP Client

The PBAP client (`PbapClient<T>`) provides access to a device's phonebook. It operates over an async transport implementing `AsyncRead + AsyncWrite + Unpin`.

### Session Lifecycle

#### Connection

```rust
pub async fn connect(stream: T) -> Result<Self, PbapError>
```

Establishes a PBAP session by sending the PBAP PSE UUID as OBEX `Target` plus `PBAPSupportedFeatures` (`Download | DatabaseIdentifier | FolderVersionCounters` = `0x0000000D`). Validates the server response. Does not validate the RFCOMM channel.

**Errors:** Returns `PbapError::Obex` if the server rejects the connection or the response omits the `ConnectionId` header. Returns `PbapError::Transport` on I/O failure.

#### Disconnection

```rust
pub async fn disconnect(self) -> Result<(), PbapError>
```

Sends OBEX DISCONNECT and checks the response opcode only. Does not close the underlying stream.

**Errors:** Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure. Returns `PbapError::ServerError` if the remote returns a non-OK response.

### Phonebook Operations

#### Pull All Contacts

```rust
pub async fn pull_all(
    &mut self,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> Result<Vec<Contact>, PbapError>
```

Performs `PullPhoneBook` for `path`, windowed to `limit` entries starting at `offset` (device-side `MaxListCount`/`ListStartOffset`). `limit: None` fetches everything. Returns contacts in device-reported order. Silently skips unparseable vCards. Does not normalize numbers or filter `0.vcf`.

**Errors:** Returns `PbapError::ServerError` if the remote returns a non-OK response. Returns `PbapError::ResponseTooLarge` if the body exceeds 4 MiB. Returns `PbapError::InvalidEncoding` if the body is not valid UTF-8. Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure.

#### Phonebook Metadata

```rust
pub async fn phonebook_metadata(
    &mut self,
    path: PhonebookPath,
) -> Result<PhonebookMetadata, PbapError>
```

Performs a metadata-only `PullPhoneBook` (`MaxListCount=0`): no vCard body is fetched, just `PhonebookSize`/`DatabaseIdentifier`/version-counter fields in the response's `AppParams`. Fields the device omits come back `None` in `PhonebookMetadata`.

**Requirements:** `DatabaseIdentifier` and version counters require declaring `DatabaseIdentifier`/`FolderVersionCounters` in `PBAPSupportedFeatures` at CONNECT time.

**Errors:** Returns `PbapError::ServerError` if the remote returns a non-OK response. Returns `PbapError::ResponseTooLarge` if the body exceeds 4 MiB. Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure.

#### List vCard Entries

```rust
pub async fn list(
    &mut self,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> Result<Vec<CardEntry>, PbapError>
```

Performs `ListvCardObjects` for `path`, windowed to `limit` entries starting at `offset`. `limit: None` and `offset: 0` omits both and fetches everything. Returns entries in device-reported order. Does not filter `0.vcf` or fetch vCard content.

**Errors:** Returns `PbapError::ServerError` if the remote returns a non-OK response. Returns `PbapError::CardListing` if the listing XML cannot be parsed. Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure.

#### Search

```rust
pub async fn search(
    &mut self,
    path: PhonebookPath,
    attribute: SearchAttribute,
    value: &str,
    limit: Option<u16>,
    offset: u16,
) -> Result<Vec<CardEntry>, PbapError>
```

Performs `ListvCardObjects` filtered device-side by `SearchAttribute`/`SearchValue`: entries whose `attribute` field matches `value`. Windowed the same way as `list`.

**Input constraints:** `value` must not contain CR or LF and must not exceed 255 UTF-8 bytes.

**Errors:** Returns `PbapError::InvalidInput` if `value` contains CR or LF or exceeds 255 UTF-8 bytes. Returns `PbapError::ServerError` if the remote returns a non-OK response. Returns `PbapError::CardListing` if the listing XML cannot be parsed. Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure.

#### Pull Single Entry

```rust
pub async fn pull(&mut self, path: PhonebookPath, handle: &str) -> Result<Contact, PbapError>
```

Performs `PullvCardEntry` for the given handle. Silently ignores unrecognised vCard properties; does not normalize numbers.

**Input constraints:** `handle` must be non-empty and must not contain CR or LF.

**Errors:** Returns `PbapError::InvalidInput` if `handle` is empty or contains CR or LF. Returns `PbapError::ServerError` if the remote returns a non-OK response. Returns `PbapError::ResponseTooLarge` if the body exceeds 4 MiB. Returns `PbapError::InvalidEncoding` if the body is not valid UTF-8. Returns `PbapError::Contact` if calcard cannot parse the vCard. Returns `PbapError::Transport` or `PbapError::Obex` on lower-layer failure.

## PBAP Phonebook Paths

```rust
pub enum PhonebookPath {
    Pb,   // telecom/pb — main phonebook
    Ich,  // telecom/ich — incoming call history
    Och,  // telecom/och — outgoing call history
    Mch,  // telecom/mch — missed call history
    Cch,  // telecom/cch — combined call history
    Spd,  // telecom/spd — speed dial
    Fav,  // telecom/fav — favourites
}
```

### OBEX Name Header Values

| Path | `pull_name()` | `list_name()` |
|------|---------------|---------------|
| `Pb` | `telecom/pb.vcf` | `telecom/pb` |
| `Ich` | `telecom/ich.vcf` | `telecom/ich` |
| `Och` | `telecom/och.vcf` | `telecom/och` |
| `Mch` | `telecom/mch.vcf` | `telecom/mch` |
| `Cch` | `telecom/cch.vcf` | `telecom/cch` |
| `Spd` | `telecom/spd.vcf` | `telecom/spd` |
| `Fav` | `telecom/fav.vcf` | `telecom/fav` |

### Entry Name

```rust
pub fn entry_name(self, handle: &str) -> String
```

Returns the OBEX `Name` header value for a `PullvCardEntry` GET, e.g. `telecom/pb/41.vcf` for `Pb` with handle `41.vcf`. Does not validate `handle`.

## PBAP Search Attributes

```rust
pub enum SearchAttribute {
    Name,   // Wire value 0x00 — match against contact name
    Number, // Wire value 0x01 — match against phone number
    Sound,  // Wire value 0x02 — match against phonetic sound
}
```

## PBAP Contact Entries

```rust
pub struct CardEntry {
    handle: String,
    name: Option<String>,
}
```

A phonebook entry from a `ListvCardObjects` response. The handle is the opaque identifier to pass to `PbapClient::pull`.

```rust
pub fn handle(&self) -> &str  // Opaque vCard handle, e.g. "41.vcf"
pub fn name(&self) -> Option<&str>  // Display name from listing XML; None if absent
```

## PBAP Phonebook Metadata

```rust
pub struct PhonebookMetadata {
    pub size: Option<u16>,           // Total contact count in the requested phonebook
    pub database_id: Option<[u8; 16]>, // Fixed identifier for the phone's contacts database
    pub primary_version: Option<[u8; 16]>, // Monotonic counter for handle set changes
    pub secondary_version: Option<[u8; 16]>, // Monotonic counter for entry property changes
}
```

Fields are `None` when the device omitted them. `database_id` changes only if the underlying database itself changes (e.g. restored from a different backup); stable across reconnects otherwise. `primary_version` increments when contacts are added or removed. `secondary_version` increments when existing entry properties change.

## PBAP Application Parameters

### Connect Parameters

```rust
pub const fn connect_params() -> Bytes
```

Returns `PBAPSupportedFeatures` app-params: `Download | DatabaseIdentifier | FolderVersionCounters` (`0x0000000D`). Wire encoding: `[0x10, 0x04, 0x00, 0x00, 0x00, 0x0D]`. Declaring this bit combination at CONNECT is required to unlock `DatabaseIdentifier`/`PrimaryVersionCounter`/`SecondaryVersionCounter` in later `PullPhoneBook` responses.

### Pull All Parameters

```rust
pub fn pull_all_params(limit: Option<u16>, offset: u16) -> Bytes
```

`PullPhoneBook` app-params: `Format=vcard30`, `PROPERTY_SELECTOR` (VERSION | FN | TEL | UID), `MaxListCount=limit`. `0xFFFF` when `limit` is `None`. `ListStartOffset` included only when `offset != 0`.

### List Parameters

```rust
pub fn list_params(limit: Option<u16>, offset: u16) -> Option<Bytes>
```

`ListvCardObjects` app-params: `MaxListCount`/`ListStartOffset`. Returns `None` for the "fetch everything" default.

### Search Parameters

```rust
pub fn search_params(
    attribute: SearchAttribute,
    value: &str,
    limit: Option<u16>,
    offset: u16,
) -> Bytes
```

`ListvCardObjects` search app-params: `SearchAttribute`, `SearchValue` (raw UTF-8, length-prefixed), plus windowing. `value` must already be validated by the caller: at most 255 UTF-8 bytes.

### Pull Entry Parameters

```rust
pub const fn pull_entry_params() -> Bytes
```

`PullvCardEntry` app-params: `Format=vcard30`, `PROPERTY_SELECTOR`. Wire encoding: `[0x07, 0x01, 0x01, 0x06, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x83]`.

### Metadata Parameters

```rust
pub const fn metadata_params() -> Bytes
```

`PullPhoneBook` metadata-only app-params: `Format=vcard30`, `MaxListCount=0`. Triggers a response with no vCard body. Wire encoding: `[0x07, 0x01, 0x01, 0x04, 0x02, 0x00, 0x00]`.

## Common OBEX Primitives

Both MAP and PBAP clients use the underlying OBEX client state machine (`ObexClient`) for packet construction and response parsing.

### Connection State

```rust
pub fn is_connected(&self) -> bool  // True after successful CONNECT
pub fn conn_id(&self) -> Result<u32, ObexError>  // Connection ID from server
pub fn max_packet(&self) -> u16  // Server-negotiated max packet size; 0 if not connected
```

### Request Construction

The OBEX client provides methods for constructing requests:

- `connect_request(target_uuid, app_params)` — CONNECT with Target and optional AppParams
- `setpath_request(name)` — SETPATH navigate-to-child
- `setpath_backup_request()` — SETPATH backup to parent
- `get_request(type_, name, app_params)` — GET FINAL with Type, optional Name, optional AppParams
- `get_continue_request()` — GET FINAL with only ConnectionId (continues after Continue)
- `put_final_request(type_, extra_headers)` — PUT FINAL with Type and extra headers
- `disconnect_request()` — DISCONNECT with ConnectionId

### Response Parsing

```rust
pub fn parse_response(data: &Bytes) -> Result<Packet, ObexError>
```

Stateless decode — does not advance client state.

## Response Size Limits

Both MAP and PBAP clients enforce a maximum response body size of 4 MiB (`MAX_BODY_BYTES`). Responses exceeding this limit return `MapError::ResponseTooLarge` or `PbapError::ResponseTooLarge` respectively.

## Error Types

### MAP Errors

- `MapError::InvalidInput` — invalid input parameters (empty handle, invalid characters)
- `MapError::ServerError(u8)` — server returned non-OK response with given opcode
- `MapError::MissingHandle` — push response missing Name header
- `MapError::InvalidEncoding` — response body is not valid UTF-8
- `MapError::ResponseTooLarge` — body exceeds 4 MiB
- `MapError::FolderListing` — malformed folder listing XML
- `MapError::Obex` — OBEX protocol error
- `MapError::Transport` — transport I/O error

### PBAP Errors

- `PbapError::InvalidInput` — invalid input parameters
- `PbapError::ServerError(u8)` — server returned non-OK response
- `PbapError::ResponseTooLarge` — body exceeds 4 MiB
- `PbapError::InvalidEncoding` — response body is not valid UTF-8
- `PbapError::CardListing` — malformed vCard listing XML
- `PbapError::Contact` — calcard failed to parse vCard
- `PbapError::Obex` — OBEX protocol error
- `PbapError::Transport` — transport I/O error

### OBEX Errors

- `ObexError::NotConnected` — called before successful CONNECT
- `ObexError::Packet(PacketError)` — packet codec failure
- `ObexError::ConnectRejected(u8)` — server refused CONNECT
- `ObexError::MissingConnectionId` — CONNECT response missing ConnectionId
- `ObexError::BodyTooLarge` — body exceeds 4 GiB

## See Also

- [MNS Server Reference](mns-server-reference.md) — MAP notification server for receiving event reports
- [OBEX Codec Reference](obex-codec-reference.md) — OBEX packet encoding and decoding