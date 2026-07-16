# xtask

Bowser's internal workspace automation binary keeps local verification, CI, and
read-only AI review on one command surface. Developers should use it when
changing either Bowser crate or repository tooling.

## Responsibilities

- Define the local equivalent of pull-request and main-branch CI
- Keep browser-backed tests serial and isolated from ambient headed settings
- Enforce Rust source layout, file length, and trait-test boundaries
- Lint every GitHub Actions workflow
- Run a guarded, read-only Codex review against `origin/main`

## What This Crate Does

`cargo xtask check` runs release helper tests, workflow linting, Rust audits,
formatting, clippy, `xtask` unit tests, Bowser unit and doctests, every
browser-backed integration target, CLI integration tests, and a real
`bowser --help` smoke command.

Browser-running commands set `BOWSER_HEADLESS=true` and disable the two
headed-launch stealth experiments. Pure library and CLI unit tests run without
those overrides so they continue to assert production defaults.

`cargo xtask review` gathers the branch diff from the merge base with
`origin/main`, staged and unstaged changes, and guarded untracked text diffs.
It runs an ephemeral Codex process from a neutral temporary directory with a
read-only sandbox and a scrubbed environment. Sensitive paths are reported but
not opened or included in diff bodies. Tracked and untracked config/data paths
with secret, password, credential, or token components use the same
conservative omission boundary; ordinary Rust source filenames remain
reviewable.

## Quick Start

```bash
cargo xtask check
cargo xtask check --include bowser
cargo xtask review
```

Available check phases are `scripts`, `workflow`, `rust-audit`,
`rust-fmt`, `rust-clippy`, `rust-test`, and `bowser`. Repeat
`--include` or `--exclude`, or pass comma-separated values.
The source-adjacent test-layout validation belongs to `rust-audit`, so a focused
phase does not fail on an audit the caller did not select; an unfiltered full
check still runs it.

## Development

```bash
cargo test --package xtask
cargo clippy --package xtask --all-targets -- -D warnings
cargo xtask rust-source-audit
cargo xtask rust-trait-audit
cargo xtask rust-file-length-lint --all
```

Source and test-layout walkers skip symlinks instead of traversing outside the
repository. The trait audit parses Rust syntax, so generic and multiline trait
implementations in source-adjacent tests cannot bypass the Unimock rule.

The full check expects Rust with `rustfmt` and `clippy`, Python 3, `actionlint`,
ShellCheck, and a local Chrome or Chromium executable. Browser integration tests
run one test thread at a time.

The review command additionally requires an installed and authenticated Codex
CLI. It prints findings to stdout and does not change repository files.

### Key Code

- `src/main.rs` — command parsing and dispatch
- `src/check_plan.rs` — repository-wide verification phases
- `src/check_plan_bowser.rs` — browser and CLI test matrix
- `src/review.rs` — guarded review context collection
- `src/rust_source_audit.rs` — Cargo module graph audit

### Related Docs

- [Workspace README](../README.md)
- [Bowser protocol](../docs/protocol/bowser/README.md)
- [Rust dependency boundaries](../docs/dev/rust/architecture/dependencies.md)
