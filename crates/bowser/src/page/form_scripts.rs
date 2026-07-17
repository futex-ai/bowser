//! Browser-side scripts for form and scroll actions.

pub(super) fn select_option_expression(element_id: u32, value_json: &str) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  if (!(el instanceof HTMLSelectElement)) return {{ ok: false, kind: 'not_interactable' }};
  const option = Array.from(el.options).find((candidate) => candidate.text === {value_json});
  if (!option) return {{ ok: false, kind: 'not_interactable' }};
  el.value = option.value;
  el.dispatchEvent(new Event('input', {{ bubbles: true }}));
  el.dispatchEvent(new Event('change', {{ bubbles: true }}));
  return {{ ok: true }};
}})()
"#
    )
}

pub(super) fn submit_plan_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  const form = el instanceof HTMLFormElement ? el : (el.form || el.closest('form'));
  if (!form) return {{ ok: false, kind: 'not_interactable' }};
  const focusTarget = (target) => {{
    if (!target) return;
    target.scrollIntoView({{ block: 'center', inline: 'center' }});
    if (typeof target.focus === 'function') {{
      try {{
        target.focus({{ preventScroll: true }});
      }} catch (_error) {{
        target.focus();
      }}
    }}
  }};
  const submitStrategy = (target) => {{
    if (!target || !target.tagName) return null;
    const tag = target.tagName.toLowerCase();
    const type = tag === 'input'
      ? (target.getAttribute('type') || 'text').toLowerCase()
      : '';
    if (tag === 'button' || (tag === 'input' && ['submit', 'button', 'image'].includes(type))) {{
      return 'click_self';
    }}
    if (tag === 'input' && !['button', 'submit', 'reset', 'file', 'checkbox', 'radio', 'range', 'color', 'hidden'].includes(type)) {{
      return 'enter';
    }}
    return null;
  }};
  const direct = submitStrategy(el);
  if (direct === 'click_self') return {{ ok: true, strategy: direct }};
  if (direct === 'enter') {{
    focusTarget(el);
    return {{ ok: true, strategy: direct }};
  }}
  const active = document.activeElement;
  if (active && form.contains(active) && submitStrategy(active) === 'enter') {{
    focusTarget(active);
    return {{ ok: true, strategy: 'enter' }};
  }}
  const fallback = Array.from(form.elements || []).find((candidate) => submitStrategy(candidate) === 'enter');
  if (fallback) {{
    focusTarget(fallback);
    return {{ ok: true, strategy: 'enter' }};
  }}
  return {{ ok: true, strategy: 'dom' }};
}})()
"#
    )
}

pub(super) fn submit_dom_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  const form = el instanceof HTMLFormElement ? el : (el.form || el.closest('form'));
  if (!form) return {{ ok: false, kind: 'not_interactable' }};
  if (form.requestSubmit) form.requestSubmit();
  else form.submit();
  return {{ ok: true }};
}})()
"#
    )
}

pub(super) fn scroll_to_element_expression(element_id: u32) -> String {
    format!(
        r#"
(() => {{
  const el = window.__bowserElements && window.__bowserElements.get({element_id});
  if (!el) return {{ ok: false, kind: 'not_found' }};
  el.scrollIntoView({{ block: 'center', inline: 'center' }});
  return {{ ok: true }};
}})()
"#
    )
}
