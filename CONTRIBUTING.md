# Contributing

## Prerequisites

- Rust 1.89.0+ (`rust-toolchain.toml` pins the version automatically)
- `just` — `cargo install just`
- BlueZ (`bluetoothd`) for any live Bluetooth tests

## Development

```bash
just ci            # full check — mirrors CI exactly
just fmt           # auto-format
just fmt-check     # fmt --check only
just lint          # clippy with -D warnings
just test          # cargo test --workspace
just build         # build all targets and features
```

`just pre-commit` runs `fmt`, `check`, and `lint`. `just ci` must be green before every commit — never run `cargo -p <crate>` in isolation, it skips cross-crate issues.

## Coding standards

- **Keep functions and modules focused on a single responsibility.** When something is hard to name, hard to test, or hard to read in one pass — extract it.
- **`unsafe` is forbidden.** `unsafe_code = "forbid"` is enforced workspace-wide — no exceptions.
- **Errors propagate through typed channels.** Impossibilities are documented at the call site.
- **Prefer maintained crates over hand-rolled equivalents.** Burden of proof is on rolling your own.
- **Doc comments state guarantees not visible from the signature.** Restating identifier names is noise. Every `pub` item gets a `///` doc comment — no exceptions.

## Tests

Tests should demonstrate real behaviour from the caller's perspective — that the code does what it claims, under the conditions that matter.

- A behaviour change without a failing-then-passing test is incomplete.

## Pull requests

- One logical change per PR
- Conventional Commits (`feat:` `fix:` `refactor:` `perf:` `ci:` `chore:` `docs:`)
- Add an entry under `[Unreleased]` in `CHANGELOG.md`, written from the consumer's perspective — what changed for them, not how
- Open an issue first for anything beyond a small fix

## Bugs

Open a GitHub issue or email `bugs@gnu.foo`.

For security vulnerabilities, see [SECURITY.md](SECURITY.md).
