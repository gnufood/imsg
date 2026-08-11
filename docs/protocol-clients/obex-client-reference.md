# OBEX Client Reference

The OBEX client is a sans-IO state machine that encodes OBEX requests and decodes responses. Callers provide the transport; this layer handles packet construction, connection state, and protocol correctness.

## State Machine

The client has two states:

| State | Description |
|-------|-------------|
| `Disconnected` | Initial state; no connection established. |
| `Connected` | Session active; carries `conn_id` and `max_packet`. |

### Transitions

```
Disconnected ──connect_request──> [send] ──handle_connect_response──> Connected
Connected ──disconnect_request──> [send] ──> Disconnected
```

All request methods except `connect_request` return `ObexError::NotConnected` when called from the `Disconnected` state.

## Operations

### Connect

**Request:** `connect_request(target_uuid, app_params)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `target_uuid` | `[u8; 16]` | 16-byte service UUID identifying the target profile (e.g., PBAP, MAP). |
| `app_params` | `Option<Bytes>` | Optional `AppParams` header; profile-specific capability bitmask. |

The request includes a `Target` header with the UUID and optionally `AppParams`. The opcode is `Connect` (0x80). The fixed extra bytes encode OBEX version 0x10, flags 0x00, and max packet size 0xFFFF.

**Response:** `handle_connect_response(data)`

On success, transitions to `Connected` and returns the server-assigned `ConnectionId` (u32). The response must contain a `ConnectionId` header; absence yields `ObexError::MissingConnectionId`. Non-OK opcodes yield `ObexError::ConnectRejected`.

### SetPath

**Request:** `setpath_request(name)`

Navigates to a child folder by name. Includes `ConnectionId` and `Name` headers. The flags byte is 0x02 (navigate to child, do not create).

**Request:** `setpath_backup_request()`

Navigates to the parent folder. Flags byte is 0x03 (backup bit set, no create). The `Name` header is empty.

Both methods return `ObexError::NotConnected` if called before a successful CONNECT exchange.

### Get

**Request:** `get_request(type_, name, app_params)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `type_` | `&[u8]` | Object type string, null-terminated ASCII (e.g., `x-bt/phonebook\x00`). |
| `name` | `Option<&str>` | Optional folder or object name. |
| `app_params` | `Option<Bytes>` | Optional application parameters blob. |

The request includes `ConnectionId`, `Type`, optionally `Name`, and optionally `AppParams`. Opcode is `GetFinal` (0x83).

**Request:** `get_continue_request()`

Continues a multi-packet GET after receiving `Continue` (0x90). Sends only `ConnectionId` with `GetFinal` opcode.

### Put

**Request:** `put_final_request(type_, extra_headers)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `type_` | `&[u8]` | Object type string. |
| `extra_headers` | `Vec<Header>` | Additional headers (e.g., `Name`, `Body`, `EndOfBody`). |

Prepends `ConnectionId` and `Type` before `extra_headers`. Opcode is `PutFinal` (0x82).

### Disconnect

**Request:** `disconnect_request()`

Sends `ConnectionId` in the DISCONNECT payload. Does not automatically transition to `Disconnected`; the caller must discard the client or re-initialize it.

### Response Parsing

**Method:** `parse_response(data)`

Stateless decode. Returns a `Packet` containing the opcode, any fixed extra bytes, and the header list. Does not advance client state.

## Connection State Queries

| Method | Returns |
|--------|---------|
| `conn_id()` | `Result<u32, ObexError>` — the assigned connection ID. Fails with `NotConnected` if disconnected. |
| `is_connected()` | `bool` — true after successful CONNECT. |
| `max_packet()` | `u16` — server-negotiated maximum packet size; 0 if disconnected. |

## Headers

The client uses these header types from `crate::headers::Header`:

| Header | Wire Tag | Value Type | Usage |
|--------|----------|------------|-------|
| `Target` | 0x46 | `Bytes` (16 bytes) | Service UUID in CONNECT. |
| `ConnectionId` | 0xCB | `u32` | Session handle; required in all post-CONNECT requests. |
| `Name` | 0x01 | `String` (UTF-16BE) | Folder or object name. |
| `Type` | 0x42 | `Bytes` (null-terminated ASCII) | Object type (e.g., `x-bt/phonebook\x00`). |
| `AppParams` | 0x4C | `Bytes` | Profile-specific TLV parameters. |
| `Body` | 0x48 | `Bytes` | Intermediate body chunk. |
| `EndOfBody` | 0x49 | `Bytes` | Final body chunk. |
| `Length` | 0xC3 | `u32` | Pre-declared object size. |
| `Srm` | 0x97 | `u8` | Single Response Mode control. |

## Errors

| Error | Condition |
|-------|-----------|
| `NotConnected` | Request method called before successful CONNECT. |
| `Packet(e)` | Packet encoding or decoding failure. |
| `ConnectRejected(opcode)` | Server returned non-OK opcode for CONNECT. |
| `MissingConnectionId` | CONNECT response lacks `ConnectionId` header. |
| `BodyTooLarge` | Message body exceeds 4 GiB (OBEX `Length` header limit). |

## Framing

OBEX uses length-prefix framing. Each packet begins with a 3-byte header:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 1 | Opcode |
| 1 | 2 | Total packet length (big-endian u16) |

The length includes the opcode and length field itself (minimum 3 bytes). The `ObexCodec` type in `crate::codec` implements `tokio_util::codec::Decoder` and `Encoder<Bytes>` by reading the u16 at bytes 1–2 to determine packet boundaries.

## Usage Pattern

Typical integration with a transport:

```rust
// Assuming a connected async stream `stream`
let mut transport = wrap(stream);
let mut obex = ObexClient::new();

// CONNECT
let req = obex.connect_request(&target_uuid, app_params)?;
transport.send(req).await?;
let rsp = transport.next().await.ok_or(...)?.map_err(...)?;
let conn_id = obex.handle_connect_response(&rsp)?;

// ... issue requests via obex.request_*() and send/recv on transport ...

// DISCONNECT
let req = obex.disconnect_request()?;
transport.send(req).await?;
```

Higher-level clients (MAP, PBAP) wrap `ObexClient` and add profile-specific request construction and response parsing. See [MAP Client Reference](./imsg-map/client-reference.md) and [PBAP Client Reference](./imsg-pbap/client-reference.md) for those layers.

## See Also

- [OBEX Packet Reference](./imsg-obex/packet-reference.md) — wire format details, opcode enumeration, and `PacketExtra` variants.
- [OBEX Headers Reference](./imsg-obex/headers-reference.md) — header encoding rules and tag bytes.
- [OBEX Server Reference](./imsg-obex/server-reference.md) — symmetric server state machine.