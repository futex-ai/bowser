# Google Smoke Results

This page records live Google search smoke results for Bowser stealth work.
Rows are evidence for a specific date, network, Chrome build, profile state,
and Bowser revision; they are not permanent claims about Google's detector.

Use this page when comparing PRs that change Bowser launch flags, identity,
CDP usage, capture, or input behavior. Store raw logs under `.context/` and
keep this page as the durable cross-PR ledger.

## Default Stealth Feature Set

Bowser treats stealth as one default browser behavior. There is no public
low-runtime mode flag; `BOWSER_INTERNAL_STEALTH_FEATURES` remains a
developer-only matrix override for testing one behavior at a time.

| Feature ID | Default behavior | Independent test question |
|---|---|---|
| `launch-headed` | With stealth enabled, Bowser omits `--headless=new` and launches headed Chrome. | Does native headless fail when every other stealth behavior stays enabled? |
| `launch-native-window` | Headed stealth mode avoids forcing `--window-size` on a real display; Bowser-owned Xvfb displays still use the configured viewport as window size. | Does adding a forced headed window size on a real display trigger `/sorry/` by itself? |
| `runtime-disable` | Off by default. Bowser leaves CDP Runtime-domain events enabled unless an internal/config diagnostic explicitly sends `Runtime.disable`. | Does sending `Runtime.disable` help or hurt Google acceptance? |
| `accessibility-capture` | Off by default. Bowser uses its full JS DOM traversal for normal captures; the accessibility snapshot path remains an internal probe because it is too lossy for lists, images, iframe structure, and link metadata. | Does enabling accessibility-tree capture change Google acceptance when launch/input stay stealth-safe? |
| `skip-runtime-stability` | Off by default. Bowser keeps the Runtime-based DOM stability sampler enabled. | Does skipping the stability sampler help or hurt Google acceptance? |
| `backend-focus-input` | Types by backend-node focus plus keyboard events where possible. | Does JS focus/text-entry preparation trigger `/sorry/` when capture remains stealth-safe? |
| `enter-submit` | Submits Google search by focusing the query input and pressing Enter for the search button path. | Does pointer-clicking the visible Google Search button remain accepted? |
| `backend-pointer-target` | Resolves pointer targets from backend-node geometry before falling back to page JS element lookup. | Does JS element lookup/action targeting change Google acceptance? |

`runtime-disable` does not disable page JavaScript. When enabled for diagnosis,
it only asks Chrome to stop sending DevTools Runtime-domain events for the page.

## Recording Rules

- Record the exact variant, profile mode, result, and evidence marker.
- Treat a result as stable only when a known passing control still passes in
  the same test window.
- Mark runs as inconclusive when repeated probes appear to poison the control
  flow or when the flow fails before the Google search assertion.
- Do not store cookies, profile data, page HTML, or personally identifying
  request details in this file.
- Prefer `pass`, `fail`, or `inconclusive` in the result column.

## Passing Control Contract

The current accepted automation control is the gstack-style headed Playwright
persistent-context flow. Bowser alignment work must compare against this
contract before judging any feature as accepted or rejected.

Control launch and identity:

- Use Playwright Chromium or system Chrome in headed mode.
- Launch a persistent context with `viewport: null`.
- Use a fresh profile unless the variant explicitly says it is persistent.
- Use gstack's headed-mode custom User-Agent shape by default:
  native Chrome version plus a trailing `GStackBrowser` token. Direct-control
  variants may set `BOWSER_GOOGLE_CONTROL_NATIVE_UA=1` to test the native UA.
- Pass `--hide-crash-restore-bubble` and
  `--disable-blink-features=AutomationControlled`.
- Ignore Playwright's default extension-disabling args:
  `--disable-extensions` and
  `--disable-component-extensions-with-background-pages`.
- Do not route traffic through Bowser-managed proxy logic; network routing is
  external to Bowser.
- Inject only the narrow gstack script before navigation: an own-property
  `navigator.webdriver` mask, enumerable `cdc_`/`__webdriver` global cleanup,
  and the notifications-only Permissions API patch.

Control flow:

- Navigate to `https://www.google.com`.
- Accept a cookie prompt when one appears.
- Fill `hello` into `textarea[name="q"]` or `input[name="q"]`.
- Submit with Enter.
- Treat the run as passing only when the terminal URL is a Google `/search`
  URL with `q=hello` and not `/sorry/`.

Run the direct control:

```bash
bash scripts/bowser-google-control.sh
```

Run the direct control and Bowser's current headed low-runtime control in the
same test window:

```bash
bash scripts/bowser-google-control.sh
```

The direct-control runner writes structured JSON logs under
`.context/google-headless-matrix/logs/`. A Bowser regression or feature
add-back result is meaningful only when this direct-control runner passes in
the same test window.

## Result Ledger

| Date | Scope | Variant | Profile | Result | Evidence | Notes |
|---|---|---|---|---|---|---|
| 2026-05-12 | external baseline | stock Chrome on user device and network | existing user profile | pass | Google search works | User-confirmed residential network baseline; network reputation is not the primary blocker. |
| 2026-05-12 | direct control | Playwright Chromium, native headless | fresh | fail | `/sorry/` | Fresh native headless Chromium is challenged by Google on this machine/network. |
| 2026-05-12 | direct control | Playwright Chromium, headed | fresh | pass | `/search?q=hello` | Fresh headed Chromium passed before repeated probing. |
| 2026-05-12 | direct control | Playwright Chromium, headed with `--window-size=1920,1080` | fresh | fail | `/sorry/` | Bowser's old forced headed window size reproduced the challenge. |
| 2026-05-12 | Bowser accepted mode | `bowser --no-ai --disable-runtime-events -i https://www.google.com` | fresh | pass | `google_search_flow_avoids_captcha` | Headed Chrome, no forced window size, accessibility capture, backend-node input. |
| 2026-05-12 | Bowser headless probe | low-runtime path plus `BOWSER_CHROME_ARGS=--headless=new` | fresh | fail | `/sorry/` | First headless add-back probe after accepted mode; still challenged. |
| 2026-05-12 | Bowser noisy control | low-runtime headed control after repeated live probes | fresh | inconclusive | `/sorry/` | Treat as polluted signal until a spaced control run passes again. |
| 2026-05-12 | Bowser matrix | headed-low-runtime-control | fresh | fail | `/sorry/` | Spaced control rejected by Google; matrix run 20260512T150845Z; raw log .context/google-headless-matrix/logs/headed-low-runtime-control.log |
| 2026-05-12 | baseline restore | current matrix command vs accepted Bowser flow | fresh | inconclusive | command-equivalent | `headed-low-runtime-control` invokes the same live test command path as the earlier accepted Bowser mode; the failure is not a matrix setup difference. |
| 2026-05-12 | direct control | Playwright bundled Chromium, headed, Enter submit | fresh | fail | `/sorry/` | Direct headed Chromium is now also challenged; raw log .context/google-headless-matrix/logs/direct-headed-chromium-enter-20260512T151756Z.log |
| 2026-05-12 | direct control | Playwright system Chrome, headed, Enter submit | fresh | fail | `/sorry/` | Direct headed system Chrome under Playwright is also challenged; raw log .context/google-headless-matrix/logs/direct-system-chrome-enter-20260512T151950Z.log |
| 2026-05-12 | baseline restore | Bowser headed low-runtime persistent profile | persistent | fail | `/sorry/` | Persistent-profile Bowser low-runtime run is also challenged; raw log .context/google-headless-matrix/logs/bowser-headed-low-runtime-persistent-20260512T151828Z.log |
| 2026-05-12 | baseline restore | current automation baseline | mixed | inconclusive | no passing control | Stock Chrome on the user profile was previously confirmed passing, but all current headed automation controls reject; feature add-back milestones are blocked until a fresh control passes. |
| 2026-05-13 | Bowser matrix | headed-low-runtime-control | fresh | fail | `/sorry/` | Overnight retry still rejected by Google; matrix run 20260513T081934Z; raw log .context/google-headless-matrix/logs/20260513T081934Z-headed-low-runtime-control.log |
| 2026-05-13 | manual control | manual Google search in the same browser | existing browser | pass | Google search works | User-confirmed same-browser manual search works; current baseline failures should be treated as automation-path-specific rather than session, network, or Chrome-binary failures. |
| 2026-05-13 | Bowser matrix | headed-low-runtime-control | fresh | fail | `/sorry/` | Matrix run 20260513T084354Z; raw log .context/google-headless-matrix/logs/20260513T084354Z-headed-low-runtime-control.log |
| 2026-05-13 | Bowser matrix | headed-low-runtime-control, URL-only post-submit assertion | fresh | fail | `/sorry/` | Matrix run 20260513T085927Z; raw log .context/google-headless-matrix/logs/20260513T085927Z-headed-low-runtime-control.log |
| 2026-05-13 | Bowser matrix | addback-enter-submit-pointer-submit | fresh | fail | `/sorry/` | Matrix run 20260513T090021Z; feature result is not conclusive because the same-window Bowser control failed; raw log .context/google-headless-matrix/logs/20260513T090021Z-addback-enter-submit-pointer-submit.log |
| 2026-05-13 | manual-handoff probe | Bowser-launched Chrome plus macOS AppleScript OS input | fresh | inconclusive | Apple Events denied | `osascript` could not send events to System Events on this machine, so the manual-handoff baseline was blocked before search input. |
| 2026-05-13 | external control | gstack headed mode via `launchPersistentContext` | fresh | pass | `/search?q=hello` | Same machine and network; gstack source uses a narrow webdriver mask, cdc cleanup, and notification Permissions API patch. |
| 2026-05-13 | direct control | Playwright bundled Chromium, gstack-like headed fill plus Enter | fresh | pass | `/search?q=hello` | Raw log .context/google-headless-matrix/logs/20260513T091541Z-direct-playwright-gstack-like-fill-enter.log |
| 2026-05-13 | direct control | Playwright bundled Chromium, gstack-like headed keyboard type plus Enter | fresh | pass | `/search?q=hello` | Raw log .context/google-headless-matrix/logs/20260513T091633Z-direct-playwright-gstack-like-keyboard-enter.log |
| 2026-05-13 | direct control | system Chrome, gstack-like headed keyboard type plus Enter | fresh | pass | `/search?q=hello` | Raw log .context/google-headless-matrix/logs/20260513T091726Z-direct-system-chrome-gstack-like-keyboard-enter.log |
| 2026-05-13 | direct control | system Chrome plus `Runtime.disable` before navigation | fresh | pass | `/search?q=hello` | `Runtime.disable` alone does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T091829Z-direct-system-chrome-gstack-like-runtime-disable.log |
| 2026-05-13 | direct control | system Chrome plus Bowser-like UA and UA-CH override | fresh | pass | `/search?q=hello` | UA override alone does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T091940Z-direct-system-chrome-gstack-like-ua-override.log |
| 2026-05-13 | direct control | system Chrome with remote debugging port and Playwright-over-CDP actions | fresh | pass | `/search?q=hello` | Remote debugging port alone does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T092043Z-direct-system-chrome-remote-port-playwright-actions.log |
| 2026-05-13 | direct control | remote debugging port plus accessibility captures between actions | fresh | pass | `/search?q=hello` | AX capture alone does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T092146Z-direct-system-chrome-remote-port-ax-captures.log |
| 2026-05-13 | direct control | remote debugging port plus Bowser-like `DOM.focus` and raw key events | fresh | pass | `/search?q=hello` | Bowser-style raw input alone does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T092326Z-direct-system-chrome-remote-bowser-input.log |
| 2026-05-13 | direct control | remote debugging port plus Bowser-like cookie accept and search input | fresh | pass | `/search?q=hello` | Bowser-style cookie accept plus raw search input does not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T092459Z-direct-system-chrome-bowser-accept-enter.log |
| 2026-05-13 | direct control | system Chrome with Bowser launch args | fresh | pass | `/search?q=hello` | `--lang=en-GB`, `--force-device-scale-factor=1`, and remote debugging port do not reproduce the Bowser failure; raw log .context/google-headless-matrix/logs/20260513T092608Z-direct-system-chrome-bowser-launch-args.log |
| 2026-05-13 | direct control | system Chrome with extra blank target before Google navigation | fresh | pass | `/search?q=hello` | Bowser's extra blank target shape does not reproduce the failure; raw log .context/google-headless-matrix/logs/20260513T092733Z-direct-system-chrome-extra-blank-target.log |
| 2026-05-13 | direct control | system Chrome with Bowser's visible CDP sequence | fresh | pass | `/search?q=hello` | Explicit Bowser trace calls do not reproduce the failure outside Bowser; raw log .context/google-headless-matrix/logs/20260513T093235Z-direct-system-chrome-exact-cdp-sequence.log |
| 2026-05-13 | direct control | system Chrome with chromiumoxide-like init commands and Bowser's visible CDP sequence | fresh | pass | `/search?q=hello` | Approximate chromiumoxide init sequence still passes outside Bowser; raw log .context/google-headless-matrix/logs/20260513T093703Z-direct-system-chrome-chromiumoxide-init-cdp.log |
| 2026-05-13 | Bowser manual REPL | headed low-runtime flow with explicit CDP trace | fresh | fail | `/sorry/` | The recorded trace only covers Bowser's explicit `cdp_trace` calls, not internal chromiumoxide traffic; raw trace .context/google-headless-matrix/logs/20260513T092934Z-bowser-manual-trace.jsonl |
| 2026-05-13 | direct control | checked-in runner with native UA and Bowser-stricter init script | fresh | fail | `/sorry/` | This reproduced the challenge before the runner was aligned to gstack's exact headed UA and simpler init script; raw log .context/google-headless-matrix/logs/20260513T110231Z-direct-gstack-like-control.json |
| 2026-05-13 | direct control | checked-in runner with exact gstack headed UA and init script | fresh | pass | `/search?q=hello` | Raw log .context/google-headless-matrix/logs/20260513T110539Z-direct-gstack-like-control.json |
| 2026-05-13 | Bowser matrix | headed-low-runtime-control | fresh | pass | completed | matrix run 20260513T110632Z; raw log .context/google-headless-matrix/logs/20260513T110632Z-headed-low-runtime-control.log |
| 2026-05-13 | Bowser matrix | headed-low-runtime-control | fresh | pass | completed | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-headed-low-runtime-control.log |
| 2026-05-13 | Bowser matrix | addback-launch-headed-native-headless | fresh | fail | google_sorry | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-launch-headed-native-headless.log |
| 2026-05-13 | Bowser matrix | addback-launch-native-window-forced-size | fresh | pass | completed | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-launch-native-window-forced-size.log |
| 2026-05-13 | Bowser matrix | addback-runtime-disable-runtime-events-on | fresh | inconclusive | trace_assertion | Matrix run reached `/search?q=hello`; the failure was the old smoke-test trace assertion, corrected by the later passing runtime-disable add-back row. Raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-runtime-disable-runtime-events-on.log |
| 2026-05-13 | Bowser matrix | addback-accessibility-capture-js-dom | fresh | inconclusive | flow_not_search | Historical run from the earlier accessibility-capture default: JS DOM capture changed the flow before the Google search assertion; the run stayed on the consent page rather than proving a Google captcha rejection. Raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-accessibility-capture-js-dom.log |
| 2026-05-13 | Bowser matrix | addback-skip-runtime-stability-sampler | fresh | pass | completed | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-skip-runtime-stability-sampler.log |
| 2026-05-13 | Bowser matrix | addback-backend-focus-input-js-focus | fresh | inconclusive | flow_timeout | JS focus/text-entry setup timed out before the Google search assertion, so this is an automation-flow failure rather than a confirmed captcha rejection. Raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-backend-focus-input-js-focus.log |
| 2026-05-13 | Bowser matrix | addback-enter-submit-pointer-submit | fresh | pass | completed | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-enter-submit-pointer-submit.log |
| 2026-05-13 | Bowser matrix | addback-backend-pointer-target-js-target | fresh | pass | completed | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-addback-backend-pointer-target-js-target.log |
| 2026-05-13 | Bowser matrix | headless-low-runtime-persistent-profile | persistent | fail | google_sorry | matrix run 20260513T110920Z; raw log .context/google-headless-matrix/logs/20260513T110920Z-headless-low-runtime-persistent-profile.log |
| 2026-05-13 | Bowser matrix | addback-runtime-disable-runtime-events-on | fresh | pass | completed | matrix run 20260513T112133Z; raw log .context/google-headless-matrix/logs/20260513T112133Z-addback-runtime-disable-runtime-events-on.log |
| 2026-05-13 | Bowser default stealth | `bowser --no-ai -i https://www.google.com` | fresh | pass | `/search?q=hello` | Public `--disable-runtime-events` flag removed; default stealth leaves Runtime events enabled and still passes. Raw log .context/google-headless-matrix/logs/20260513T113440Z-default-stealth-no-flag.log |
| 2026-05-13 | Bowser default stealth | `bowser --no-ai -i https://www.google.com` | fresh | pass | `/search?q=hello` | Default stealth now uses full JS DOM capture, keeps Runtime events enabled, accepts the consent dialog, and reaches search results. Raw log .context/google-headless-matrix/logs/20260513T123700Z-default-stealth-dom-capture.log |
<!-- google-smoke-results:end -->

## Matrix Runner

The tracked helper `scripts/bowser-google-smoke-matrix.sh` runs selected live
variants, writes raw logs under `.context/google-headless-matrix/`, and appends
summary rows to this ledger. It uses the internal-only
`BOWSER_INTERNAL_STEALTH_FEATURES` override to toggle one feature ID for each
matrix test. Use `BOWSER_GOOGLE_MATRIX_ONLY` to run one variant at a time, and
keep a cooldown between runs when Google starts challenging the headed control.

Example:

```bash
BOWSER_GOOGLE_MATRIX_ONLY=headless-low-runtime-no-window-size \
BOWSER_GOOGLE_MATRIX_SLEEP_SECONDS=60 \
bash scripts/bowser-google-smoke-matrix.sh
```
