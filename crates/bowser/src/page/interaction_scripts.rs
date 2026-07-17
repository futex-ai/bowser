//! Browser-side scripts for element interaction.

pub(super) fn interaction_target_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return null;
  const compute = () => {{
    const style = window.getComputedStyle(el);
    const rect = el.getBoundingClientRect();
    const rendered =
      style.display !== 'none' &&
      style.visibility !== 'hidden' &&
      Number(style.opacity || '1') > 0 &&
      rect.width > 0 &&
      rect.height > 0;
    const inViewport =
      rendered &&
      rect.bottom > 0 &&
      rect.right > 0 &&
      rect.top < window.innerHeight &&
      rect.left < window.innerWidth;
    const unit = () => {{
      if (window.crypto && typeof window.crypto.getRandomValues === 'function') {{
        const values = new Uint32Array(1);
        window.crypto.getRandomValues(values);
        return values[0] / 4294967296;
      }}
      return Math.random();
    }};
    const coordinate = (start, end, viewportLimit) => {{
      const center = start + (end - start) / 2;
      const low = Math.min(start + 10, center);
      const high = Math.max(end - 10, center);
      const maxViewport = Math.max(viewportLimit - 1, 0);
      const min = Math.min(Math.max(low, 0), maxViewport);
      const max = Math.min(Math.max(high, 0), maxViewport);
      if (max <= min) return min;
      return min + unit() * (max - min);
    }};
    const x = coordinate(rect.left, rect.right, window.innerWidth);
    const y = coordinate(rect.top, rect.bottom, window.innerHeight);
    let obscured = false;
    if (inViewport && window.innerWidth > 0 && window.innerHeight > 0) {{
      const top = document.elementFromPoint(x, y);
      obscured = !!top && top !== el && !el.contains(top) && !top.contains(el);
    }}
    const enabled =
      (!('disabled' in el) || !el.disabled) &&
      el.getAttribute('aria-disabled') !== 'true';
    const pointerEnabled = style.pointerEvents !== 'none';
    const tag = el.tagName ? el.tagName.toLowerCase() : '';
    const type = tag === 'input'
      ? (el.getAttribute('type') || 'text').toLowerCase()
      : tag;
    const textEntryTypes = new Set([
      'text',
      'password',
      'email',
      'number',
      'tel',
      'url',
      'search',
      'date',
      'textarea'
    ]);
    const focusAfterClick = textEntryTypes.has(type) || el.isContentEditable === true;
    return {{
      x,
      y,
      viewport_width: window.innerWidth,
      viewport_height: window.innerHeight,
      visible: rendered && inViewport && !obscured,
      pointer_enabled: pointerEnabled,
      enabled,
      focus_after_click: focusAfterClick
    }};
  }};
  let target = compute();
  if (!target.visible) {{
    el.scrollIntoView({{ block: 'center', inline: 'center' }});
    target = compute();
  }}
  return target;
}})()
"#
    )
}

pub(super) fn focus_after_pointer_click_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  const tag = el.tagName ? el.tagName.toLowerCase() : '';
  const type = tag === 'input' ? (el.getAttribute('type') || 'text').toLowerCase() : tag;
  const textEntryTypes = new Set(['text', 'password', 'email', 'number', 'tel', 'url', 'search', 'date', 'textarea']);
  if (!textEntryTypes.has(type) && el.isContentEditable !== true) return {{ ok: true }};
  const doc = el.ownerDocument || document;
  if (doc.activeElement === el || typeof el.focus !== 'function') return {{ ok: true }};
  try {{
    el.focus({{ preventScroll: true }});
  }} catch (_error) {{
    el.focus();
  }}
  return {{ ok: true }};
}})()
"#
    )
}

pub(super) fn synthetic_click_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  if (typeof el.click !== 'function') return {{ ok: false, kind: 'not_interactable' }};
  const focusTarget = () => {{
    if (typeof el.focus !== 'function') return;
    try {{
      el.focus({{ preventScroll: true }});
    }} catch (_error) {{
      el.focus();
    }}
  }};
  el.scrollIntoView({{ block: 'center', inline: 'center' }});
  focusTarget();
  el.click();
  focusTarget();
  return {{ ok: true }};
}})()
"#
    )
}

pub(super) fn viewport_bounds_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return null;
  el.scrollIntoView({{ block: 'center', inline: 'center' }});
  const rect = el.getBoundingClientRect();
  return {{
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: rect.height
  }};
}})()
"#
    )
}

pub(super) fn runtime_state_expression(element_id: u32, page_space_bounds: bool) -> String {
    let left = if page_space_bounds {
        "rect.left + window.scrollX"
    } else {
        "rect.left"
    };
    let top = if page_space_bounds {
        "rect.top + window.scrollY"
    } else {
        "rect.top"
    };
    let right = if page_space_bounds {
        "rect.right + window.scrollX"
    } else {
        "rect.right"
    };
    let bottom = if page_space_bounds {
        "rect.bottom + window.scrollY"
    } else {
        "rect.bottom"
    };
    format!(
        r#"
(() => {{
  const missing = {{
    present: false,
    in_viewport: false,
    obscured: false,
    enabled: false,
    visible: false,
    clickable: false,
    focused: false,
    bounds: null
  }};
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el || !el.isConnected) return missing;
  const doc = el.ownerDocument || document;
  const style = window.getComputedStyle(el);
  const rect = el.getBoundingClientRect();
  const rendered =
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    Number(style.opacity || '1') > 0 &&
    rect.width > 0 &&
    rect.height > 0;
  const inViewport =
    rendered &&
    rect.bottom > 0 &&
    rect.right > 0 &&
    rect.top < window.innerHeight &&
    rect.left < window.innerWidth;
  let obscured = false;
  if (inViewport && window.innerWidth > 0 && window.innerHeight > 0) {{
    const x = Math.min(
      Math.max(rect.left + rect.width / 2, 0),
      window.innerWidth - 1
    );
    const y = Math.min(
      Math.max(rect.top + rect.height / 2, 0),
      window.innerHeight - 1
    );
    const topElement = document.elementFromPoint(x, y);
    obscured = !!topElement &&
      topElement !== el &&
      !el.contains(topElement) &&
      !topElement.contains(el);
  }}
  const enabled =
    (!('disabled' in el) || !el.disabled) &&
    el.getAttribute('aria-disabled') !== 'true';
  const pointerEnabled = style.pointerEvents !== 'none';
  const visible = rendered && inViewport && !obscured;
  const clickable = visible && enabled && pointerEnabled;
  const focused = doc.activeElement === el;
  const bounds = {{
    top_left: [{left}, {top}],
    bottom_right: [{right}, {bottom}]
  }};
  return {{
    present: true,
    in_viewport: inViewport,
    obscured,
    enabled,
    visible,
    clickable,
    focused,
    bounds
  }};
}})()
"#
    )
}

pub(super) fn image_png_expression(element_id: u32) -> String {
    format!(
        r#"
(async () => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  if (!(el instanceof HTMLImageElement)) return {{ ok: false, kind: 'not_image' }};
  if (!el.complete || !el.naturalWidth || !el.naturalHeight) {{
    await new Promise((resolve) => {{
      const done = () => resolve();
      el.addEventListener('load', done, {{ once: true }});
      el.addEventListener('error', done, {{ once: true }});
      setTimeout(done, 3000);
    }});
  }}
  if (!el.naturalWidth || !el.naturalHeight) {{
    return {{ ok: false, kind: 'image_not_loaded' }};
  }}
  const rect = el.getBoundingClientRect();
  const width = Math.max(1, Math.round(rect.width || el.naturalWidth));
  const height = Math.max(1, Math.round(rect.height || el.naturalHeight));
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) return {{ ok: false, kind: 'canvas_unavailable' }};
  try {{
    context.drawImage(el, 0, 0, width, height);
    return {{ ok: true, data_url: canvas.toDataURL('image/png') }};
  }} catch (_error) {{
    return {{ ok: false, kind: 'canvas_failed' }};
  }}
}})()
"#
    )
}
