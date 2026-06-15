use std::fmt;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use codex_api::ImageBackground;
use codex_api::ImageQuality;

pub(crate) const DEFAULT_IMAGE_MODEL: &str = "gpt-image-2";
pub(crate) const MAX_EDIT_IMAGES: usize = 5;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(long, global = true, value_name = "DIR")]
    pub codex_home: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Generate(GenerateArgs),
    Edit(EditArgs),
}

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
pub struct GenerateArgs {
    #[arg(long)]
    pub prompt: String,

    #[arg(long, value_name = "PATH")]
    pub out: PathBuf,

    #[arg(long, default_value = DEFAULT_IMAGE_MODEL)]
    pub model: String,

    #[arg(long, default_value_t = BackgroundArg::Auto)]
    pub background: BackgroundArg,

    #[arg(long, default_value_t = QualityArg::Auto)]
    pub quality: QualityArg,

    #[arg(long, default_value = "auto")]
    pub size: String,

    #[arg(long)]
    pub n: Option<u64>,

    #[arg(long)]
    pub transport: Option<TransportArg>,
}

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
pub struct EditArgs {
    #[arg(long = "image", value_name = "PATH", required = true)]
    pub images: Vec<PathBuf>,

    #[arg(long)]
    pub prompt: String,

    #[arg(long, value_name = "PATH")]
    pub out: PathBuf,

    #[arg(long, default_value = DEFAULT_IMAGE_MODEL)]
    pub model: String,

    #[arg(long, default_value_t = BackgroundArg::Auto)]
    pub background: BackgroundArg,

    #[arg(long, default_value_t = QualityArg::Auto)]
    pub quality: QualityArg,

    #[arg(long, default_value = "auto")]
    pub size: String,

    #[arg(long)]
    pub n: Option<u64>,

    #[arg(long)]
    pub transport: Option<TransportArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BackgroundArg {
    Auto,
    Opaque,
    Transparent,
}

impl fmt::Display for BackgroundArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Opaque => "opaque",
            Self::Transparent => "transparent",
        })
    }
}

impl From<BackgroundArg> for ImageBackground {
    fn from(value: BackgroundArg) -> Self {
        match value {
            BackgroundArg::Auto => Self::Auto,
            BackgroundArg::Opaque => Self::Opaque,
            BackgroundArg::Transparent => Self::Transparent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum QualityArg {
    Auto,
    Low,
    Medium,
    High,
}

impl fmt::Display for QualityArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        })
    }
}

impl From<QualityArg> for ImageQuality {
    fn from(value: QualityArg) -> Self {
        match value {
            QualityArg::Auto => Self::Auto,
            QualityArg::Low => Self::Low,
            QualityArg::Medium => Self::Medium,
            QualityArg::High => Self::High,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TransportArg {
    CodexHosted,
    ImageApi,
}

impl fmt::Display for TransportArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CodexHosted => "codex-hosted",
            Self::ImageApi => "image-api",
        })
    }
}
