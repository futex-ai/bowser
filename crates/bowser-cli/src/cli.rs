//! CLI command resolution and config overrides.

use crate::args::{AiProviderArg, Cli, Command, GetArgs, InteractiveArgs};
use crate::error::{CliError, Result};
use crate::format::PageFormat;

pub(crate) fn resolve_command(cli: Cli) -> Result<Command> {
    if let Some(command) = cli.command {
        return Ok(command);
    }
    if cli.interactive {
        return Ok(Command::Interactive(InteractiveArgs {
            url: cli.url,
            wait: None,
            wait_timeout: 10,
        }));
    }
    if let Some(url) = cli.url {
        return Ok(Command::Get(GetArgs {
            url,
            wait: None,
            wait_timeout: 10,
            delay: 0,
            screenshot: None,
            output: None,
            format: PageFormat::Yaml,
        }));
    }
    Err(CliError::InvalidCommand {
        input: "missing command or URL".to_string(),
    })
}

pub(crate) fn config_overrides(cli: &Cli) -> bowser::ConfigOverrides {
    bowser::ConfigOverrides {
        chrome_path: cli.chrome_path.clone(),
        chrome_args: if cli.chrome_args.is_empty() {
            None
        } else {
            Some(cli.chrome_args.clone())
        },
        session_id: cli.session.clone(),
        session_dir: cli.session_dir.clone(),
        session_ttl_secs: cli.session_ttl,
        headless: if cli.headed { Some(false) } else { None },
        persistent_profile: if cli.persistent_profile {
            Some(true)
        } else {
            None
        },
        user_data_dir: cli.user_data_dir.clone(),
        viewport: cli.viewport.clone(),
        timeout_secs: cli.timeout,
        truncate: if cli.all || cli.no_truncate {
            Some(false)
        } else {
            None
        },
        include_hidden: None,
        max_children: None,
        max_list_items: None,
        max_table_rows: None,
        stealth: if cli.no_stealth { Some(false) } else { None },
        disable_runtime_events: None,
        ai_enabled: if cli.no_ai { Some(false) } else { None },
        ai_provider: cli.ai_provider.map(map_ai_provider),
        ai_model: cli.ai_model.clone(),
        ai_api_key_env: None,
        ai_endpoint: None,
    }
}

fn map_ai_provider(provider: AiProviderArg) -> bowser::AiProvider {
    match provider {
        AiProviderArg::Anthropic => bowser::AiProvider::Anthropic,
        AiProviderArg::Openai => bowser::AiProvider::OpenAi,
        AiProviderArg::Ollama => bowser::AiProvider::Ollama,
    }
}
