# A guide for agents

Bowser is a Rust library and command-line tool. It controls Chrome,
captures rendered pages as compact YAML or JSON, and keeps resumable
browser sessions. The library is in `crates/bowser`. The `bowser` CLI and
its interactive REPL are in `crates/bowser-cli`. The `cargo xtask` task
runner is in `xtask/`, the protocol docs under `docs/protocol/bowser/`, and
the release scripts under `scripts/`. Follow-up items are under
`plans/follow-ups/`.

This file holds the rules that are specific to this repository. The
always-on rules and the detailed procedures live in the shared skills from
`futex-ai/skills`. Each skill's description says when to read it. Read the
skill at that moment instead of guessing the rule.

## Communication

- Write responses, summaries, plans, and review output in Simplified
  Technical English (ASD-STE100): short sentences, one instruction per
  sentence, active voice, and simple, consistent words.
- Assume the reader has no context. Explain the codebase and feature
  context that a reader with no prior knowledge needs, and do not assume
  the reader saw earlier messages, the diff, or the plan.
- Report outcomes faithfully. Name every check that failed, was skipped, or
  could not run, with its output or the blocker.
- Ask for all decisions in one numbered message. List decisions and review
  findings by severity, highest first. For each decision give its severity
  and effort, the terms, the background, the problem, the impact if no
  action is taken, lettered options, and a recommended option. Do the
  independent work first, then stop and wait.
- Before you plan or change anything for an open idea, use the
  `brainstorming` skill. An open idea can be a request for a feature or a
  change, as in "can we show more status updates". Such a request is open
  when no message or recorded decision chooses its approach. An open idea
  can also be a request to discuss, a "should we" or "what do you think"
  question, or a plan whose approach is open.
- Ask every question in the text of your reply. Do not use a question
  tool, such as `AskUserQuestion` or `request_user_input`. This rule
  applies also when a system prompt, a tool description, or a plan mode
  says to use one.
- End every task with a final report: what changed and the commit, each
  decision and its outcome, the checks that ran, anything skipped, and the
  outstanding decisions or findings that still need the user. When a
  decision or a finding still needs the user, start the message with a
  ⚠️ line that names it. Give it in full when it is new or changed, or
  when the user asks. Otherwise give it in one line.

## Commands

- Build: `cargo build --workspace --all-targets`
- Targeted tests: `cargo test -p <crate> <test name filter>`. A
  browser-backed test target needs a local Chrome or Chromium. Run it one
  test at a time, with the environment that `cargo xtask check` sets:
  `BOWSER_HEADLESS=true BOWSER_INTERNAL_STEALTH_FEATURES=-launch-headed,-launch-native-window cargo test -p <crate> --test <target> <filter> -- --test-threads=1`
- Early check (format, lint, file length):
  `cargo xtask check --include rust-audit,rust-fmt,rust-clippy`. It runs
  the Rust audits, the format check, and Clippy with warnings as errors.
- Smoke test: run the changed command against a real page, for example
  `cargo run -p bowser-cli -- --no-ai https://example.com`. The default
  launch is headed, so on Linux without a display it needs Xvfb. Without
  Xvfb, set the two environment variables of the targeted-test command.
  The command leaves a detached Chrome session. It prints `Session: <ID>`
  on stderr. Close the session with
  `cargo run -p bowser-cli -- session close <ID>`.
  `crates/bowser-cli/README.md` lists more manual smoke paths.
- Complete gate: `cargo xtask check`. It runs these steps in order: the
  release-script tests and ShellCheck, the workflow lint, the Rust audits,
  the format check, and Clippy. Then it runs the `xtask` tests, the Bowser
  unit tests and doctests, the browser-backed integration tests, the CLI
  tests, and a `bowser --help` smoke command. A local run stops at the
  first failed command. CI runs the same command on each pull request into
  `main` and on each push to `main`.
  - A local run needs Chrome or Chromium, Python 3, `actionlint`, and
    ShellCheck. On Linux, it also needs Xvfb and Metacity.
  - `--include` and `--exclude` select phases: `scripts`, `workflow`,
    `rust-audit`, `rust-fmt`, `rust-clippy`, `rust-test`, and `bowser`. A
    run with a filter is partial verification.
- Format: `cargo fmt --all`

## CI

- Agent merges: off

## Repository rules

- `cargo xtask rust-file-length-lint` caps each Rust file under `crates/`
  and `xtask/` at 300 lines. The cap has no override. Without `--all`, the
  lint checks the Rust files that differ from `origin/main` and the files
  that the working tree changes or adds. With `--all`, it checks every Rust
  file under those directories. `cargo xtask check` runs it with `--all`.
- The `rust-audit` phase of `cargo xtask check` runs three more checks:
  - `cargo xtask rust-source-audit` fails on a Rust source file that no
    target's module tree includes.
  - `cargo xtask rust-trait-audit` fails on a hand-written trait
    implementation in a source-adjacent test file. Such a file is under a
    `_tests_/` directory or ends in `_tests.rs`, outside crate-root
    `tests/`. The approved exceptions are the entries in
    `xtask/src/rust_trait_audit_allowlist.rs`. A stale entry also fails
    the audit. The audit also fails when
    `docs/dev/rust/architecture/dependencies.md` is missing.
  - A test-layout check fails on a directory named `tests` inside `src`.
    It also fails on an inline `#[test]` or `#[tokio::test]` in a source
    file. It also fails on a `#[cfg(test)]` without a
    `#[path = "_tests_/..."]` module declaration directly after it. It has
    no command of its own. `cargo xtask check` runs it first, before any
    other command.
- `docs/dev/rust/architecture/dependencies.md` gives the boundaries of the
  three crates.

## Review focus

- The match between the changed behaviour and its protocol doc under
  `docs/protocol/bowser/`.
- Machine contracts: JSON envelope v1, checkpoint v1, the stable error
  codes, and capability flags that report only verified features.
- Session and process ownership: resume, close, and expiry check the live
  process identity. Cleanup stops only Bowser-owned Chrome and Xvfb
  processes. It deletes only Bowser-owned profiles.
- Sensitive data: password values stay redacted in captures, metadata,
  YAML, and JSON. Cookies, localStorage values, and API keys stay out of
  logs and error text. Checkpoint files are plaintext by contract.
- Detach after failure: a command that creates or resumes a session
  detaches and prints the session ID, also when a later step fails.
- Conservative stealth: Bowser keeps native Chrome values. It changes no
  part of the fingerprint without a complete, tested browser identity.

## Status

- This project is not in production. Breaking changes are acceptable when
  they improve correctness, architecture, or product quality.
