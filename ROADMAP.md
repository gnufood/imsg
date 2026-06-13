# Roadmap

> Last updated: 2026-06-13

---

## Planned

### Persistence layer (`store` crate)

Local encrypted message database so `imsg` works without the phone connected.

- [ ] `store` crate — SQLCipher (SQLite with full-database encryption)
- [ ] 256-bit key generated on first run, stored in OS keyring (GNOME Keyring / KWallet). Never on disk. No plaintext.
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

### OBS-02 — Proxy teardown log noise

`INFO hub proxy ch{n}: session closed` on every command. `copy_bidirectional` sees abort before clean EOF because the endpoint closes first. Fix: thread `Connection` through the command lifecycle, drain before `Endpoint::close`.

### GAP-06 — Undelete error surface

`imsg delete --undelete` exposes raw `RSP_NOT_IMPLEMENTED`. iOS doesn't implement MAP undelete. Map the response to a human-readable error.

---

## Completed

