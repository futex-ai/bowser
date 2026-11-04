# Firna contracts merged without a post-push review

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium. **Harder later:** Yes.
- **Kind:** Post-merge task.
- **Added:** 2026-11-04, from the branch `conductor/agents-md-bowser`, which deleted the plan `plans/firna-browser-driver.md`.
- **Location:** `crates/bowser/src/checkpoint/`, `crates/bowser/src/browser/checkpoint/`, `crates/bowser-cli/src/output.rs`, `crates/bowser-cli/src/error.rs`, `crates/bowser-cli/src/commands/capabilities.rs`, `crates/bowser-cli/src/commands/session.rs`, `crates/bowser-cli/src/dispatch.rs`, `crates/bowser/src/config/validation.rs`, and `docs/protocol/bowser/checkpoints-and-machine-output.md`.
- **Reason:** The change is already on `main`, so only a review of the merged code can still find its defects.

**Terms:**

- **Checkpoint:** a plaintext JSON file that `bowser session export` writes
  and `bowser session restore` reads, with cookies, localStorage, open-tab
  URLs, and the selected tab.
- **JSON envelope:** the one JSON object that `--json-envelope` makes each
  non-interactive command print on stdout.

**Background:**

- Pull request #2, "Add portable Bowser contracts for Firna", merged on
  2026-08-13 as commit `5ef8015`. It changed 56 files.
- Pull request #2 added checkpoint export and restore, the JSON envelope
  with stable error codes, the `capabilities` command, and precedence rules
  for caller Chrome arguments.
- The plan of pull request #2 required a post-push run of
  `cargo xtask review`. Both attempts failed before analysis, because the
  OpenAI API returned HTTP 401 for WebSocket and HTTPS authentication. So
  no review read the change.
- Later pull requests #3 and #4 changed some of the same files.
- Firna is a separate platform that uses Bowser. Firna depends on the
  checkpoint and envelope formats.

**Task:** Review the code that pull request #2 added, as it is on `main`
now. Use the `implementation-review` skill of `futex-ai/skills`. The paths
under **Location** are the named area of that review. They replace the
diff as its scope. Handle each finding with the fix rule of that skill.

**Impact if no action:** A defect in checkpoint export and restore or in
the JSON envelope can stay unnoticed. Checkpoints hold login cookies. A
format defect costs more to fix after more callers depend on the format.

**Needs:** Nothing beyond the repository and a review agent.

**Done when:** A review of the area under **Location** has run. Each of its
findings has a fix or a follow-up item.
