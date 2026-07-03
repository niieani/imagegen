use std::fmt;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use codex_api::ImageBackground;
use codex_api::ImageQuality;

pub(crate) const DEFAULT_IMAGE_MODEL: &str = "gpt-image-2";
pub(crate) const MAX_EDIT_IMAGES: usize = 5;

const CLI_LONG_ABOUT: &str = "\
Codex-authenticated OpenAI image generation.

Use generate for text-to-image. Use edit when existing images should be changed,
combined, or used as references for a new output.

Default model: gpt-image-2. It handles photorealism, compositing, and dense
text-in-image prompts well. For newspapers, UI screenshots, signs, labels, or
posters, include ALL required text in the prompt; it is very likely to render
that provided text correctly.";

const CLI_AFTER_LONG_HELP: &str = "\
gpt-image-2 output notes:
  Sizes: auto, 1024x1024, 1536x1024, 1024x1536, 2048x2048,
         2048x1152, 3840x2160, 2160x3840, or any WIDTHxHEIGHT that
         satisfies: max edge <=3840px, both edges multiples of 16px,
         long:short ratio <=3:1, total pixels 655360..8294400.
  Quality: auto, low, medium, high. Use low for fast drafts.
  Background: auto or opaque for gpt-image-2. background=transparent is not
         supported by gpt-image-2.
  Latency: complex, high-quality, or 4K requests can take up to 2 minutes.";

const GENERATE_LONG_ABOUT: &str = "\
Generate a new image from a text prompt.

For text-heavy outputs, include ALL exact text in quotes, describe typography
and placement, and use medium or high quality for dense layouts. gpt-image-2 can
generate newspaper pages or screenshots with all prompt-provided text and is
very likely to get it right.";

const EDIT_LONG_ABOUT: &str = "\
Edit one or more input images, or use them as references for a new output.

Use repeated --image flags. In the prompt, describe each image by role when
there are multiple inputs, then state what should change and what must be
preserved.";

const EDIT_AFTER_LONG_HELP: &str = "\
Edit prompting:
  Good: \"Image 1 is the product. Image 2 is the style reference. Put the
  product from Image 1 into the lighting style of Image 2. Keep the label text
  unchanged.\"";

const SIZE_LONG_HELP: &str = "\
Output size.

For gpt-image-2, use auto or WIDTHxHEIGHT. Popular sizes: 1024x1024,
1536x1024, 1024x1536, 2048x2048, 2048x1152, 3840x2160, 2160x3840.

Custom gpt-image-2 sizes must satisfy all constraints: max edge <=3840px,
both edges multiples of 16px, long:short ratio <=3:1, and total pixels between
655360 and 8294400. Square images are usually fastest.";

const VARIANT_SEPARATOR_HELP: &str = "\
Text inserted between --prompt and each --variant. Any text is accepted.
Escapes \\n, \\r, \\t, and \\\\ are decoded; other backslashes are literal.
Default is \\n.";

#[derive(Debug, Parser)]
#[command(version, about, long_about = CLI_LONG_ABOUT, after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "DIR",
        help = "Override Codex home directory"
    )]
    pub codex_home: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Generate a new image from text", long_about = GENERATE_LONG_ABOUT)]
    Generate(GenerateArgs),
    #[command(
        about = "Edit images or use them as references",
        long_about = EDIT_LONG_ABOUT,
        after_long_help = EDIT_AFTER_LONG_HELP
    )]
    Edit(EditArgs),
}

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
pub struct GenerateArgs {
    #[arg(long, help = "Text prompt describing the desired image")]
    pub prompt: String,

    #[arg(long, value_name = "PATH", help = "PNG output path")]
    pub out: PathBuf,

    #[arg(long, default_value = DEFAULT_IMAGE_MODEL, help = "Image model to use")]
    pub model: String,

    #[arg(
        long,
        default_value_t = BackgroundArg::Auto,
        help = "Output background behavior; gpt-image-2 does not support transparent"
    )]
    pub background: BackgroundArg,

    #[arg(
        long,
        default_value_t = QualityArg::Auto,
        help = "Rendering quality; low is fastest for drafts"
    )]
    pub quality: QualityArg,

    #[arg(
        long,
        default_value = "auto",
        value_name = "SIZE",
        help = "Output size; gpt-image-2 accepts auto or constrained WIDTHxHEIGHT",
        long_help = SIZE_LONG_HELP
    )]
    pub size: String,

    #[arg(long, help = "Samples per prompt; codex-hosted fans out client-side")]
    pub n: Option<u64>,

    #[arg(
        long = "variant",
        value_name = "TEXT",
        help = "Prompt variant appended to --prompt; repeat for batches"
    )]
    pub variants: Vec<String>,

    #[arg(
        long,
        value_name = "TEXT",
        help = "Text inserted between --prompt and each --variant",
        long_help = VARIANT_SEPARATOR_HELP
    )]
    pub variant_separator: Option<String>,

    #[arg(long, help = "Force transport instead of provider-aware default")]
    pub transport: Option<TransportArg>,
}

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
pub struct EditArgs {
    #[arg(
        long = "image",
        value_name = "PATH",
        required = true,
        help = "Input image; repeat for references or compositing"
    )]
    pub images: Vec<PathBuf>,

    #[arg(long, help = "Edit prompt describing changes and preserved details")]
    pub prompt: String,

    #[arg(long, value_name = "PATH", help = "PNG output path")]
    pub out: PathBuf,

    #[arg(long, default_value = DEFAULT_IMAGE_MODEL, help = "Image model to use")]
    pub model: String,

    #[arg(
        long,
        default_value_t = BackgroundArg::Auto,
        help = "Output background behavior; gpt-image-2 does not support transparent"
    )]
    pub background: BackgroundArg,

    #[arg(
        long,
        default_value_t = QualityArg::Auto,
        help = "Rendering quality; low is fastest for drafts"
    )]
    pub quality: QualityArg,

    #[arg(
        long,
        default_value = "auto",
        value_name = "SIZE",
        help = "Output size; gpt-image-2 accepts auto or constrained WIDTHxHEIGHT",
        long_help = SIZE_LONG_HELP
    )]
    pub size: String,

    #[arg(long, help = "Samples per prompt; codex-hosted fans out client-side")]
    pub n: Option<u64>,

    #[arg(
        long = "variant",
        value_name = "TEXT",
        help = "Prompt variant appended to --prompt; repeat for batches"
    )]
    pub variants: Vec<String>,

    #[arg(
        long,
        value_name = "TEXT",
        help = "Text inserted between --prompt and each --variant",
        long_help = VARIANT_SEPARATOR_HELP
    )]
    pub variant_separator: Option<String>,

    #[arg(long, help = "Force transport instead of provider-aware default")]
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
