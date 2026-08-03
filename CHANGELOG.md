# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-08-03

### Added

- **broker-client:** Extract shared IPC client crate
- **ipc:** Feature-gate specta::Type derives on the wire DTOs
- **map:** Expose EventType's string parser as a public FromStr impl
- **ipc:** Type WatchEvent.event_type instead of a raw wire string
- **gui:** Scaffold imsg-gui crate with local-store DTO conversions
- **gui:** Add local-store read path (open_store, list/get/threads wrappers)
- **broker:** Broadcast MessageDeleted on do_delete success
- **gui:** Add tauri/tauri-specta deps and #[tauri::command] read shims
- **broker-client:** Extract shared broker-response interpretation
- **gui:** Add config/unsync/daemon commands, restructure commands.rs by domain
- **broker-client,gui:** Land send/delete write path; revert unsync from GUI scope
- **gui:** Self-provision daemon, dedupe spawn mechanics, generate typed bindings
- **broker-client,cli:** Extract sync into imsg-broker-client::write
- **gui:** Add tauri.conf.json/build.rs/icons scaffold
- **config,gui:** Make MAP/PBAP RFCOMM channels GUI-editable
- **gui:** Scaffold Vite + React frontend with strict type-check/lint
- **gui:** Add daemon_restart command; eliminate result_large_err allows
- **transport,config,cli,gui:** Add bluesdp-based device discovery for GUI onboarding
- **gui:** Add Tailwind v4 + lucide-react, first atomic-design UI primitives
- **gui:** Add molecules/organisms tier, @/ import alias, motion for Splash
- **gui:** Main.rs entrypoint with Rust-owned startup gate, polled by the frontend
- **gui:** Add Storybook and split gate pages for full atomic-design conformance
- **gui:** Add read-only messages UI, enforce full atomic-design import boundaries
- **gui:** Add message send box
- **gui:** Mark conversation messages read on load
- **gui:** Add settings screen, persistent app shell, pane-collapse, delete-conversation
- **gui:** Add Storybook design-tokens docs page
- **gui:** Animate Modal enter/exit via Motion
- **gui:** Crossfade between messages and settings screens
- **gui:** Add light/dark/system appearance preference
- **gui:** Show real per-level daemon install status, redesign controls
- **gui:** Redesign Settings page — headers, layout, channel overrides
- **store,session,ipc,broker:** Cache PBAP contact display names for GUI thread list
- **config,session,transport,gui:** Configurable RFCOMM BT_SECURITY level for MAP connect
- **session:** Filter unread MAP messages device-side via FilterReadStatus
- **pbap,obex:** Add PBAP protocol capabilities confirmed against real device
- **store:** Rewrite contact cache as UID-keyed contacts + contact_phones
- **session,broker:** Version-aware contact sync, add shared live PBAP ops
- **ipc,broker:** Wire contract + dispatch for live PBAP contact ops
- **broker-client,cli:** Add contacts sync helper, rewire CLI contacts command
- **session,gui,cli:** Wire best-effort PBAP contacts sync into GUI startup
- **gui:** Add ThreadDto.contact_name and Tauri contacts read/sync commands
- **gui:** Auto-render cached contact names in the conversation UI
- **gui:** Add security-level (BT_SECURITY) settings UI
- **cli:** Split imsg into lib + bin, add shell completions
- **gui:** Redesign security-level settings as a single-card segmented control
- **gui:** Add WebDriver testing support behind a `webdriver` opt-in
- **formats,pbap,cli:** Normalize phone numbers via phonenumber, not digit-stripping
- **formats:** Add PhoneField pairing raw and canonical E.164
- **store,session,ipc,gui,cli:** Normalize phone numbers to E.164, match on canonical
- **cli:** Normalize all three phone inputs uniformly at dispatch
- **transport:** Report BlueZ device link state
- **broker:** Drop the session when the transport reports the link down
- **session:** Report what a sync did instead of a bare count
- **session,ipc,broker,cli:** Route `folders` through the broker
- **service,cli,gui:** Report whether uninstall removed anything
- **transport,gui:** Trace SDP channel resolution per profile
- **cli:** Support installing shell completions directly
- **gui:** Fade transition between splash gate and ready app

### Changed

- **gui:** Reorder Storybook sidebar to match atomic-design tiers
- **gui:** Centralize interactive-state classes, fix missing focus/disabled styling
- **gui:** Route remaining headings and story fixtures through the Text atom
- **gui:** Render Checkbox as an icon glyph instead of native styling
- **gui:** Tighten Button and TextInput vertical padding
- **gui:** Remove collapse from the conversation pane
- **workspace:** Fix broken and private intra-doc links across the workspace
- **store:** Stop returning the caller's own input length
- **ipc:** Split BrokerResponse out of proto.rs
- **service:** Split vocabulary types out of the crate root
- **cli:** Generate CLI reference into docs/cli.md instead of embedding in README
- **roadmap:** Tighten prose across prefix and section descriptions
- **roadmap:** Rename tag prefixes and renumber open items after pruning
- **readme:** Rewrite around desktop app and CLI as equal peers
- **readme:** Add CI, crates.io version, MSRV, and license badges
- **gui:** Rewrite frontend README to reflect actual GUI capabilities

### Fixed

- **gui:** Animate pane collapse via width/opacity, not layout-projection scale
- **broker:** Hold a persistent PBAP session in the device actor
- **gui:** Add contact_name to ThreadDto story fixtures
- **gui:** Reposition unread badge to row corner, drop seconds from timestamps
- **gui:** Flatten unread badge to accent icon, drop filled pill
- **gui:** Enable tauri's custom-protocol feature by default
- **store,session,cli:** Derive sync freshness from per-folder cursors
- **store:** Allow deleting a message still referenced by outbox
- **gui:** Stop offering a system-service install that can't succeed
- **gui:** Close the channel editor when the save lands
- **gui:** Fit post-gate screens to the window instead of the viewport
- **obex,formats:** Zero-copy decode, single-buffer encode, TYPE field
- **cli:** Inherit workspace rust-version so published metadata isn't empty
- **cliff:** Stop letting chore commits escape the skip rule via "remove"
- **gui:** Stop imsg-gui from being publish-eligible

### Security

- **deps:** Bump quick-xml to 0.41.0 (RUSTSEC-2026-0194, RUSTSEC-2026-0195)
- **cliff:** Stop matching identifier names as security disclosures
## [0.3.1] - 2026-07-08

### Fixed

- **deps:** Bump iroh to 1.0 for patched hickory-dns
- **changelog:** Move security rule before fix parser
## [0.3.0] - 2026-07-08

### ⚠️ Breaking Changes

- **cli:** Remove watch command in favor of daemon
- **broker:** Close daemon-mode reliability gaps

### Added

- **daemon:** Idle-optional actor lifecycle, eager MNS start, event-to-store wiring
- **daemon:** Persistent entry point and graceful shutdown coordinator
- **cli:** Imsg daemon start/stop/status, detaching by default
- **service:** Add imsg-service crate for OS service-manager integration
- **cli:** Add imsg daemon install/uninstall
- **cli:** Remove watch command in favor of daemon
- **service:** Resolve real invoking user for --system daemon installs
- **cli:** Resolve daemon install's default --device under --system
- **cli:** Announce daemon connection status under --foreground

### Changed

- **service:** Flag start/stop/status as unwired and unverified
- Document imsg daemon in the README

### Fixed

- **daemon:** Case-insensitive folder match, persist MNS events, always fan out
- **daemon:** Retry the initial connect indefinitely in persistent mode
- **cli:** Daemon start/stop/status log permissions and labeling
- **session:** Re-fetch true status on ReadStatusChanged instead of assuming read
- **broker:** Close daemon-mode reliability gaps
- **service:** Stop the service before uninstalling
- **broker:** Register MNS with BlueZ before enabling notifications, not after
- **broker:** Close the daemon-stop race that resets in-flight connections
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

