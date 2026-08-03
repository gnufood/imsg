# imsg desktop frontend

React + TypeScript + Vite frontend for the Tauri-based `imsg-gui` desktop app. Talks to the
Rust backend exclusively through generated bindings (`src/bindings.ts`) — never hand-written,
regenerate after any backend command/type change:

```sh
cargo run --example export_bindings -p imsg-gui
```

## Layout

Atomic-design-ish, feature-sliced: each domain (`gate`, `messages`, `contacts`, `settings`,
`theme`) owns its `application/` (hooks, one per Tauri command or command group), `organisms/`,
`pages/`, and a `connected/` component wiring hooks to presentation. `ui/` holds the
domain-agnostic atoms/molecules/templates shared across all of them.

## Capabilities

- Bluetooth device setup: lists paired devices, resolves MAP/PBAP RFCOMM channels over SDP,
  persists config — same flow the CLI's `imsg config setup` runs, driven instead by
  `GateStatus` polling
- 1:1 conversation threads, polled: list, read (last 200 messages per thread), send plain
  text, mark-read-on-open, delete whole conversation
- Per-message outbound delivery state (queued/sending/sent/delivered/failed/unknown)
- Contact-name sync via PBAP (manual trigger), surfaced as thread labels only
- Daemon lifecycle: stop, restart, install/uninstall as a user or system service, live
  service-install-state and MAP-session-state polling
- Bluetooth security-level (`BT_SECURITY` SDP/Low/Medium/High) and MAP/PBAP channel overrides
- Light/dark/system theme, persisted to `localStorage`

## Not yet supported

- Attachments/MMS media — text bodies only
- Group conversations — threads are single-address, 1:1 only
- Per-message delete or manual read/unread toggling — conversation-level delete only,
  read state is implicit on open
- In-app search/filtering — fixed most-recent-200-message window per thread, no CLI-parity
  `--from`/`--since`/folder controls
- Hub/spoke (remote Bluetooth adapter) setup — `hub_node_key` rides along in `ConfigDto` but
  nothing in the frontend reads or writes it; CLI-only via `imsg spoke add`
- Contact photos — placeholder avatar icon regardless of synced contact data

## Development

```sh
npm ci
npm run dev          # vite dev server against a running tauri backend
npm run typecheck    # tsc -b
npm run lint         # oxlint
npm run build        # tsc -b && vite build
```

Storybook (`npm run storybook`) is available locally for atom/molecule work but its config and
stories are gitignored — not part of the tracked tree.
