//! Sensitive path classification for local AI review prompts.

use std::path::{Component, Path};

/// Return whether a tracked path should have its diff body omitted.
pub(crate) fn is_sensitive_tracked_relative_path(relative_path: &str) -> bool {
    let components = Path::new(relative_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();
    components.iter().enumerate().any(|(index, component)| {
        is_explicit_sensitive_component(component)
            || (is_broad_sensitive_component(component)
                && (index + 1 < components.len() || is_sensitive_data_file_name(component)))
    })
}

/// Return whether a tracked path should be omitted from AI review context.
pub(crate) fn is_omitted_tracked_relative_path(relative_path: &str) -> bool {
    is_sensitive_tracked_relative_path(relative_path)
        || is_large_mockup_artifact_relative_path(relative_path)
}

/// Return whether an untracked path should be omitted before reading contents.
pub(crate) fn is_sensitive_untracked_relative_path(relative_path: &str) -> bool {
    Path::new(relative_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .any(|component| {
            is_explicit_sensitive_component(component) || is_broad_sensitive_component(component)
        })
}

/// Return whether an untracked path should be omitted from AI review context.
pub(crate) fn is_omitted_untracked_relative_path(relative_path: &str) -> bool {
    is_sensitive_untracked_relative_path(relative_path)
        || is_large_mockup_artifact_relative_path(relative_path)
}

fn is_large_mockup_artifact_relative_path(relative_path: &str) -> bool {
    let path = Path::new(relative_path);
    path.starts_with("docs/mockups")
        && !path.starts_with("docs/mockups/src")
        && path.extension().and_then(|value| value.to_str()) == Some("html")
}

fn is_explicit_sensitive_component(component: &str) -> bool {
    let component = component.to_ascii_lowercase();
    let file_name = component.as_str();

    matches!(
        file_name,
        ".env"
            | ".envrc"
            | ".npmrc"
            | ".pypirc"
            | ".netrc"
            | ".ssh"
            | ".aws"
            | ".azure"
            | ".gcloud"
            | ".kube"
            | "id_rsa"
            | "id_dsa"
            | "id_ecdsa"
            | "id_ed25519"
            | "credentials"
            | "secrets"
            | "private_key"
            | "private-key"
    ) || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || is_sensitive_config_file_name(file_name)
}

fn is_broad_sensitive_component(component: &str) -> bool {
    let component = component.to_ascii_lowercase();
    let file_name = component.as_str();

    file_name.contains("secret")
        || file_name.contains("password")
        || file_name.contains("credential")
        || file_name.contains("token")
}

fn is_sensitive_data_file_name(file_name: &str) -> bool {
    let path = Path::new(file_name);
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return true;
    };

    matches!(
        extension.to_ascii_lowercase().as_str(),
        "env" | "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "properties" | "txt"
    )
}

fn is_sensitive_config_file_name(file_name: &str) -> bool {
    let Some((base_name, _)) = file_name.split_once('.') else {
        return false;
    };
    let Some((_, extension)) = file_name.rsplit_once('.') else {
        return false;
    };

    !base_name.is_empty()
        && matches!(
            base_name,
            "credential"
                | "credentials"
                | "secret"
                | "secrets"
                | "private_key"
                | "private-key"
                | "token"
                | "tokens"
                | "password"
                | "passwords"
        )
        && matches!(
            extension,
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "properties" | "txt"
        )
}
