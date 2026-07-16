### Interactive REPL Commands

Once in interactive mode, the user is presented with a prompt and can issue commands:

```
bowser>
```

| Command | Description |
|---|---|
| `goto <URL>` | Navigate to a URL. Bare public hostnames such as `slack.com` should be treated as `https://slack.com`, while loopback hosts such as `localhost:3000` may default to `http://`. Bowser should use one bounded navigation deadline, mark cached captures/live element IDs stale before requesting document-changing navigation, use browser navigation first on fresh pages so redirects can be correlated by main-frame loader, use top-level location assignment first on resumed pages to avoid stale target-session navigation hangs, and verify the destination document is ready before follow-up capture or interaction. |
| `back` | Go back in browser history and mark cached captures/live element IDs stale before requesting the history command |
| `forward` | Go forward in browser history and mark cached captures/live element IDs stale before requesting the history command |
| `reload` | Reload the current page and mark cached captures/live element IDs stale before requesting the reload |
| `pages` | List known session pages using stable page IDs and mark the selected page |
| `page <PAGE_ID>` | Switch to a different live page in the current session |
| `new page [URL]` | Open a new page, optionally navigating it immediately, then select it |
| `close page [PAGE_ID]` | Close the current selected page, or the given live page, then select the replacement page |
| `yaml` | Print the YAML representation of the current page |
| `yaml <ID>` | Print the YAML subtree rooted at the element with the given ID |
| `click <ID>` | Click the element with the given ID. Visible targets should prefer a pointer-driven browser click with cursor movement and scrolling; non-visible, failed, or stalled pointer clicks may fall back to a synthetic DOM click. If the target is focusable, Bowser also focuses it so the next capture and `meta` output reflect the active control. |
| `keypress <KEY...>` | Press one or more keys on the currently focused element. Space-separated keys are held together as a combo, for example `keypress cmd enter`. |
| `type <ID> <TEXT>` | Replace the current value, then type text into the target text input one character at a time using real browser keyboard events with tiny randomized pauses. Text containing spaces should be quoted: `type 8 "hello world"` |
| `clear <ID>` | Clear the value of an input element by focusing it, selecting the existing value, and pressing Backspace |
| `select <ID> <VALUE>` | Select an option in a `<select>` element by visible text |
| `submit <ID>` | Submit the form containing the element with the given ID. Bowser should prefer Enter on a text-like focused/form control, or a real click when the target itself is a submit button, before falling back to DOM form submission helpers. |
| `scroll <down\|up\|to ID>` | Scroll the page down/up by one viewport using mouse-wheel input, or scroll directly to an element |
| `screenshot [PATH]` | Take a full-page screenshot. Default path: `./screenshot-<timestamp>.png` |
| `screenshot <ID> [PATH]` | Take a screenshot of a specific element |
| `wait <SELECTOR>` | Wait for a CSS selector to appear (up to timeout) |
| `refresh` | Re-capture the page YAML (useful after dynamic content updates) |
| `expand <ID>` | Print the fully expanded YAML subtree for a previously truncated node |
| `meta <ID>` | Print metadata for any captured element with an ID, including focus and live visibility details |
| `describe <ID>` | Generate or print a cached AI description for an image element |
| `session` | Print the current Bowser session ID |
| `url` | Print the current URL |
| `title` | Print the current page title |
| `html` | Print the current full rendered document as raw HTML |
| `js <EXPRESSION>` | Execute JavaScript and print the result |
| `help` | Print available commands with usage and one-line descriptions |
| `quit` / `exit` | Exit interactive mode and print the resumable session ID |

For REPL commands that take an element ID, Bowser accepts either the bare numeric ID (`12`) or the rendered YAML key form (`input#12`, `link#12`, `table#12`). When a prefix is present, Bowser ignores it and uses the numeric suffix.

### Interactive Mode Output Behaviour

After any navigation or interaction command (`goto`, `page`, `new page`, `close page`, `click`, `keypress`, `type`, `clear`, `select`, `submit`, `back`, `forward`, `reload`), Bowser automatically:

1. Waits for the page to reach a stable state. Bowser should initially prefer `document.readyState === "complete"`, bounded network idleness, and no pending DOM mutations for 500ms, but after a short grace period it may ignore still-open background requests as long as the DOM is quiet so chatty pages do not stall the interaction loop for the full timeout.
2. Re-captures the page structurally. This re-capture must not call the AI image-description provider automatically; explicit image descriptions remain opt-in through `describe <ID>`.
3. Prints a `---` document separator line.
4. Prints the updated YAML.

This gives the user an immediate view of the result of their action. For page-management commands that select an already captured page (`page <PAGE_ID>` and replacement selection after `close page`), Bowser may print the stored selected-page preview if Chrome stalls or reports a stale target during the live re-capture. The `refresh` command manually triggers a new re-capture without performing any action — useful when the page updates asynchronously (e.g., WebSocket data, timers).

Interactive page renders are separated by a standalone `---` line so it is visually obvious when a new full-page capture begins. The same separator is printed before `yaml` when it emits the current full page. Subtree output such as `yaml <ID>`, `expand <ID>`, `meta <ID>`, and `describe <ID>` does not add a page separator. The `html` command likewise prints raw document text without a separator.

The `pages` command does not mutate page state. It prints the known session pages using stable page IDs such as `pg_1`, marks the selected page, and shows the stored page summary information.

The `page <PAGE_ID>` command switches the selected page within the current session, attempts to activate that page in the browser, then prints a fresh capture for it. If the live activation or re-capture stalls and a stored preview exists for that page, Bowser may print that stored preview instead so the prompt remains responsive.

The `new page [URL]` command creates a new page in the current session, selects it, optionally navigates it immediately, and then prints a fresh capture for that new page.

The `close page [PAGE_ID]` command closes the current selected page by default, or the explicit page ID when provided, then selects a replacement live page and prints its capture or stored preview. The close request must be bounded and may fall back to Chrome target-level close if the page-level close stalls. If closing the last remaining page would leave the session empty, Bowser creates and selects a replacement `about:blank` page.

The `type <ID> <TEXT>` command replaces the current field value, then simulates a very fast human typist with real browser keyboard input. Bowser focuses the target, selects the existing value, and sends per-character CDP key events with tiny randomized pauses between characters. It must not synthesize DOM `KeyboardEvent` or `InputEvent` objects for normal typing, and it must not force a synthetic `change` event at the end of typing. For automated tests, Bowser may use a fixed random seed to make the cadence reproducible.

The `keypress <KEY...>` command dispatches real browser keyboard input against the currently focused element, or the page itself when nothing more specific is focused. Keys are pressed together: Bowser sends `keydown` for each key in order, then releases them in reverse order. Printable characters, `Enter`, and `Space` should still surface as normal browser key/input behavior because the underlying input source is CDP key dispatch rather than synthetic DOM events. Special-key aliases include `cmd`/`meta`, `ctrl`, `alt`, `shift`, `enter`, and `space`.

The `click <ID>` command first attempts a user-like pointer click when the target is currently visible and can receive pointer events. Bowser scrolls the element into view when necessary, starts from a randomized in-viewport idle cursor location on the first visible click in a live attached page, then follows the shape of the CLI pointer telemetry fixture: a tiny idle drift, coarse travel, a slower approach into the target, final micro-corrections, and a short browser-level press/release hold. The exact path, target point, pauses, and hold duration remain randomized. Later visible clicks on the same attached page continue from the last simulated cursor position. If the target is too small for a full 10px inset, Bowser may use the nearest valid interior point. If the target is not visibly clickable, or if low-level pointer movement/button dispatch fails or stalls, Bowser falls back to the synthetic DOM `click()` path instead of waiting for the outer REPL command timeout.

The stability wait used by `capture()` and interactive post-command re-captures should not block indefinitely on long-lived background activity. After a short grace period, Bowser may stop requiring strict network idleness while still requiring `document.readyState === "complete"` and a quiet DOM window.

The `clear <ID>` command should follow the same real-input path as typing: focus the control, select its existing value, and send a Backspace key press rather than assigning `value = ""` directly.

The `submit <ID>` command should prefer the most user-like path available. If the target is itself a submit button, Bowser should click it. If the target or its containing form has an appropriate text-like control, Bowser should focus that control and press Enter. Only when no reasonable user-like path is available may Bowser fall back to `form.requestSubmit()` or `form.submit()`.

The `scroll down` and `scroll up` commands should prefer browser mouse-wheel input anchored at the current simulated cursor location when it is on-screen, or a reasonable in-viewport fallback point otherwise. `scroll to <ID>` may still use direct bring-into-view behavior because it is a targeted assist command rather than a generic page scroll gesture.

The `expand <ID>` command does not mutate page state. It prints the fully expanded subtree for the truncated node referenced by that element ID from the most recent capture.

The `meta <ID>` command does not mutate page state. It prints metadata for the referenced element from the most recent capture, enriched with the element's current `focused` state, live runtime visibility state from the current page, and current page-space bounds.

The `describe <ID>` command does not mutate DOM state. It targets images only. If the current session state already holds a cached image `description`, Bowser may print that immediately. Otherwise it screenshots the image element, sends it to the configured AI backend, caches the result in session state, and prints a structured payload with `alt`, `src`, `filename`, and `description`. Compact page YAML should continue to label images from `alt` plus filename; `describe` is the explicit path for the AI-generated prose.

After a detached session is resumed, the first ID-based runtime command such as `meta <ID>`, `describe <ID>`, `click <ID>`, or `type <ID> ...` may refresh the page capture lazily before executing so the live `__bowserElements` mapping exists again for runtime lookup. That lazy rebuild may use a lightweight structural recapture that skips the normal startup stability wait because its only purpose is to rebuild the live DOM-to-ID map.

If an underlying browser wait blocks longer than the configured timeout, Bowser should abort that in-flight wait, print a timeout error, keep the interactive session alive, and return to a fresh prompt. For REPL commands that also owe the user a follow-up capture, Bowser may reserve a small additional outer-command grace window so the required re-capture can finish instead of being cut off exactly at the page-load timeout boundary. When a timeout happens after navigation or some other state-changing action, Bowser should treat the prior capture IDs as stale until the next successful capture.

### Interactive Prompt Behaviour

The interactive prompt uses shell-like line editing and history via `rustyline`.

The prompt is URL-aware and colorized in interactive terminals. It includes the current selected page URL in the prompt prefix so the active page is visible before each command.

When interactive mode resumes an existing detached session without an explicit new `URL` or `wait` request, Bowser may print the stored selected-page preview immediately instead of blocking on a fresh startup recapture. Live DOM IDs are then rebuilt lazily by the first runtime command that actually needs them.

`Ctrl-C` does not immediately exit the REPL:

1. If the current input buffer is non-empty, the first `Ctrl-C` clears that buffer and shows a fresh prompt.
2. If the prompt is already empty, the next `Ctrl-C` exits interactive mode and prints the resumable session ID to `stderr`.

If a command is already running, `Ctrl-C` cancels that in-flight command instead of waiting for it forever. After cancellation, Bowser prints an interruption message, keeps the session alive, clears the most recent capture IDs because page state may have changed, and returns to a fresh prompt.

The REPL also provides suggestions:

- command-name completion and suggestions while typing
- closest-match suggestions for unknown commands
- element-ID suggestions from the most recent capture for commands such as `click`, `type`, `yaml`, `expand`, `meta`, and `describe`
- page-ID suggestions from the known session page inventory for commands such as `page` and `close page`
- common special-key suggestions for `keypress`, such as `cmd`, `enter`, and `space`
- an inline faded/gray auto-completion hint once the user has typed at least one character of the current token
- pressing Right Arrow while the cursor is at the end of the line accepts the visible inline hint

Suggestions are best-effort and do not affect command semantics.

The `help` command prints a formatted command reference in interactive mode. Each line includes the command usage form and a short description, followed by a note that ID-taking commands accept either bare numeric IDs such as `12` or rendered keys such as `input#12`. For `keypress`, the help text must make it clear that combo keys are space-separated in the REPL, for example `keypress cmd enter`.

If the user enters an invalid command, Bowser prints the parse error, keeps the interactive session alive, and shows a fresh prompt. Invalid commands must not terminate the REPL or detach the session. The same is true for recoverable command failures such as timeouts or interrupted in-flight commands.

### Stability Detection

"Page stable" is defined as:

1. **Network idle**: no more than 2 in-flight network requests for 500ms (matches Chrome's `networkIdle` event semantics).
2. **DOM settled**: no DOM mutations observed via `MutationObserver` for 500ms.
3. **Timeout cap**: `wait_for_stable()` itself returns a timeout error if stability is not reached within the configured timeout. Capture flows may ignore that error and proceed with whatever state is available rather than failing the capture outright.

For the `refresh` command and library `capture()` method, only DOM settled is checked (no new navigation occurs).
