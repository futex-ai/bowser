//! Stability detection scripts.

/// Returns the polling expression for a compact page stability sample.
pub fn sample_expression(navigation: bool) -> String {
    let ready_clause = if navigation {
        "document.readyState === 'complete'"
    } else {
        "true"
    };
    format!(
        r#"
(() => {{
  const root = document.documentElement;
  const signature = [
    root ? root.innerHTML.length : 0,
    root ? root.getElementsByTagName('*').length : 0,
    document.title ? document.title.length : 0
  ].join(':');
  return {{
    ready: {ready_clause},
    signature
  }};
}})()
"#,
        ready_clause = ready_clause
    )
}
