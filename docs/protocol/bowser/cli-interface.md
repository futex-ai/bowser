## CLI Interface

### Binary Name

`bowser`

### Global Flags

```
bowser [OPTIONS] <COMMAND|URL>

Options:
  --chrome-path <PATH>       Path to Chrome/Chromium binary
                              [default: auto-detect]
  --chrome-args <ARGS>       Additional Chrome launch arguments
                              (comma-separated)
  -i, --interactive          Shortcut for `interactive`
  --session <ID>             Resume an existing Bowser session
  --headed                   Launch Chrome with a visible window instead of headless mode
  --persistent-profile       Reuse a stable default Chrome profile path across fresh sessions
  --user-data-dir <DIR>      Chrome user data directory
                              (highest-precedence profile override)
  --session-dir <DIR>        Session metadata directory
                              [default: platform state dir]
  --session-ttl <SECONDS>    Detached session idle TTL [default: 1800]
  --viewport <WxH>           Positive viewport size [default: 1920x1080]
  --timeout <SECONDS>        Per-step page load timeout [default: 30]
  -a, --all                  Disable truncation for this capture
  --no-truncate              Disable collection truncation for this command
  --no-stealth               Disable stealth patches
  --no-ai                    Disable AI image summarization
  --ai-provider <PROVIDER>   AI provider [default: anthropic]
  --ai-model <MODEL>         AI model name
  --config <PATH>            Config file path
                              [default: ~/.config/bowser/config.yaml]
  -v, --verbose              Increase log verbosity (-v, -vv, -vvv)
  -q, --quiet                Suppress non-essential output
  -h, --help                 Print help
  -V, --version              Print version
```

If the first non-flag argument is a URL instead of a named command, Bowser treats it as `get <URL>`.

The platform-default config file is optional when `--config` is omitted. When `--config <PATH>` is supplied, that exact file must exist and any missing-file, read, parse, or validation failure is reported with the selected path.

Navigation entrypoints accept either fully qualified URLs or bare hostnames. Bowser should assume `https://` for public hostnames such as `slack.com`, and `http://` for loopback hosts such as `localhost:3000` or `127.0.0.1:3000`.

### Commands

#### `bowser get <URL>`

Single-shot mode. Navigate to the URL, wait for render, output YAML, JSON, or full rendered HTML, detach from the browser, and print the session ID to `stderr`. Once a session exists, Bowser performs the detach and prints that ID even when a later operation fails, then returns the original operation error.

Shortcut form: `bowser <URL>`

```
bowser get [OPTIONS] <URL>

bowser [OPTIONS] <URL>

Arguments:
  <URL>                      The URL to navigate to

Options:
  --wait <SELECTOR>          Wait for a CSS selector to appear before capture
  --wait-timeout <SECONDS>   Max time to wait for selector [default: 10]
  --delay <MS>               Additional delay after load before capture
                              [default: 0]
  --screenshot <PATH>        Save a full-page screenshot to file
  --output <PATH>            Write the rendered output to file instead of stdout
  --format <FORMAT>          Output format: yaml | json | html [default: yaml]
                              yaml = compact display, json = explicit full data,
                              html = full rendered document
```

Example:

```bash
# basic page capture
bowser get https://example.com

# implicit get
bowser https://example.com

# wait for SPA to render a specific element
bowser get --wait "#main-content" --screenshot page.png https://example.com/app

# output to file as JSON
bowser get --format json --output page.json https://example.com

# output the full current rendered document as HTML
bowser get --format html --output page.html https://example.com

# continue the same browser session later
bowser get --session bsr_01HZX... https://example.com/account
```

After successfully resuming a session, `capture`, non-REPL interaction, and
`page` commands must attempt `BrowserEngine::detach` even when page selection,
the requested action, or rendering fails. The original operation error takes
precedence over a detach error. Terminal and file output must happen only after
the operation and detach both succeed, so an output failure cannot bypass the
detach boundary.

#### `bowser capture`

Capture the currently selected page, or an explicit page ID, from a resumable
session without navigating.

```
bowser capture [OPTIONS]

Options:
  --page-id <PAGE_ID>        Optional page ID to select before capture
  --output <PATH>            Write output to file instead of stdout
  --format <FORMAT>          Output format: yaml | json | html [default: yaml]
```

Example:

```bash
bowser capture --session bsr_01HZX... --page-id pg_2 --format html
```

#### `bowser expand <ELEMENT_ID>`

Expand a previously truncated node from the most recent capture in a resumable session.

```
bowser expand [OPTIONS] <ELEMENT_ID>

Arguments:
  <ELEMENT_ID>               Element ID of a truncated node

Options:
  --output <PATH>            Write YAML to file instead of stdout
  --format <FORMAT>          Output format: yaml | json [default: yaml]
                              yaml = compact display, json = explicit full data
```

`bowser expand` requires a resumable session, typically via `--session <ID>`. The element ID must come from the most recent capture in that session and refer to a truncated node.

Example:

```bash
# expand a truncated list from a previous capture
bowser expand --session bsr_01HZX... 1
```

#### `bowser meta <ELEMENT_ID>`

Fetch metadata for any ID-bearing element from the most recent capture in a resumable session. For links and images this includes deferred fields such as `href`, `src`, and `description`. For all supported elements it also includes live visibility state from the current page, including whether the element is presently visible and directly clickable to a user, plus the current page-space top-left and bottom-right coordinates of the element. Image metadata also includes `describable: true` when the current AI configuration can satisfy `describe`.

```
bowser meta [OPTIONS] <ELEMENT_ID>

Arguments:
  <ELEMENT_ID>               Element ID for any captured element with an ID

Options:
  --output <PATH>            Write YAML to file instead of stdout
  --format <FORMAT>          Output format: yaml | json [default: yaml]
                              yaml = compact display, json = explicit full data
```

Examples:

```bash
# fetch full metadata for link#12 from the last capture in the session
bowser meta --session bsr_01HZX... 12

# fetch full metadata for image#3
bowser meta --session bsr_01HZX... 3
```

#### Browser Interaction Commands

Non-REPL interaction commands run against a detached session and print an
updated capture.

```
bowser click [OPTIONS] <ELEMENT_ID>
bowser type [OPTIONS] <ELEMENT_ID> <TEXT>
bowser clear [OPTIONS] <ELEMENT_ID>
bowser submit [OPTIONS] <ELEMENT_ID>
bowser key [OPTIONS] <KEY>
bowser scroll [OPTIONS]

Options:
  --page-id <PAGE_ID>        Optional page ID to select before the action
  --format <FORMAT>          Output format: yaml | json | html [default: yaml]
  --output <PATH>            Write output to file instead of stdout
  --direction <up|down>      Scroll direction for `scroll`
  --element-id <ELEMENT_ID>  Element to bring into view for `scroll`
```

Examples:

```bash
bowser click --session bsr_01HZX... 12
bowser type --session bsr_01HZX... 4 "hello"
bowser scroll --session bsr_01HZX... --direction down
```

#### `bowser describe <ELEMENT_ID>`

Generate or print a cached explicit AI description for an image from the most recent capture in a resumable session. The target element ID must refer to an image. The result includes the image `alt`, `src`, derived `filename` when available, and the `description` text. An uncached provider request must finish within the global `--timeout` deadline.

```
bowser describe [OPTIONS] <ELEMENT_ID>

Arguments:
  <ELEMENT_ID>               Element ID for a captured image

Options:
  --output <PATH>            Write YAML to file instead of stdout
  --format <FORMAT>          Output format: yaml | json [default: yaml]
                              yaml = compact display, json = explicit full data
```

Examples:

```bash
# explicitly describe an image from the last capture in the session
bowser describe --session bsr_01HZX... 3
```

#### `bowser download <URL> <PATH>`

Download a URL through the active browser session and save the completed file to
`PATH`.

```
bowser download [OPTIONS] <URL> <PATH>
bowser download [OPTIONS] --link <ELEMENT_ID> --output <PATH>

Arguments:
  <URL>                       URL to download through Chrome
  <PATH>                      Destination file path

Options:
  --link <ELEMENT_ID>          Download the href from a captured link element in the current session
  --output <PATH>              Destination file path for --link mode
```

Examples:

```bash
# download through the current browser session
bowser download --session bsr_01HZX... https://example.com/report.csv report.csv

# download through a fresh session, then detach
bowser download https://example.com/report.csv report.csv

# download a link from the current captured page
bowser download --session bsr_01HZX... --link 12 --output report.csv
```

The download path must name a file whose parent directory already exists.
Downloads are browser-native: redirects, cookies, authentication state, and
`Content-Disposition` are handled by Chrome rather than by a separate HTTP
client.

The current local CLI does not include a `--proxy` flag. The future remote CLI
selects managed egress through the hosted API rather than passing proxy
credentials to the local browser process; see
[Managed Dashboard And CLI](./managed-dashboard-and-cli.md).

### Google Smoke Flow

The Google captcha regression flow uses the default stealth mode:

```bash
bowser --no-ai -i https://www.google.com
BOWSER_GOOGLE_SMOKE=1 cargo test -p bowser-cli --test interactive_repl google_search_flow_avoids_captcha -- --ignored --nocapture
```

Default stealth uses Bowser's full JS DOM capture, backend-node focus for text
entry where possible, Enter submission for search forms, headed Chrome without
a forced window size, and CDP Runtime-domain events left enabled. The durable
source of truth is the live result ledger in
[`google-smoke-results.md`](./google-smoke-results.md), which records both the
direct gstack-style control and the Bowser-native control for the same test
window.

Routine automated CLI browser tests do not run headed. Their shared helpers set
`BOWSER_HEADLESS=true`, keep stealth enabled, and disable the headed launch
features through
`BOWSER_INTERNAL_STEALTH_FEATURES=-launch-headed,-launch-native-window`. The
ignored live Google smoke is the only CLI test that passes `--headed`, and it
requires the explicit `BOWSER_GOOGLE_SMOKE=1` opt-in above.

### Headed Container Behavior

`--headed` launches visible Chrome. On Linux, if neither `DISPLAY` nor
`WAYLAND_DISPLAY` is set, Bowser starts Xvfb on a free display and stores the
Xvfb process metadata with the browser session so `session close` and expiry
cleanup can validate the stored display before terminating it safely.

#### `bowser interactive [URL]`

Interactive REPL mode. Optionally starts at a URL. Exiting the REPL detaches from the browser and prints the resumable session ID to `stderr`.

Shortcut form: `bowser -i [URL]`

```
bowser interactive [OPTIONS] [URL]

bowser -i [OPTIONS] [URL]

Arguments:
  [URL]                      Optional starting URL

Options:
  --wait <SELECTOR>          Wait for selector before first capture
  --wait-timeout <SECONDS>   Max time to wait [default: 10]
```

Example:

```bash
# start an interactive session and note the printed session ID on exit
bowser interactive https://example.com

# shorthand entry point
bowser -i https://example.com

# resume that same session later
bowser interactive --session bsr_01HZX...
```

#### `bowser page <SUBCOMMAND>`

Page management for a detached session with multiple pages.

```
bowser page <SUBCOMMAND>

Subcommands:
  list                       List known pages in the detached session
  select <PAGE_ID>          Select a live page and print its capture
  new [URL]                 Open a new page, optionally navigating immediately
  close [PAGE_ID]           Close the current selected page, or the given page ID

Options for select/new/close:
  --format <FORMAT>          Output format: yaml | json | html [default: yaml]
```

Page IDs are stable session-scoped identifiers such as `pg_1`.

Examples:

```bash
# list pages in a detached session
bowser page list --session bsr_01HZX...

# switch to a different page and print its YAML
bowser page select --session bsr_01HZX... pg_2

# open a new page and navigate it immediately
bowser page new --session bsr_01HZX... https://example.com/dashboard

# close the selected page and print the replacement page capture
bowser page close --session bsr_01HZX...
```

#### `bowser pointer-log`

Local-only pointer telemetry capture page. The command must bind only to a
loopback address, show one randomized target element at a time, replace the
target after it is clicked, and append accepted events to a local JSONL file.
Manual mode defaults to the repo-root `browser-log`, which is gitignored
because it contains local capture data. Stored records must omit session IDs
and user-agent strings. Each new randomized target placement must also append a
`target_spawn` record with the target's viewport rect. The page must render a
visible cursor trail and click pulse from received pointer events so CDP-driven
mouse automation is observable in a headed browser.

Demo mode starts the same loopback server, launches a headed Bowser browser
session, opens the capture page, and repeatedly clicks the randomized target by
using the library's real `PageEngine::click` path. Demo mode runs until
`Ctrl-C` unless `--demo-clicks <N>` is provided. Demo mode must only persist
events when `--output` is provided; without `--output`, the `/events` route
accepts batches without writing a file.

```
bowser pointer-log [OPTIONS]

Options:
  --bind <ADDR>              Loopback bind address [default: 127.0.0.1:8765]
  --output <PATH>            JSONL event log
  --demo                     Drive the page with headed Bowser mouse automation
  --demo-clicks <N>          Stop demo mode after N automated clicks
```

Example:

```bash
bowser pointer-log --output .context/pointer-events.jsonl
bowser pointer-log --demo --demo-clicks 10
bowser pointer-log --demo --output .context/pointer-demo.jsonl
```

#### `bowser session <SUBCOMMAND>`

Session lifecycle management.

```
bowser session <SUBCOMMAND>

Subcommands:
  list                       List detached sessions
  info <SESSION_ID>          Show detached-session metadata, including selected page and page summaries
  close <SESSION_ID>         Close a detached session, remove its metadata, and remove any Bowser-owned ephemeral profile
```
