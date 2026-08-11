# Getting Started Walkthrough

This walkthrough traces the end-to-end flow of a typical operation: configuring the device, performing an initial sync to populate the local store, and executing basic message operations. The flow illustrates how the CLI routes commands through the broker, manages transport selection, and coordinates with the local encrypted store.

## Operation Context

The system supports two transport paths for communicating with a paired phone:

- **RFCOMM path**: Commands are forwarded to an ephemeral broker subprocess that manages the Bluetooth connection. The broker is auto-spawned on first use.
- **Hub path**: When `--hub` is passed, the CLI binds an iroh spoke endpoint and connects directly to a remote hub, bypassing local Bluetooth.

Most commands also support an **opt-in store layer**: after an initial `sync`, read operations can be served from a local encrypted SQLite database rather than the device, enabling offline access and faster queries.

## Initial Setup Flow

### Phase 1: Device Configuration

The user begins with no configured device. Invoking any command that requires a device connection first triggers config validation in `config::load()`, which applies the layered configuration strategy defined in `imsg-config/src/lib.rs`:

```rust
// Layered config: defaults → /etc/imsg.toml → XDG user config → ./imsg.toml → explicit path → env
pub fn load(explicit: Option<PathBuf>) -> Result<Config, ConfigError> {
    let cfg: Config = figment(explicit).extract()?;
    validate(&cfg)?;
    Ok(cfg)
}
```

The config layers are (in ascending priority):

1. Compiled-in defaults: MAP channel 2, PBAP channel 13, broker timing parameters
2. `/etc/imsg.toml` (system-wide)
3. `~/.config/imsg/imsg.toml` (user-specific)
4. `./imsg.toml` (project-local)
5. Explicit `--config` path
6. `IMSG_` environment variables with `__` as nesting separator

If `device.address` is absent, `config::load` returns a `ConfigError::Load` variant, and the CLI surfaces a helpful message directing the user to run `imsg config set-device <ADDR>`.

### Phase 2: Interactive Setup

The recommended setup path is `imsg config setup`, implemented in `commands/config.rs`. This command:

1. **Lists paired Bluetooth devices** via `transport::discover::list_paired_devices()`, which uses the BlueR stack to query the Bluetooth adapter
2. **Prompts for selection** using `dialoguer::Select`, presenting device names and addresses
3. **Resolves RFCOMM channels** via Service Discovery Protocol (SDP): `transport::discover::resolve_channels(&address)` queries the device for its MAP and PBAP service records
4. **Persists the configuration** by writing the address and both channels to the user config file via `config::set_device_and_channels()`

```rust
pub(crate) async fn run_setup() -> Result<String> {
    let devices = transport::discover::list_paired_devices().await
        .context("listing paired devices")?;
    // ... prompt for device selection ...
    let channels = transport::discover::resolve_channels(&address.to_string())
        .await
        .context("resolving MAP/PBAP channels over SDP")?;
    // ... validate both channels present ...
    ::config::set_device_and_channels(&address.to_string(), map, pbap)?;
    // ...
}
```

If the device lacks either MAP or PBAP service records, the command fails with a diagnostic message rather than persisting incomplete configuration.

## Initial Sync Flow

### Phase 1: Command Dispatch

After setup, the user runs `imsg sync` to backfill the local store. The flow begins in `main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbosity)?;
    commands::dispatch(args).await
}
```

The `dispatch` function in `commands/mod.rs` receives the parsed `Cli` struct. It checks for the `--hub` flag; if present, it binds an iroh spoke endpoint:

```rust
pub async fn dispatch(cli: Cli) -> Result<()> {
    let Cli { hub, device, config: config_path, command, .. } = cli;
    let spoke = if hub {
        Some(transport::iroh::bind_spoke().await.context("binding iroh spoke endpoint")?)
    } else {
        None
    };
    let result = run_command(command, spoke.as_ref(), device.as_deref(), config_path).await;
    // ... close spoke endpoint on exit ...
}
```

For `sync`, the command handler in `commands/sync.rs` is invoked:

```rust
pub(crate) async fn run(
    cfg: &Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    store: &Store,
    folder: Option<FolderArg>,
    config_path: Option<&Path>,
) -> Result<String> {
    let folder_scope = folder.map(folder_arg_to_folder);

    if spoke.is_none() {
        // RFCOMM path: delegate to broker
        let folder_name = folder_scope.map(|f| f.as_str().to_owned());
        let text = broker::sync(cfg, device, config_path, folder_name).await?;
        let contacts = contacts::run_sync(cfg, spoke, device, store, config_path).await?;
        store.set_meta("sync_enabled", "true").await?;
        return Ok(format!("{text}; {contacts}"));
    }

    // Hub path: direct MAP connection
    let mut client = conn::connect_map(cfg, spoke, device).await?;
    let now = session::util::now_ms();
    session::outbox::drain_outbox(&mut client, store, now).await?;
    session::sync::backfill(&mut client, store, folder_scope).await?;
    let contacts = contacts::run_sync(cfg, spoke, device, store, config_path).await?;
    store.set_meta("sync_enabled", "true").await?;
    Ok(format!("sync complete; {contacts}"))
}
```

### Phase 2: Broker Auto-Spawn (RFCOMM Path)

When not using `--hub`, the RFCOMM path delegates to the broker. The broker module in `commands/broker/mod.rs` handles auto-spawning:

```rust
pub(crate) async fn call(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    req: BrokerRequest,
) -> Result<BrokerResponse> {
    let addr = resolve_addr(cfg, device);
    spawn::ensure_running(cfg, device, config_path).await?;
    send_request(addr, req).await
}
```

The `spawn::ensure_running` function checks whether a broker process is already running for the device (via a Unix abstract socket). If not, it forks an ephemeral broker subprocess that inherits the config path and device override, then waits for the socket to become ready.

### Phase 3: MAP Backfill

The broker (or direct MAP connection in hub mode) performs the backfill:

1. **Drains the outbox**: Any pending outgoing messages in the local store are pushed to the device via `session::outbox::drain_outbox()`. This ensures messages queued while offline are delivered.
2. **Backfills folders**: `session::sync::backfill()` iterates through each MAP folder (inbox, sent, outbox, deleted), fetching messages since the last cursor. Cursors are stored in the `meta` table of the SQLite store.
3. **Refreshes contacts**: `contacts::run_sync()` pulls the full PBAP phonebook and stores it in the contacts cache.
4. **Marks sync complete**: On success, `store.set_meta("sync_enabled", "true")` records that the user has opted into store-backed reads.

The store is an encrypted SQLite database opened in `commands/mod.rs`:

```rust
pub(in crate::commands) async fn open_store(cfg: &::config::Config) -> Result<store::Store> {
    let path = cfg.store.resolve()
        .context("no data directory available (set HOME or XDG_DATA_HOME)")?;
    let ready = keyring::init_store().context("Secret Service store init failed")?;
    let key = keyring::get_or_create_db_key(&ready).context("getting database encryption key")?;
    store::Store::open(path, key).await.context("opening message store")
}
```

The encryption key is retrieved from the system keyring (Secret Service API on Linux), creating one if absent.

## Message Operations After Sync

### Opt-In Dispatch

After sync, the system enters **opted-in** mode. Read operations now route to the local store rather than the device, unless the user explicitly passes `--hub` or the store lacks the requested data.

The dispatch logic in `commands/dispatch.rs` checks the opt-in state before each operation:

```rust
pub(in crate::commands) async fn run_list(
    cfg: &::config::Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    mut opts: list::ListOpts,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    opts.from = opts.from.map(|f| canonical_number(&f));
    if is_opted_in(db, "sync_enabled").await {
        with_spinner("listing", list::run_store(opts, db)).await
    } else {
        with_spinner("listing", list::run(cfg, spoke, device, opts, config_path)).await
    }
}
```

The `is_opted_in` helper queries the `meta` table:

```rust
async fn is_opted_in(store: &store::Store, key: &str) -> bool {
    match store.get_meta(key).await {
        Ok(v) => v.as_deref() == Some("true"),
        Err(e) => {
            tracing::warn!("failed to read {key} from store, falling back to phone: {e}");
            false
        }
    }
}
```

### List Operation (Store Path)

When opted in, `imsg list` reads from the local store via `list::run_store()`:

```rust
pub(crate) async fn run_store(opts: ListOpts, store: &Store) -> Result<String> {
    let folder_str = folder_of(opts.folder).as_str();
    let since_ms = opts.since.as_deref().and_then(session::sync::datetime_to_ms);
    let rows = store
        .list_messages(
            Some(folder_str),
            opts.unread,
            opts.from.as_deref(),
            since_ms,
            opts.limit.unwrap_or(1024),
            opts.offset.unwrap_or(0),
        )
        .await?;
    // ... render rows ...
    out.push_str(&crate::commands::freshness_line(store.latest_sync_at().await?));
    Ok(out)
}
```

The output includes a freshness footer showing when the store was last synced, prompting the user to run `imsg sync` to refresh.

### Get Operation (Store Path)

`imsg get <handle>` retrieves a single message body:

```rust
pub(crate) async fn run_store(handle: String, mark_read: bool, store: &Store) -> Result<String> {
    let row = store
        .get_by_handle(&handle)
        .await?
        .ok_or_else(|| anyhow::anyhow!("message {handle} not found in local store"))?;
    if mark_read && row.status == STATUS_UNREAD {
        store.update_status(&handle, STATUS_READ).await?;
    }
    // ... render body with freshness footer ...
}
```

If `mark_read` is true, the local status is updated immediately; device-side status is deferred to the next sync.

### Send Operation

`imsg send <number> <message>` supports two paths depending on opt-in state:

**Opted-in path** (`run` in `send.rs`): The message is first recorded in the local outbox, then pushed to the device:

```rust
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    store: &Store,
    config_path: Option<&Path>,
) -> Result<String> {
    let now = session::util::now_ms();
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::outbox::send_sms(&mut client, store, &number, &message, now).await;
    // ... disconnect ...
    result
}
```

**Non-opted-in path** (`run_live`): The message is pushed directly to the device without store interaction:

```rust
pub(crate) async fn run_live(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    config_path: Option<&Path>,
) -> Result<String> {
    // ... direct push without store ...
}
```

The dispatch in `dispatch.rs` selects the appropriate path based on `is_opted_in(db, "sync_enabled")`.

## Contacts Operations

Contacts follow a similar pattern but with independent opt-in state (`contacts_synced` in the meta table). The `contacts` subcommand supports:

- `--list`: List handles and names without full vCards
- `--get <handle>`: Fetch a single contact
- `--lookup <number>`: Reverse-lookup by phone number
- `--sync`: Refresh the contacts cache from the device

The sync operation is always performed against the live device, regardless of opt-in state, because it is a write operation:

```rust
pub(in crate::commands) async fn run_contacts(
    // ...
) -> Result<String> {
    if opts.sync {
        let fut = contacts::run_sync(cfg, spoke, device, db, config_path);
        return with_spinner("syncing contacts", fut).await;
    }
    // ... read path with opt-in dispatch ...
}
```

## Failure Handling

Several failure conditions affect the overall flow:

- **Config load failure**: If `device.address` is unset, the CLI exits with a message suggesting `imsg config set-device <ADDR>`.
- **Broker spawn failure**: If the broker cannot be started (e.g., Bluetooth unavailable), the error propagates to the user.
- **Store open failure**: If the keyring is unavailable or the database is corrupted, the CLI falls back to live-device mode.
- **Opt-in state read failure**: If the store query fails, the system logs a warning and falls back to the live path, ensuring the command still works.

## Resulting State

After completing the setup and initial sync, the system is in the following state:

- **Config**: Device address and MAP/PBAP channels are persisted in the user config file
- **Store**: Encrypted SQLite database contains all messages from the device and a contacts cache
- **Opt-in flags**: `sync_enabled = "true"` and `contacts_synced` are set in the store meta table
- **Transport**: Subsequent read operations default to the store path; write operations go to the device and update the store

The user can now run `imsg list`, `imsg get`, `imsg send`, and `imsg contacts` without a device connection (for reads) or with faster response times (store-backed reads vs. live device queries).