default: build

# imsg-gui needs tauri's GTK/WebKit dev headers, which this toolchain doesn't provision —
# excluded from these workspace-wide recipes; build/test/lint it directly via `-p imsg-gui`
# (see gui-cargo-check/gui-cargo-lint/gui-cargo-test below).
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

metainfo-release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    FILE=crates/imsg-gui/packaging/foo.gnu.imsg.metainfo.xml
    VERSION=$(echo '{{VERSION}}' | sed 's/^v//')
    if grep -q "<release version=\"${VERSION}\"" "${FILE}"; then
        exit 0
    fi
    ENTRY=$(mktemp)
    trap 'rm -f "${ENTRY}"' EXIT
    {
        printf '    <release version="%s" date="%s">\n' "${VERSION}" "$(date +%F)"
        printf '      <url>https://github.com/gnufood/imsg/releases/tag/v%s</url>\n' "${VERSION}"
        printf '    </release>\n'
    } > "${ENTRY}"
    sed -i "/^  <releases>$/r ${ENTRY}" "${FILE}"

# The Rust side of imsg-gui itself — not covered by the workspace-wide check/lint/test above.
gui-cargo-check:
    cargo check -p imsg-gui --all-targets --all-features

gui-cargo-lint:
    cargo clippy -p imsg-gui --all-targets --all-features -- -D warnings

# nextest, not `cargo test` — imsg-gui's suite is known-flaky under `cargo test`'s in-process
# threaded runner; nextest's per-test-process isolation (plus the crate's own `#[serial]`
# markers via `serial_test`) is what makes it safe to run. Requires `cargo-nextest` on PATH.
gui-cargo-test:
    cargo nextest run -p imsg-gui --all-features

# crates/imsg-gui/frontend — kept separate from the Rust recipes above; no GTK/WebKit
# headers needed for these, just Node.
gui-typecheck:
    cd crates/imsg-gui/frontend && npm run typecheck

gui-lint:
    cd crates/imsg-gui/frontend && npm run lint

gui-build:
    cd crates/imsg-gui/frontend && npm run build

gui-ci: gui-cargo-check gui-cargo-lint gui-cargo-test gui-typecheck gui-lint gui-build

# Mirrors pre-commit/pre-push above but scoped to imsg-gui — skips gui-cargo-test (slow,
# and pre-commit stays fast on the Rust side too) and gui-build (neither Rust pre-commit
# nor pre-push builds; that's reserved for `ci`/`gui-ci`).
gui-pre-commit: gui-cargo-check gui-cargo-lint gui-typecheck gui-lint

gui-pre-push: gui-cargo-check gui-cargo-lint gui-cargo-test gui-typecheck gui-lint

# WebDriver test build (`@wdio/tauri-service`, embedded provider): the one build where the
# Rust `webdriver` feature, the frontend `VITE_WEBDRIVER` flag, and `withGlobalTauri` (via
# `TAURI_CONFIG`, not the base `tauri.conf.json` — see main.rs/main.tsx) all need to line up.
# Neither plugin nor `withGlobalTauri` is present in a plain `gui-build`/`cargo build -p
# imsg-gui`, debug or release.
gui-webdriver-build:
    cd crates/imsg-gui/frontend && VITE_WEBDRIVER=true npm run build
    TAURI_CONFIG='{"app":{"withGlobalTauri":true}}' cargo build -p imsg-gui --features webdriver

# Local sanity check for the bundled .deb/.rpm/.AppImage before trusting CI with it. Requires
# `cargo-tauri` on PATH (`cargo install tauri-cli --version 2.11.4 --locked`, matching the
# `tauri-cli` dev-dependency pin) — the CI release job installs its own copy separately.
gui-package:
    cd crates/imsg-gui && cargo tauri build

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
