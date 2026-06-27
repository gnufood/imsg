# Roadmap

> Last updated: 2026-06-27

---

## Planned

### Persistence layer (`store` crate)

Local encrypted message database so `imsg` works without the phone connected.

- [ ] `watch` upgraded to sync daemon — writes incoming messages to DB on MNS event
- [ ] Body fetch strategy: eager (full content on every MNS event) vs. lazy (metadata only, fetch on open)

### Built-in device discovery

Eliminates `imsg config set-device <MAC>`.

- [ ] Scan paired Bluetooth devices on startup
- [ ] Interactive picker when no device is configured
- [ ] Write selected MAC to config

### Full TUI client

Interactive TUI beyond the current watch panel.

- [ ] Scrollable thread list (arrow-key navigation)
- [ ] Message reader pane (selected thread contents)
- [ ] Compose / reply pane
- [ ] Unified layout with live MNS updates

### Advanced security

Post-store-crate hardening.

- [ ] Per-record encryption — independent key per message
- [ ] Audit log — track decryption access (who read what, when)
- [ ] Key rotation — re-key DB without full rewrite

### Library + SDKs

Stable public API for building on top of imsg.

- [ ] Publish workspace crates to crates.io (`imsg-session`, `imsg-map-core`, etc.) — currently `publish = false`
- [ ] C FFI layer for cross-language binding
- [ ] Python SDK (PyO3 / maturin)
- [ ] Go SDK (cgo)

### Web UI

Browser-based interface as an alternative to the CLI/TUI.

- [ ] Serves the local message DB over a local HTTP server
- [ ] Thread list, message view, compose / reply in the browser
- [ ] Real-time updates via SSE or WebSocket from the watch sync daemon

---

## Research

Items that need protocol investigation or feasibility work before they land on the Planned list.

### Additional transport protocols

Beyond RFCOMM (Classic Bluetooth) and iroh (hub/spoke QUIC).

- [ ] **BLE** — investigate ANCS for notification delivery; MAP requires Classic Bluetooth, so full message sync over BLE is unproven
- [ ] **TCP** — plain TCP transport for same-machine or LAN use without iroh's relay overhead
- [ ] **iAP** — iPod Accessory Protocol for USB-connected iOS devices; potentially lower latency than RFCOMM

### Additional Bluetooth profiles

Expand beyond MAP (messaging) and PBAP (contacts).

- [ ] **HFP** — Hands-Free Profile for call history and call control
- [ ] **OBEX FTP** — File Transfer Profile for browsing and pulling files from the phone

---

## Known issues

---

## Completed

- [x] **OBS-02** — Proxy teardown log noise — `70f9b44`
- [x] **GAP-06** — Undelete error surface — `a36219b`
- [x] **STORE-01** — Encrypted SQLCipher message store — `68fef48`
- [x] **STORE-02** — Keyring-backed 256-bit DB key — `3dd359e`

