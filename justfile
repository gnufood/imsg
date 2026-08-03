default: build

# imsg-gui needs tauri's GTK/WebKit dev headers, which this toolchain doesn't provision —
# excluded from these workspace-wide recipes; build/test/lint it directly via `-p imsg-gui`.
build:
    cargo build --workspace --exclude imsg-gui --all-targets --all-features

check:
    cargo check --workspace --exclude imsg-gui --all-targets --all-features

test:
    cargo test --workspace --exclude imsg-gui --all-features

lint:
    cargo clippy --workspace --exclude imsg-gui --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

docs:
    cargo run --example gen-readme -p imsg

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
