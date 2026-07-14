//! Generates `bindings.ts` for the frontend from the real `#[tauri::command]` surface.
//!
//! `Builder::export` only reads the registered command signatures — no running app/window
//! needed, so this works without `tauri.conf.json`/`build.rs`/icons/`main.rs` existing yet.
//! Run via `cargo run --example export_bindings -p imsg-gui`; writes
//! `crates/imsg-gui/frontend/src/bindings.ts` for direct import by the frontend.

use specta_typescript::Typescript;

fn main() -> anyhow::Result<()> {
    let builder = imsg_gui::commands::builder::<tauri::Wry>();
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/frontend/src/bindings.ts");
    builder.export(Typescript::default(), out)?;
    println!("wrote {out}");
    Ok(())
}
