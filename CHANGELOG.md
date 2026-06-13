# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-12

### Added
- Read, send, and delete SMS/MMS over Bluetooth MAP (iPhone via BlueZ)
- List messages across inbox, sent, outbox, and deleted folders; filter by
  sender (`--from`), date (`--since`), read status (`--unread`), count
  (`--limit`), and offset; `--long` to show MAP handles for use with `get`/`delete`
- Fetch message body with optional mark-read (`--mark-read`)
- Browse the MAP folder tree on the device (`folders`)
- Conversation threads grouped from inbox and sent by contact, most-recent-first
- Live MAP notification events (`watch`); ratatui TUI panel with scrollable
  event log when built with `--features tui`
- Pull contacts and call history (incoming/outgoing/missed/combined) via PBAP;
  list handles, fetch by handle, or reverse-lookup by number; E.164 normalisation
  by default (`--raw` to skip); pagination via `--limit`/`--page`
- Cross-machine access via iroh: run `imsg hub` on the paired machine, connect
  from anywhere with `imsg spoke add <key>`; hub key persists across restarts;
  single-instance lock prevents conflicting hubs
- Inspect the resolved config (`config show`) and set the paired device address
  (`config set-device <MAC>`); layered config from compiled defaults → `/etc` →
  XDG → local `imsg.toml` → explicit `--config` path → `IMSG_*` env vars;
  configurable RFCOMM channels (`map_channel`, `pbap_channel`); per-invocation
  device override (`--device`); verbosity control (`-v`/`-vv`/`-q`)
