//! imsg GUI entry point — wiring only.
//!
//! Two launch modes, distinguished by argv (`classify_launch`): a normal GUI launch builds the
//! Tauri app and spawns the startup gate ([`imsg_gui::gate::run`]) once; a self-respawned
//! headless child (see `daemon::provision::ensure_running`) runs the daemon in-process and
//! never touches Tauri. The headless branch mirrors the CLI's `daemon start --foreground`
//! handler: load config and open the store together, right at the point of use.

use imsg_gui::daemon::provision::{self, HeadlessArgs, Launch};
use imsg_gui::gate::GateState;
use tauri::Manager as _;

fn main() -> anyhow::Result<()> {
    init_tracing()?;
    let args: Vec<String> = std::env::args().collect();
    match provision::classify_launch(&args)? {
        Launch::Headless(headless) => tauri::async_runtime::block_on(headless_main(headless)),
        Launch::Gui => gui_main(),
    }
}

/// Same subscriber setup as the CLI's `init_tracing`, minus its verbosity flag (the GUI has no
/// CLI surface) — `RUST_LOG` still applies, default `info`.
fn init_tracing() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).try_init().map_err(|e| anyhow::anyhow!(e))
}

/// The re-exec'd daemon child. Safe to `config::load` unconditionally: this branch is only ever
/// reached from `ensure_running`'s spawn, which the gate only performs after a device config is
/// guaranteed to exist.
async fn headless_main(args: HeadlessArgs) -> anyhow::Result<()> {
    let cfg = config::load(args.config_path)?;
    let store = imsg_gui::reads::open_store(&cfg).await?;
    provision::run_headless(cfg, Some(args.device), store).await?;
    Ok(())
}

fn gui_main() -> anyhow::Result<()> {
    let specta_builder = imsg_gui::commands::builder::<tauri::Wry>();
    tauri::Builder::default()
        .manage(GateState::default())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<GateState>();
                imsg_gui::gate::run(
                    &state,
                    None,
                    |cfg| Box::pin(imsg_gui::reads::open_store(cfg)),
                    // `manage` returns false only if a `Store` was already managed — impossible
                    // here, the gate runs once and is the only thing that manages one.
                    |store| {
                        handle.manage(store);
                    },
                )
                .await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
