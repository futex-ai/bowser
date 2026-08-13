## Library API

The CLI is a thin wrapper around the `bowser` library crate. The library exposes the same capabilities programmatically.

### Crate Structure

```
bowser/
├── Cargo.toml              # workspace root
├── crates/
│   ├── bowser/              # library crate (core logic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── browser/         # browser lifecycle, pages, and checkpoint orchestration
│   │       ├── page/            # page interaction (navigate, click, type, etc.)
│   │       ├── capture/         # DOM → YAML transformation
│   │       ├── checkpoint/      # portable checkpoint types and atomic JSON I/O
│   │       ├── keyboard.rs      # browser key normalization and dispatch helpers
│   │       ├── mouse.rs         # pointer-route helpers for visible clicks
│   │       ├── stealth.rs       # stealth patch injection
│   │       ├── stability.rs     # page stability detection
│   │       ├── yaml/            # YAML serialization
│   │       ├── ai/              # AI image summarization client
│   │       ├── config/          # configuration types
│   │       ├── session/         # detached session lifecycle and metadata
│   │       └── error.rs         # error types
│   └── bowser-cli/          # binary crate (CLI)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── dispatch.rs      # one-shot dispatch and envelope ownership
│           ├── output.rs        # human/machine result and deferred file output
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── expand.rs    # expand command
│           │   ├── get.rs       # single-shot command
│           │   ├── interactive.rs  # REPL command
│           │   ├── describe.rs  # explicit image-description command
│           │   ├── meta.rs      # metadata command
│           │   ├── page.rs      # page management commands
│           │   ├── repl.rs      # REPL parsing and completions
│           │   ├── repl_help.rs # REPL help text
│           │   └── session.rs   # session management commands
│           └── error.rs
```
