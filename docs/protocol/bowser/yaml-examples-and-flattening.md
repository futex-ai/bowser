### Compact YAML Output Example

```yaml
url: "https://news.ycombinator.com"
title: "Hacker News"
content:
  visible:
    - nav#1:
        - link#2: "new"
        - link#3: "past"
        - link#4: "comments"
    - main#5:
        - article#6:
            - link#7: "Show HN: A cool Rust project"
            - text: "120 points by user1 3 hours ago"
            - link#8: "145 comments"
    - form#9:
        action: "/search"
        content:
          - input#10: { type: text, name: q, placeholder: Search, value: "" }
          - input: { type: hidden, name: csrf_token, value: "csrf-token-123" }
          - button#11: "Search"
  obscured:
    - link#12: "Secret admin link"
```

### Truncated Collection Example

```yaml
url: "https://example.com/catalog"
title: "Catalog"
content:
  visible:
    - ul#1:
        items:
          - article#2:
              - image#3: "Blue running shoes"
              - link#4: "Blue Runner Pro"
              - text: "$129.99"
          - article#6:
              - image#7: "Red hiking boots"
              - link#8: "Trail Master X"
              - text: "$189.99"
        truncated: { shown: 2, total: 124 }
    - table#10:
        headers:
          - text: "Field"
          - text: "Value"
        rows:
          - cells:
              - text: "Coupon"
              - input#14: { type: text, name: coupon, placeholder: "Enter code", value: "" }
        truncated: { shown: 1, total: 47 }
  obscured: []
```

### Obscured Modal Example

```yaml
url: "https://example.com/search"
title: "Search"
content:
  visible:
    - dialog#1:
        - h2: "Before you continue"
        - text: "We use cookies and data to keep the service secure."
        - button#2: "Reject all"
        - button#3: "Accept all"
  obscured:
    - header#4:
        - link#5: "About"
    - main#6:
        - h1: "Background Search"
        - form#7:
            action: "/search"
            content:
              - input#8: { type: text, name: q, placeholder: Search, value: "" }
```

### Truncated Root Preview Example

```yaml
url: "https://example.com/orders"
title: "My orders"
content:
  visible:
    - body#42:
        content:
          - header#1:
              - link#2: "Orders"
              - link#3: "Account"
          - nav#4:
              - link#5: "Home"
              - link#6: "My orders"
          - h1: "My orders"
          - text: "Amend, view or cancel your scheduled orders, or view your previous orders."
        truncated: { shown: 4, total: 10 }
  obscured: []
```

### Flattening Rules

The DOM-to-YAML transformation applies these rules:

1. **Split non-rendered semantic content**: anything with `display: none`, `visibility: hidden`, or zero rendered dimensions goes under `content.obscured` when it produces semantic content, except `<input type="hidden">`, which stays inside its owning form.
2. **Flatten wrapper divs**: a `<div>` (or `<span>` used as a wrapper) that has no semantic role, no text content of its own, and only contains other elements — its children are promoted to the parent.
3. **Merge adjacent text**: consecutive text nodes or inline elements containing only text are merged into a single `text` element.
4. **Preserve semantic containers**: `<nav>`, `<form>`, `<section>`, `<article>`, `<main>`, `<header>`, `<footer>`, `<aside>`, dialog-like containers, and table structure (`<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>`) are always preserved as nesting boundaries.
5. **Promote single-child wrappers**: if a non-semantic element contains exactly one child, the child replaces the parent.
6. **Prefer rendered text**: element-level text fallback uses rendered text (`innerText`-style semantics), not raw DOM `textContent`, so embedded CSS/JS markup is not surfaced as page text.
7. **Trim whitespace**: all text content is trimmed. Elements that become empty after trimming are dropped.
8. **Collapse deep nesting**: if multiple levels of non-semantic elements nest without adding content or meaning, they collapse to a single level.
9. **Do not flatten tables**: table headers and body rows preserve cell boundaries, and inputs inside table cells remain nested within those cells.
10. **Skip non-rendered markup nodes**: `<script>`, `<style>`, `<noscript>`, and `<template>` do not contribute capture output.
11. **Split blocking overlays from obscured background content**: when a modal/backdrop obscures the underlying page, capture the visible dialog subtree under `content.visible` and the background under `content.obscured`.
12. **Truncate long collections by preview**: when a collection exceeds the configured preview limit, retain the leading entries and attach `truncated` metadata to that element.
13. **Capture the full rendered document, not just the viewport**: rendered content below the fold remains eligible for capture. If truncation affects the top-level page preview, expose it as a synthetic `body#<ID>` container so omitted root entries are explicit and expandable.

`-a` / `--all` disables truncation for that command. Visibility and occlusion pruning are not needed for root content because hidden and obscured semantic content is emitted under `content.obscured` by default.
