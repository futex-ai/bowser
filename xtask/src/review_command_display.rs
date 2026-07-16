//! Shell command display helpers for review diagnostics.

/// Format a command for error messages.
pub(crate) fn command_display(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_owned())
        .chain(args.iter().map(|arg| shell_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format stderr for appending to a status error.
pub(crate) fn stderr_string(stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return String::new();
    }

    format!(": {stderr}")
}

fn shell_arg(value: &str) -> String {
    if !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b'=')
        })
    {
        return value.to_owned();
    }

    format!("'{}'", value.replace('\'', r#"'\''"#))
}
