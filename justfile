default: build

build:
    cargo build --workspace --all-targets --all-features

check:
    cargo check --workspace --all-targets --all-features

test:
    cargo test --workspace --all-features

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

pre-commit: fmt check lint

pre-push: fmt-check check lint test

ci: fmt-check check lint test build

release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(git cliff --bumped-version | sed 's/^v//')
    cargo release --workspace "${VERSION}" --execute

ship: ci release
    git push --follow-tags
