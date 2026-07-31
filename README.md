`imsg` connects to a paired iPhone using the standard Bluetooth MAP (Message Access Profile)
and PBAP (Phone Book Access Profile) protocols. No iCloud credentials, no Apple Silicon, no macOS bridge.

---

## Requirements

- Linux with BlueZ (`bluetoothd` running)

> 📎 That's it!

## Install

```sh
curl -sSfL https://releases.gnu.foo/imsg/latest/install.sh | sh
```

Or via cargo:

```sh
cargo install imsg
```

### Shell completions

Interactively, no download needed (bash/zsh/fish only — installs to the shell's conventional
completion directory after a confirmation prompt):

```sh
imsg completions zsh --install
```

Or print the script and redirect it yourself (works for every shell, including elvish/powershell):

```sh
imsg completions zsh > ~/.zfunc/_imsg
```

Or fetch the prebuilt static file straight from the latest release:

```sh
# bash — requires the `bash-completion` package
curl -sSfL https://releases.gnu.foo/imsg/latest/imsg.bash -o ~/.local/share/bash-completion/completions/imsg

# zsh — add `fpath+=(~/.zfunc)` before `compinit` in ~/.zshrc if not already
curl -sSfL https://releases.gnu.foo/imsg/latest/_imsg -o ~/.zfunc/_imsg

# fish — auto-loaded, no rc edit needed
curl -sSfL https://releases.gnu.foo/imsg/latest/imsg.fish -o ~/.config/fish/completions/imsg.fish
```

---

## Quick start

> [!NOTE]
> **Before you begin:** your iPhone must be paired to this machine via `bluetoothctl`, and
> `imsg` needs the RFCOMM channel numbers iOS has assigned to its MAP and PBAP services —
> these are dynamic and change across iOS versions and re-pairings so they can't be hardcoded.
>
> Run `setup` to discover them automatically from any paired device:
>
> ```sh
> curl -sSfL "https://releases.gnu.foo/imsg/latest/setup-$(uname -m)" -o setup && chmod +x setup && ./setup
> ```
>
> Feed the printed channel numbers into step 1 below.

### 1. Configure imsg

```sh
imsg config set-device A1:B2:C3:D4:E5:F6
```

If your channel numbers differ from the defaults (map=2, pbap=13), set them in
`~/.config/imsg/imsg.toml`:

```toml
[device]
address      = "A1:B2:C3:D4:E5:F6"
map_channel  = 2
pbap_channel = 13
```

### 2. Read and send messages

```sh
imsg list                                     # inbox
imsg list --unread                            # unread only
imsg list sent                                # sent folder
imsg list --from +15550001234 --limit 20      # filter by sender
imsg list --since 20260601T000000 --long      # since a date, show MAP handles
imsg get <handle>                             # full message body
imsg get <handle> --mark-read                 # fetch and mark as read
imsg send +15550001234 "hey"                  # send a message
imsg threads                                  # conversations grouped by contact
```

---

## Commands

See [`docs/cli.md`](docs/cli.md) for the full command-line reference, or run `imsg <command> --help` for command-specific flags.

---

## Hub / spoke (remote Bluetooth adapter)

If your iPhone is paired to a different machine — a Raspberry Pi, a server, a desktop in
another room — you can run `imsg` from any machine on the internet without re-pairing.

**On the machine with the paired phone:**

```sh
imsg hub
# prints: node key: <KEY>
```

**On your laptop (or anywhere else):**

```sh
imsg spoke add <KEY>
imsg --hub list
imsg --hub send +15550001234 "hello from anywhere"
```

The hub and spoke connect over QUIC via [iroh](https://iroh.computer/) — no port
forwarding or VPN required. 

---

## Daemon (background sync)

`imsg daemon start` runs a persistent background broker that keeps the MAP session connected
to your paired phone and the local store fresh — replaces the old `imsg watch` command.

```sh
imsg daemon start              # detaches into the background; idempotent if already running
imsg daemon status              # daemon for A1:B2:C3:D4:E5:F6: connected
imsg daemon stop                # graceful stop; no-op if nothing is running
```

Run under a process supervisor (e.g. systemd) with `--foreground` instead, or let `imsg`
register one for you:

```sh
imsg daemon install             # user-level systemd/launchd/... unit
imsg daemon install --system    # system-wide (requires elevated privileges)
imsg daemon uninstall [--system]
```

---

## Configuration

Config is layered in ascending priority:

```
compiled-in defaults
/etc/imsg.toml
~/.config/imsg/imsg.toml
./imsg.toml
--config <path>
IMSG_ environment variables   (e.g. IMSG_DEVICE__MAP_CHANNEL=15)
```

Full key reference:

```toml
[device]
address      = "A1:B2:C3:D4:E5:F6"   # required; set via `imsg config set-device`
map_channel  = 2                       # RFCOMM channel for MAP MAS  [1–30], default 2
pbap_channel = 13                      # RFCOMM channel for PBAP PSE [1–30], default 13

[hub]
node_key = "..."                       # set via `imsg spoke add <KEY>`; absent until then
```

---

## Building from source

```sh
git clone https://github.com/gnufood/imsg
cd imsg
cargo build --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow.

---

## License

MIT — see [LICENSE](LICENSE).
