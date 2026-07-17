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
│   │       ├── browser.rs       # browser lifecycle (launch, resume, detach, close)
│   │       ├── page.rs          # page interaction (navigate, click, type, etc.)
│   │       ├── capture.rs       # DOM → YAML transformation
│   │       ├── keyboard.rs      # browser key normalization and dispatch helpers
│   │       ├── mouse.rs         # pointer-route helpers for visible clicks
│   │       ├── stealth.rs       # stealth patch injection
│   │       ├── stability.rs     # page stability detection
│   │       ├── yaml.rs          # YAML serialization
│   │       ├── ai.rs            # AI image summarization client
│   │       ├── config.rs        # configuration types
│   │       ├── session.rs       # detached session lifecycle and metadata
│   │       ├── expand.rs        # truncated node expansion handling
│   │       ├── metadata.rs      # metadata lookup helpers
│   │       └── error.rs         # error types
│   └── bowser-cli/          # binary crate (CLI)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
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
