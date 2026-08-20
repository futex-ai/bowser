//! Clap argument types.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

use crate::format::{PageFormat, StructuredFormat};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum AiProviderArg {
    Anthropic,
    Openai,
    Ollama,
}

#[derive(Debug, Parser)]
#[command(name = "bowser")]
#[command(about = "Render web pages into compact YAML")]
pub(crate) struct Cli {
    #[arg(long, global = true)]
    pub(crate) chrome_path: Option<PathBuf>,
    #[arg(long, global = true, value_delimiter = ',')]
    pub(crate) chrome_args: Vec<String>,
    #[arg(short = 'i', long = "interactive", global = true, action = ArgAction::SetTrue)]
    pub(crate) interactive: bool,
    #[arg(long, global = true)]
    pub(crate) session: Option<String>,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) headed: bool,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) persistent_profile: bool,
    #[arg(long, global = true)]
    pub(crate) user_data_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) session_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    pub(crate) session_ttl: Option<u64>,
    #[arg(long, global = true, value_parser = parse_viewport)]
    pub(crate) viewport: Option<bowser::Viewport>,
    #[arg(long, global = true)]
    pub(crate) timeout: Option<u64>,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) no_truncate: bool,
    #[arg(short = 'a', long = "all", global = true, action = ArgAction::SetTrue)]
    pub(crate) all: bool,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) no_stealth: bool,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) no_ai: bool,
    #[arg(long, global = true, value_enum)]
    pub(crate) ai_provider: Option<AiProviderArg>,
    #[arg(long, global = true)]
    pub(crate) ai_model: Option<String>,
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub(crate) json_envelope: bool,
    #[arg(short = 'v', long, global = true, action = ArgAction::Count)]
    pub(crate) verbose: u8,
    #[arg(short = 'q', long, global = true, action = ArgAction::SetTrue)]
    pub(crate) quiet: bool,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
    pub(crate) url: Option<String>,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Command {
    Get(GetArgs),
    Capture(CaptureArgs),
    Download(DownloadArgs),
    Expand(ExpandArgs),
    Meta(MetaArgs),
    Describe(DescribeArgs),
    Click(ElementActionArgs),
    Type(TypeArgs),
    Clear(ElementActionArgs),
    Submit(ElementActionArgs),
    Key(KeyArgs),
    Scroll(ScrollArgs),
    Back(HistoryArgs),
    Forward(HistoryArgs),
    Reload(HistoryArgs),
    Interactive(InteractiveArgs),
    PointerLog(PointerLogArgs),
    Page {
        #[command(subcommand)]
        command: PageSubcommand,
    },
    Session {
        #[command(subcommand)]
        command: SessionSubcommand,
    },
    Capabilities,
}

#[derive(Clone, Debug, clap::Args)]
pub struct GetArgs {
    pub url: String,
    #[arg(long)]
    pub wait: Option<String>,
    #[arg(long, default_value_t = 10)]
    pub wait_timeout: u64,
    #[arg(long, default_value_t = 0)]
    pub delay: u64,
    #[arg(long)]
    pub screenshot: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct CaptureArgs {
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct DownloadArgs {
    #[arg(long, conflicts_with = "url")]
    pub link: Option<u32>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    pub url: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ExpandArgs {
    pub element_id: u32,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = StructuredFormat::Yaml)]
    pub format: StructuredFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct MetaArgs {
    pub element_id: u32,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = StructuredFormat::Yaml)]
    pub format: StructuredFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct DescribeArgs {
    pub element_id: u32,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = StructuredFormat::Yaml)]
    pub format: StructuredFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ElementActionArgs {
    pub element_id: u32,
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct TypeArgs {
    pub element_id: u32,
    pub text: String,
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct KeyArgs {
    pub key: String,
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct ScrollArgs {
    #[arg(long)]
    pub direction: Option<String>,
    #[arg(long)]
    pub element_id: Option<u32>,
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct HistoryArgs {
    #[arg(long)]
    pub page_id: Option<String>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
    pub format: PageFormat,
}

#[derive(Clone, Debug, clap::Args)]
pub struct InteractiveArgs {
    pub url: Option<String>,
    #[arg(long)]
    pub wait: Option<String>,
    #[arg(long, default_value_t = 10)]
    pub wait_timeout: u64,
}

/// Arguments for the local pointer telemetry capture server.
#[derive(Clone, Debug, clap::Args)]
pub struct PointerLogArgs {
    /// Loopback bind address for the local capture page.
    #[arg(long, default_value = "127.0.0.1:8765")]
    pub bind: SocketAddr,
    /// JSONL file to append captured pointer events to.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Launch a headed Bowser session that drives the pointer-log page.
    #[arg(long, action = ArgAction::SetTrue)]
    pub demo: bool,
    /// Stop the demo after this many automated clicks.
    #[arg(long, requires = "demo")]
    pub demo_clicks: Option<u32>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum PageSubcommand {
    List,
    Select {
        page_id: String,
        #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
        format: PageFormat,
    },
    New {
        url: Option<String>,
        #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
        format: PageFormat,
    },
    Close {
        page_id: Option<String>,
        #[arg(long, value_enum, default_value_t = PageFormat::Yaml)]
        format: PageFormat,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum SessionSubcommand {
    List,
    Info {
        session_id: String,
    },
    Close {
        session_id: String,
    },
    Export {
        #[arg(long)]
        to: PathBuf,
    },
    Restore {
        #[arg(long)]
        from: PathBuf,
    },
}

pub(crate) fn parse_viewport(value: &str) -> std::result::Result<bowser::Viewport, String> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| "expected WIDTHxHEIGHT".to_string())?;
    let width = width.parse().map_err(|_| "invalid width".to_string())?;
    let height = height.parse().map_err(|_| "invalid height".to_string())?;
    if width == 0 || height == 0 {
        return Err("width and height must be greater than zero".to_string());
    }
    Ok(bowser::Viewport { width, height })
}
