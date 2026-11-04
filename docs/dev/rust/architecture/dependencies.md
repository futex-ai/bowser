# Rust Workspace Boundaries

The `rust` skill of `futex-ai/skills` holds the general Rust rules: trait
boundaries, `dyn` dispatch, `unimock` mocks, the test layout, and where to
declare each dependency.

- `bowser` owns the reusable browser, page, session-store, and AI-provider
  contracts and their default implementations.
- `bowser-cli` owns command parsing, terminal output, and interactive
  orchestration. It consumes the library rather than duplicating browser
  behavior.
- `xtask` owns repository verification and review orchestration. Product
  code must not depend on it.
