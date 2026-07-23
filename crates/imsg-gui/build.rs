//! Runs Tauri's build-time codegen against `tauri.conf.json`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=capabilities");
    // Without this, Cargo has no dependency edge from this crate to `frontend/dist`'s
    // contents and won't recompile when it changes — `generate_context!` would keep
    // embedding whatever bundle was already linked, e.g. a stale non-webdriver `dist/`.
    println!("cargo:rerun-if-changed=frontend/dist");
    // `capabilities/webdriver/` references permissions from `tauri-plugin-wdio`/
    // `-webdriver`, which are only linked behind the `webdriver` feature — tauri-build
    // hard-errors ("permission not found") if a capability references a permission from a
    // plugin that isn't actually a compiled dependency, so the glob must exclude that
    // subdirectory unless the feature (and thus the plugins) are present.
    let pattern =
        if cfg!(feature = "webdriver") { "./capabilities/**/*" } else { "./capabilities/*" };
    tauri_build::try_build(tauri_build::Attributes::new().capabilities_path_pattern(pattern))?;
    Ok(())
}
