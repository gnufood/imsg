# CLI Commands Reference

The `imsg` command-line interface provides operations for iMessage access over Bluetooth MAP/PBAP. Commands are organized into message operations, contacts, configuration, hub/spoke networking, and daemon management.

## Global Options

| Option | Description |
|--------|-------------|
| `--hub` | Route MAP/PBAP connections through a remote iroh hub instead of local Bluetooth |
| `--device <ADDR>` | Override the configured device MAC address (RFCOMM only) |
| `--config <PATH>` | Use a specific config file instead of the layered default search |
| `-v`, `-vv` / `-q` | Increase or decrease log verbosity |

The global `--hub` flag enables spoke mode, connecting to a remote hub configured via `imsg spoke add`. The `--device` flag overrides the configured Bluetooth address for RFCOMM connections. The `--config` flag specifies an explicit config file path.

## Message Operations

### list

Lists messages in a folder, either live from the device or from the local store (if opted in).

```
imsg list [OPTIONS] [FOLDER]
```

**Arguments:**
- `FOLDER`: One of `inbox`, `sent`, `outbox`, `deleted` (default: inbox)

**Options:**
- `--unread` — Show only unread messages
- `--from <NUMBER>` — Filter by originating address
- `--since <DATE>` — Filter to messages at or after `YYYYMMDDTHHMMSS`
- `--limit <N>` — Maximum entries to return
- `--offset <N>` — Skip first N entries (pagination)
- `-l`, `--long` — Show the raw MAP handle for each message

When not opted in, output includes a `(live from device)` footer. When opted in, output includes a freshness timestamp and prompts to run `imsg sync`.

### get

Fetches a single message body by handle.

```
imsg get <HANDLE> [OPTIONS]
```

**Arguments:**
- `HANDLE`: The MAP message handle (from `list --long`)

**Options:**
- `--folder <FOLDER>` — Folder containing the message (default: inbox)
- `--mark-read` — Mark the message as read after fetching

### send

Sends an SMS to a phone number.

```
imsg send <NUMBER> <MESSAGE>
```

**Arguments:**
- `NUMBER`: Recipient phone number
- `MESSAGE`: Message body text

When opted in, the message is enqueued to the local outbox, pushed to the device, and delivery status is tracked. When not opted in, the message is pushed directly to the device without local tracking.

### delete

Marks a message as deleted (or restores it with `--undelete`).

```
imsg delete <HANDLE> [OPTIONS]
```

**Arguments:**
- `HANDLE`: The MAP message handle

**Options:**
- `--folder <FOLDER>` — Folder containing the message (default: inbox)
- `--undelete` — Restore a previously deleted message instead of deleting it

**Note:** iOS does not implement MAP undelete; the `--undelete` flag is accepted but performs no operation.

### threads

Groups inbox and sent messages into conversation threads.

```
imsg threads
```

Returns per-contact thread summaries sorted by most recent message. Output includes total message count, unread count, and latest timestamp. When not opted in, displays `(live from device)`. When opted in, displays a freshness timestamp.

### folders

Lists the MAP message folders available on the device.

```
imsg folders
```

Returns folder names from `telecom/msg`, preserving device order.

### sync

Backfills the local store with all messages from the device since the last sync.

```
imsg sync [OPTIONS]
```

**Options:**
- `--folder <FOLDER>` — Sync a single folder instead of all folders

Sync also refreshes the contacts cache. On success, sets `sync_enabled = "true"` in the store, enabling local-first reads for subsequent `list`/`get`/`threads` commands.

### unsync

Disables local-first reads, reverting to direct device access.

```
imsg unsync [OPTIONS]
```

**Options:**
- `--purge` — Delete the database file and all synced data in addition to disabling sync

The `--purge` flag removes the database file and its WAL/SHM companions. Without `--purge`, the database is preserved and can be re-enabled with `imsg sync`.

## Contacts Operations

### contacts

Pulls contacts from a phonebook, with options for listing, fetching, reverse-lookup, or syncing.

```
imsg contacts [OPTIONS]
```

**Options:**
- `--list` — List handles/UIDs and names only, without full vCards
- `--get <KEY>` — Fetch a single contact by PBAP handle (live) or cached UID (opted in)
- `--lookup <NUMBER>` — Reverse-lookup a contact by phone number
- `--sync` — Refresh the local contacts cache from the device
- `--path <PHONEBOOK>` — Phonebook to query (default: `pb`)
- `--raw` — Show phone numbers as stored, skip E.164 normalisation
- `--limit <N>` — Maximum contacts per page
- `--page <N>` — Page number (1-indexed, requires `--limit`)

**Phonebook values:**
- `pb` — Main phonebook (default)
- `ich` — Incoming call history
- `och` — Outgoing call history
- `mch` — Missed call history
- `cch` — Combined call history
- `spd` — Speed-dial entries
- `fav` — Favourites

The `--sync` flag always reads from the live device regardless of opt-in state. When opted in (`contacts_synced = "true"`), the default operation reads from the local cache.

## Configuration

### config

Inspects or modifies local configuration.

```
imsg config <SUBCOMMAND>
```

**Subcommands:**
- `show` — Print the resolved configuration
- `set-device <ADDRESS>` — Persist the device MAC address (`XX:XX:XX:XX:XX:XX`)
- `setup` — Interactively pick a paired device, resolve MAP/PBAP channels over SDP, and persist all three

The `setup` subcommand lists paired Bluetooth devices, prompts for selection, resolves MAP and PBAP RFCOMM channels via SDP, and writes the address and channels to the user config file. It fails if the device lacks either service record.

## Hub and Spoke (Remote Access)

### hub

Starts the iroh hub on the local machine and prints its node key.

```
imsg hub
```

The hub proxies MAP and PBAP RFCOMM streams from connected spokes to the paired Bluetooth device, and relays MNS notification events to iroh-connected subscribers. MNS relay failures degrade only that stream.

**Requirements:**
- Device address must be configured
- MAP and PBAP channels must be configured
- No other hub instance running (enforced via lock file)

### spoke

Manages spoke configuration for connecting to a remote hub.

```
imsg spoke add <KEY>
```

**Arguments:**
- `KEY`: The iroh hub node key (printed by `imsg hub`)

Validates the key syntactically as an iroh `EndpointId` and persists it to the user config. Semantic validity (hub reachability, correct pairing) is deferred to connect time.

## Broker and Daemon

### broker

Queries the session broker.

```
imsg broker status
```

Reports whether the broker is running and whether its MAP session is connected.

### daemon

Manages the persistent background broker (required for GUI use).

```
imsg daemon <SUBCOMMAND>
```

**Subcommands:**
- `start` — Start the persistent broker (detaches to background by default)
- `start --foreground` — Start in foreground (for process supervisors)
- `stop` — Request graceful shutdown
- `status` — Report daemon and MAP session status
- `install` — Register with OS service manager (systemd/launchd/OpenRC)
- `install --system` — Register system-wide (requires elevated privileges)
- `uninstall` — Unregister the daemon service
- `uninstall --system` — Unregister a system-wide service

The daemon auto-starts on boot/login when installed. The `--foreground` variant stays attached and responds to Ctrl-C, SIGTERM, or IPC shutdown requests.

## Shell Completions

### completions

Prints or installs shell completion scripts.

```
imsg completions <SHELL> [OPTIONS]
```

**Arguments:**
- `SHELL`: One of `bash`, `elvish`, `fish`, `powershell`, `zsh`

**Options:**
- `--install` — Write to the shell's conventional completion directory (not supported for elvish/powershell)

Without `--install`, the script is printed to stdout and should be redirected to the appropriate completion directory manually.

**Installation paths:**
- Bash: `~/.local/share/bash-completion/completions/imsg`
- Zsh: `~/.zfunc/_imsg`
- Fish: `~/.config/fish/completions/imsg.fish`

## Internal Commands

### `__broker_serve`

Internal command used by the CLI to spawn the session broker subprocess. Not for direct invocation.

## Data Flow

Commands operate in one of two modes:

1. **Live mode** (default): Reads directly from the device via MAP/PBAP, or through the broker for RFCOMM connections
2. **Store mode** (when opted in): Reads from the local encrypted SQLite store after `imsg sync`

The opt-in state is tracked independently for MAP messages (`sync_enabled`) and contacts (`contacts_synced`). The `dispatch.rs` module routes each command to the appropriate implementation based on these flags.

## Output Footers

- `(live from device)` — Data read directly from the device
- `(never synced — run 'imsg sync' to populate the store)` — Store empty, never synced
- `(store as of <timestamp> — run 'imsg sync' to refresh)` — Store data with last sync time