# Rust Dependencies And Trait Boundaries

Bowser treats trait boundaries as the default shape for non-pure Rust behavior,
including browser control, persistence, network access, subprocesses, clocks,
and hidden mutable runtime state.

## Core Rules

1. Keep only small pure logic as free functions or inherent methods on value
   types.
2. Put behavior with dependencies, ambient state, side effects, or runtime
   orchestration behind a trait.
3. Make consumers depend on `dyn Trait`, normally through
   `Arc<dyn Trait + Send + Sync>`.
4. Keep concrete construction at the binary or composition edge.
5. Use crate-local traits for local seams. Extract an interface crate only when
   several crates need to share the contract.
6. Factory traits for impure behavior return trait objects rather than concrete
   runtime types.

## Workspace Boundaries

- `bowser` owns the reusable browser, page, session-store, and AI-provider
  contracts and their default implementations.
- `bowser-cli` owns command parsing, terminal output, and interactive
  orchestration. It consumes the library rather than duplicating browser
  behavior.
- `xtask` owns repository verification and review orchestration. Product code
  must not depend on it.

External dependencies belong in the package that owns the behavior using them.
The root workspace manifest owns only shared local package relationships.

## Testing Rules

1. Unit tests mock owned impure seams with `unimock`.
2. Integration and smoke tests may use real Chrome, local HTTP servers,
   subprocesses, and files when end-to-end behavior is the purpose of the test.
3. Source-adjacent unit tests live under `src/_tests_` and are declared with
   an explicit `#[path = "_tests_/..."]` module.
4. Crate-root integration tests remain under `tests/`.
5. Source-adjacent tests must not introduce handwritten trait doubles.

## Audit And Enforcement

```bash
cargo xtask rust-source-audit
cargo xtask rust-trait-audit
cargo xtask rust-file-length-lint --all
```

These audits are part of `cargo xtask check`. They reject unreachable Rust
source files, handwritten source-adjacent trait doubles, invalid test layout,
and Rust files over the repository's 300-line limit.
