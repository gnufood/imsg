# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.2.0] - 2026-06-27

### ⚠️ Breaking Changes

- **config:** Add store and broker configuration
- **transport:** Gate RFCOMM connect on BT_CONNECTED
- **session:** Replace reconnect module with retry policy

### Added

- **keyring:** Add Secret Service-backed database key management
- **store:** Add encrypted SQLCipher message store with V1 schema
- **store:** Add durable outbox and per-folder cursors in V2 schema
- **ipc:** Add broker wire-protocol frames and session-state types
- **config:** Add store and broker configuration
- **transport:** Gate RFCOMM connect on BT_CONNECTED
- **map:** Surface MNS events for the live session path
- **session:** Add sync coordinator with per-folder cursors and outbox drain
- **session:** Add live-query reads for the non-opted-in path
- **broker:** Add session broker with actor-model runtime
- **cli:** Add sync and unsync opt-in commands
- **cli:** Route reads through the store when opted in, live otherwise
- **cli:** Spawn and query the session broker

### Changed

- **session:** Replace reconnect module with retry policy

### Fixed

- **cli:** Inherit workspace version so cargo-release renders release commit
## [0.1.3] - 2026-06-15

### Fixed

- **transport:** Drain proxy tasks before endpoint close
- **cli:** Skip undelete — iOS ignores SetMessageStatus (GAP-06)
## [0.1.2] - 2026-06-14

### Fixed

- **watch:** Handle Ctrl+C in TUI and clean up key reader thread
## [0.1.1] - 2026-06-13

