//! Installed Bowser capability handshake.

use serde::Serialize;

use crate::error::Result;
use crate::output::CommandOutput;

#[derive(Debug, Serialize)]
struct Capabilities {
    version: &'static str,
    envelope_versions: [u32; 1],
    checkpoint_versions: [u32; 1],
    features: Features,
}

#[derive(Debug, Serialize)]
struct Features {
    checkpoint: bool,
    window_target: bool,
}

pub(crate) fn run() -> Result<CommandOutput> {
    let capabilities = Capabilities {
        version: env!("CARGO_PKG_VERSION"),
        envelope_versions: [1],
        checkpoint_versions: [bowser::CHECKPOINT_VERSION],
        features: Features {
            checkpoint: true,
            window_target: false,
        },
    };
    let human = serde_json::to_string_pretty(&capabilities)
        .map_err(|source| crate::error::CliError::OutputSerialize { source })?;
    Ok(CommandOutput::result(capabilities)?.human_stdout(format!("{human}\n")))
}
