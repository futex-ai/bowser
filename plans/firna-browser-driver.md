# Firna Browser Driver

Implement Bowser's portable browser-session checkpoint and machine-facing CLI
contracts for the Firna platform, aligned with the Bowser protocol documents.
Window-target reporting remains deferred and is advertised as unavailable.

## Milestone 1: Protocol contracts

Define the public contracts before implementation so checkpoint portability,
machine output, capabilities, errors, and launch-argument precedence are
unambiguous.

- [x] Translate the Firna handoff into Bowser protocol documentation.
- [x] Define checkpoint format v1, state coverage, exclusions, and restore behavior.
- [x] Define JSON envelope v1, command payloads, and stable error codes.
- [x] Define the capabilities response and deferred `window_target` feature.
- [x] Define caller-supplied Chrome argument precedence and reserved conflicts.

## Milestone 2: Portable checkpoint library

Add a typed, versioned CDP checkpoint boundary that exports live state safely
and restores it into a fresh Bowser-owned session.

- [x] Add checkpoint model, serializer, validation, and typed failures.
- [x] Add browser-engine export and restore behavior for cookies, localStorage,
      open-tab URLs, and selected-page ordinal.
- [x] Ensure restore creates a fresh session/profile and never trusts exported
      host process or profile identity.
- [x] Add unit and real-browser integration coverage, including continued use
      of a live session after export and use after destructive source cleanup.
- [x] Update the library README with the new public boundary.

## Milestone 3: Machine CLI and capabilities

Expose every supported non-interactive command through one versioned JSON
envelope while preserving existing human-facing output.

- [x] Add `--json-envelope`, one-object stdout emission, typed result payloads,
      and stable error-code mapping.
- [x] Include the active session/page in success and failure envelopes whenever
      known, including failures after fresh-session creation.
- [x] Add `session export`, `session restore`, and `capabilities` commands.
- [x] Report capture JSON or output-file references, screenshot metadata, and
      download path/byte metadata in command results.
- [x] Verify caller Chrome args reach Chrome and enforce documented conflicts.
- [x] Add golden/shape tests across commands and representative failures.
- [x] Update CLI and workspace READMEs.

## Milestone 4: Verification and handoff

Run all repository-required checks and make the completed work reviewable.

- [x] Run focused formatting, clippy, unit, integration, CLI, and smoke tests.
- [x] Run `cargo xtask check` with a 100% pass rate.
- [x] Review the final diff against `origin/main`.
- [x] Run `git add -A`, commit all work with a Conventional Commit, and push the
      current branch.
- [ ] Run `cargo xtask review` after the push and report each finding without
      automatically changing code in response. Attempted twice after the push;
      the reviewer failed before analysis because the OpenAI API returned HTTP
      401 for both WebSocket and HTTPS authentication.
- [ ] Update this plan and the plan index to completed after the post-push
      review.
