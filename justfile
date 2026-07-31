default: build

# imsg-gui needs tauri's GTK/WebKit dev headers, which this toolchain doesn't provision —
# excluded from these workspace-wide recipes; build/test/lint it directly via `-p imsg-gui`.
build:
    cargo build --workspace --exclude imsg-gui --all-targets --all-features

check:
    cargo check --workspace --exclude imsg-gui --all-targets --all-features

test:
    cargo test --workspace --exclude imsg-gui --all-features

# Only runs tests for crates touched by uncommitted changes (plus their dependents), via
# nextest. Requires `cargo-test-changed` and `cargo-nextest` on PATH (`cargo install
# --locked cargo-test-changed cargo-nextest`). Unlike the recipes above, this has no
# `--exclude imsg-gui` equivalent (the tool doesn't support one) — if imsg-gui files are
# among the changes, expect it to attempt building imsg-gui too.
test-changed:
    cargo test-changed -r nextest -d -- --all-features

lint:
    cargo clippy --workspace --exclude imsg-gui --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

docs:
    cargo run --example gen-cli-docs -p imsg

gen-completions:
    cargo run --example gen-completions -p imsg

# crates/imsg-gui/frontend — kept separate from the Rust recipes above; no GTK/WebKit
# headers needed for these, just Node.
gui-typecheck:
    cd crates/imsg-gui/frontend && npm run typecheck

gui-lint:
    cd crates/imsg-gui/frontend && npm run lint

gui-build:
    cd crates/imsg-gui/frontend && npm run build

gui-ci: gui-typecheck gui-lint gui-build

# WebDriver test build (`@wdio/tauri-service`, embedded provider): the one build where the
# Rust `webdriver` feature, the frontend `VITE_WEBDRIVER` flag, and `withGlobalTauri` (via
# `TAURI_CONFIG`, not the base `tauri.conf.json` — see main.rs/main.tsx) all need to line up.
# Neither plugin nor `withGlobalTauri` is present in a plain `gui-build`/`cargo build -p
# imsg-gui`, debug or release.
gui-webdriver-build:
    cd crates/imsg-gui/frontend && VITE_WEBDRIVER=true npm run build
    TAURI_CONFIG='{"app":{"withGlobalTauri":true}}' cargo build -p imsg-gui --features webdriver

pre-commit: fmt check lint

pre-push: fmt-check check lint test

ci: fmt-check check lint test build

release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(git cliff --bumped-version | sed 's/^v//')
    cargo release --workspace "${VERSION}" --execute

ship: ci release
    git push origin HEAD --follow-tags
