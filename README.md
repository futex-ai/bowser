# Bowser

Bowser is a Rust library and command-line tool for controlling Chrome, capturing
rendered pages as compact structured YAML or JSON, and interacting with
resumable multi-page browser sessions.

## Features

- Launch or resume Chrome sessions and keep them available between commands
- Capture rendered semantic content without CSS and layout noise
- Navigate, click, type, submit, scroll, screenshot, and download through Chrome
- Run one-shot back, forward, and reload actions with typed history exhaustion
- Address captured elements with stable IDs within each capture
- Manage several live pages in one detached session
- Read live tab URLs and titles without changing page focus or activation
- Export portable cookie/localStorage/tab checkpoints and restore them into fresh sessions
- Emit a versioned one-object JSON command envelope for platform integrations
- Run an interactive REPL with command completion and resumable state
- Optionally describe captured images with Anthropic, OpenAI, or Ollama

## Requirements

- Rust 1.89 or newer
- Google Chrome or Chromium
- Xvfb on Linux when headed mode is used without an existing display
- Metacity for the fixed-display kiosk integration test
- `actionlint` and ShellCheck when running the full development check locally

Bowser discovers Chrome from the normal executable locations. Set
`BOWSER_CHROME_PATH` or pass `--chrome-path` when Chrome is installed
elsewhere.

## Install

Build the CLI from this workspace:

```bash
cargo build --release --package bowser-cli
install target/release/bowser ~/.local/bin/bowser
```

Tagged releases also publish archives for Linux and macOS on both x86-64 and
ARM64.

## CLI

Capture a page:

```bash
bowser https://example.com
```

Start an interactive session:

```bash
bowser --no-ai -i https://example.com
```

Resume a session and manage its pages:

```bash
bowser page list --session <SESSION_ID>
bowser capture --session <SESSION_ID>
bowser back --session <SESSION_ID>
bowser session info <SESSION_ID> --json-envelope
bowser session close <SESSION_ID>
```

Export and restore portable login state, or inspect the installed integration
contract:

```bash
bowser session export --session <SESSION_ID> --to checkpoint.json
bowser session restore --from checkpoint.json
bowser --json-envelope capabilities
```

Resumed capture, interaction, and page-management commands re-detach before
reporting operation, rendering, or output failures, so persisted page state is
left ready for the next command.

Run `bowser --help` for the full command surface. The
[CLI crate README](crates/bowser-cli/README.md) documents commands and manual
smoke paths in more detail.

## Library

```rust
use bowser::{Browser, BrowserConfig, BrowserEngine, PageEngine, to_yaml};

# async fn example() -> bowser::Result<()> {
let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.current_page().await?;
page.navigate("https://example.com").await?;
let capture = page.capture().await?;
println!("{}", to_yaml(&capture)?);
# Ok(())
# }
```

See the [library crate README](crates/bowser/README.md) for the public surface
and integration boundary.

## Configuration

Bowser reads `~/.config/bowser/config.yaml` by default. Command-line flags and
`BOWSER_*` environment variables override file settings. The
[configuration protocol](docs/protocol/bowser/implementation-and-config.md)
defines the supported keys and precedence.
Caller Chrome arguments are appended after Bowser's stealth-managed arguments;
Chrome's last-value behavior therefore applies, except
`--remote-debugging-port` and `--user-data-dir`, which Bowser rejects because
they define session identity and ownership.
Caller `--kiosk` also suppresses Bowser's window-mode and geometry defaults.
Desktop Chrome still leaves Ctrl+T, Ctrl+W, and Ctrl+N active, so
`features.kiosk` remains false; the
[kiosk protocol](docs/protocol/bowser/kiosk-history-and-live-inventory.md)
records the supported launch behavior and accelerator matrix.

## Development

```bash
cargo xtask check
```

The check command runs release-script tests, workflow linting, Rust source and
layout audits, formatting, clippy, unit tests, doctests, browser-backed
integration tests, and a CLI smoke test. Browser-backed tests use a real local
Chrome installation and run serially.

Focused commands are available when iterating:

```bash
cargo xtask check --include rust-clippy
cargo xtask check --include bowser
cargo xtask rust-file-length-lint --all
```

Pull requests and main-branch pushes run the same full check in GitHub Actions.
A `v<workspace-version>` tag runs the release workflow and publishes
checksummed CLI archives. Release verification records the exact checked-out
commit, and every platform build uses that verified commit rather than
resolving the tag again.

## Key Code

- `crates/bowser/src/` — browser lifecycle, capture model, sessions, and API
- `crates/bowser-cli/src/` — CLI parsing, dispatch, and interactive REPL
- `xtask/src/` — local and CI verification plus read-only AI review
- `scripts/release.sh` — release tag validation and portable asset packaging

## Documentation

- [Bowser protocol](docs/protocol/bowser/README.md)
- [Rust dependency and testing boundaries](docs/dev/rust/architecture/dependencies.md)
- [Workspace automation](xtask/README.md)
- [Plans index](plans/README.md)
