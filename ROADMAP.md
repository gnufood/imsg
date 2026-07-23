# Roadmap

> Last updated: 2026-07-22

---

## Planned

### Standardized error + logging (CLI + GUI)

The CLI uses `anyhow` throughout; the GUI has its own `CommandError` (`imsg-gui/src/commands.rs`)
requiring a hand-written `From` impl per domain error, which already lags behind new features.
Logging is also duplicated — `cli`/`main.rs` and `imsg-gui/src/main.rs` each hand-roll their own
`tracing_subscriber` init.

- [ ] Unify or share the error-conversion boundary between CLI and GUI
- [ ] Extract one shared `tracing_subscriber` init used by both entrypoints

### Broker mode for hub and spoke

`imsg hub` proxies raw MAP/PBAP RFCOMM per-connection today — it doesn't use the broker/daemon's
persistent session handling at all, so the hub/spoke path gets none of the benefit of the
persistent-PBAP-session work already shipped for the local/direct path.

- [ ] Extend broker/daemon session handling to the hub/spoke topology

### Send attachments

MAP supports advertising MMS, but Apple's SDP record doesn't advertise it, and the Attachment
flag isn't respected by the device — attachment support can't be driven off protocol metadata
alone. Messages are also UTF-8 constrained.

- [ ] Investigate a non-metadata-driven detection/transfer path for MMS content

### Export

- [ ] Add an export command (format/target TBD)

### Persistence layer (`store` crate)

Local encrypted message database so `imsg` works without the phone connected.

- [x] `watch` retired in favor of the persistent `daemon` — writes incoming messages to DB on MNS event
- [x] Body fetch strategy decided: eager (full content fetched on every MNS `NewMessage` event)

### Built-in device discovery

Eliminates `imsg config set-device <MAC>`.

- [x] Scan paired Bluetooth devices on startup — `imsg config setup`
- [x] Interactive picker when no device is configured
- [x] Write selected MAC + resolved channels to config

### Full TUI client

Interactive TUI for the CLI (the desktop GUI covers the graphical case separately).

- [ ] Scrollable thread list (arrow-key navigation)
- [ ] Message reader pane (selected thread contents)
- [ ] Compose / reply pane
- [ ] Unified layout with live MNS updates

### Fuzzy + semantic search of conversations

No search of any kind exists today — `list`/`get`/`threads` only filter by folder/sender/date.

- [ ] Fuzzy text match over local message bodies (typo-tolerant)
- [ ] Semantic search — embedding-based similarity search over the local store, for
      meaning-based queries rather than exact/fuzzy text match

### Advanced security

Post-store-crate hardening.

- [ ] Per-record encryption — independent key per message
- [ ] Audit log — track decryption access (who read what, when)
- [ ] Key rotation — re-key DB without full rewrite

### Library + SDKs

Stable public API for building on top of imsg.

- [ ] Define and stabilize a public Rust API surface, with real semver discipline, separate
      from the internal workspace crates
- [ ] Version the broker IPC protocol (`imsg-ipc`) — `BrokerRequest`/`BrokerResponse` currently
      carry no version tag/handshake, so a long-lived `daemon` and an upgraded CLI have no way
      to detect a wire-format mismatch
- [ ] C FFI layer for cross-language binding
- [ ] Python SDK (PyO3 / maturin)
- [ ] Go SDK (cgo)

### Web UI

Browser-based interface as an alternative to the CLI/TUI.

- [ ] Serves the local message DB over a local HTTP server
- [ ] Thread list, message view, compose / reply in the browser
- [ ] Real-time updates via SSE or WebSocket from the daemon

---

## Research

Items that need protocol investigation or feasibility work before they land on the Planned list.

### Additional transport protocols

Beyond RFCOMM (Classic Bluetooth) and iroh (hub/spoke QUIC).

- [ ] **BLE** — investigate ANCS for notification delivery; MAP requires Classic Bluetooth, so full message sync over BLE is unproven
- [ ] **TCP** — scaffolded (`imsg-transport::tcp::connect`), not yet wired into any caller; plain TCP for same-machine or LAN use without iroh's relay overhead
- [ ] **iAP** — iPod Accessory Protocol for USB-connected iOS devices; potentially lower latency than RFCOMM

### Additional Bluetooth profiles

Expand beyond MAP (messaging) and PBAP (contacts).

- [ ] **HFP** — Hands-Free Profile for call history and call control
- [ ] **OBEX FTP** — File Transfer Profile for browsing and pulling files from the phone

---

## Known issues

---

## Tags

Section-scoped — one prefix per Planned/Research section above, numbered sequentially as
sub-items in that section ship:

| Prefix | Section |
|---|---|
| `ERR` | Standardized error + logging |
| `HUB` | Broker mode for hub and spoke |
| `ATT` | Send attachments |
| `EXP` | Export |
| `STORE` | Persistence layer |
| `DISC` | Built-in device discovery |
| `TUI` | Full TUI client |
| `SEARCH` | Fuzzy + semantic search |
| `SEC` | Advanced security |
| `API` | Library + SDKs |
| `WEB` | Web UI |
| `XPORT` | Additional transport protocols (Research) |
| `BTP` | Additional Bluetooth profiles (Research) |

Cross-cutting — fixes that don't map to a roadmap line item:

| Prefix | Meaning |
|---|---|
| `OBS` | Observability — logging/tracing/diagnostics fixes |
| `GAP` | Protocol/device compliance gap — workaround for something the spec implies but the device doesn't honor |

---

## Completed

- [x] **OBS-02** — Proxy teardown log noise — `70f9b44`
- [x] **GAP-06** — Undelete error surface — `a36219b`
- [x] **STORE-01** — Encrypted SQLCipher message store — `68fef48`
- [x] **STORE-02** — Keyring-backed 256-bit DB key — `3dd359e`
- [x] **DISC-01** — Built-in device discovery (scan, picker, persist) — `1a584de`
- [x] **STORE-03** — Persistence layer: `watch` retired for the persistent `daemon` (`ca7d21c`), eager MNS body fetch (`91597b9`)
