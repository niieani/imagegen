use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

mod args;
mod auth;
mod config;
mod hosted;
mod image_io;

pub use args::Cli;

use args::Command;
use args::TransportArg;
use config::CodexConfig;
use hosted::HostedImageArgs;
use hosted::hosted_image_generation;
use image_io::direct_image_api_edit;
use image_io::direct_image_api_generate;
use image_io::edit_input_images;
use image_io::write_base64_image;

pub async fn run(cli: Cli) -> Result<PathBuf> {
    let codex_config = CodexConfig::load(cli.codex_home).await?;
    let out = match cli.command {
        Command::Generate(args) if args.transport == TransportArg::ImageApi => {
            direct_image_api_generate(&codex_config, &args).await?
        }
        Command::Generate(args) => {
            let result = hosted_image_generation(&codex_config, HostedImageArgs::from(&args), None)
                .await
                .context("image generation request failed")?;
            write_base64_image(&result, &args.out).await?;
            args.out
        }
        Command::Edit(args) if args.transport == TransportArg::ImageApi => {
            direct_image_api_edit(&codex_config, &args).await?
        }
        Command::Edit(args) => {
            let input_images = edit_input_images(&args).await?;
            let result =
                hosted_image_generation(&codex_config, HostedImageArgs::from(&args), input_images)
                    .await
                    .context("image edit request failed")?;
            write_base64_image(&result, &args.out).await?;
            args.out
        }
    };
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use clap::Parser;
    use codex_api::ImageBackground;
    use codex_api::ImageGenerationRequest;
    use codex_api::ImageQuality;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::args::BackgroundArg;
    use crate::args::Command;
    use crate::args::DEFAULT_IMAGE_MODEL;
    use crate::args::EditArgs;
    use crate::args::QualityArg;
    use crate::args::TransportArg;
    use crate::config::RawCodexConfig;
    use crate::config::codex_image_base_url;
    use crate::hosted::extract_hosted_image;
    use crate::hosted::extract_image_status;
    use crate::image_io::edit_request;
    use crate::image_io::generation_request;
    use crate::image_io::image_mime_type;

    #[test]
    fn generate_args_map_all_image_api_options() {
        let cli = Cli::parse_from([
            "imagegen",
            "generate",
            "--prompt",
            "a red square",
            "--out",
            "out.png",
            "--model",
            "gpt-image-test",
            "--background",
            "opaque",
            "--quality",
            "low",
            "--size",
            "1024x1024",
            "--n",
            "2",
        ]);
        let Command::Generate(args) = cli.command else {
            panic!("expected generate command");
        };
        assert_eq!(
            generation_request(&args),
            ImageGenerationRequest {
                prompt: "a red square".to_string(),
                background: Some(ImageBackground::Opaque),
                model: "gpt-image-test".to_string(),
                n: Some(2),
                quality: Some(ImageQuality::Low),
                size: Some("1024x1024".to_string()),
            }
        );
    }

    #[test]
    fn edit_accepts_repeated_image_arguments() {
        let cli = Cli::parse_from([
            "imagegen",
            "edit",
            "--image",
            "a.png",
            "--image",
            "b.webp",
            "--prompt",
            "merge them",
            "--out",
            "out.png",
        ]);
        let Command::Edit(args) = cli.command else {
            panic!("expected edit command");
        };
        assert_eq!(
            args.images,
            vec![PathBuf::from("a.png"), PathBuf::from("b.webp")]
        );
    }

    #[tokio::test]
    async fn edit_rejects_more_than_five_images() {
        let args = EditArgs {
            images: (0..6)
                .map(|index| PathBuf::from(format!("{index}.png")))
                .collect(),
            prompt: "too many".to_string(),
            out: PathBuf::from("out.png"),
            model: DEFAULT_IMAGE_MODEL.to_string(),
            background: BackgroundArg::Auto,
            quality: QualityArg::Auto,
            size: "auto".to_string(),
            n: None,
            transport: TransportArg::CodexHosted,
        };

        let err = edit_request(&args).await.expect_err("should reject cap");
        assert_eq!(err.to_string(), "--image accepts at most 5 files");
    }

    #[test]
    fn image_mime_type_uses_supported_extensions_only() {
        assert_eq!(image_mime_type(Path::new("a.png")).unwrap(), "image/png");
        assert_eq!(image_mime_type(Path::new("a.jpg")).unwrap(), "image/jpeg");
        assert_eq!(image_mime_type(Path::new("a.jpeg")).unwrap(), "image/jpeg");
        assert_eq!(image_mime_type(Path::new("a.webp")).unwrap(), "image/webp");
        assert_eq!(
            image_mime_type(Path::new("a.gif"))
                .expect_err("gif should fail")
                .to_string(),
            "unsupported image extension `gif`; expected png, jpg, jpeg, or webp"
        );
    }

    #[test]
    fn forced_workspace_id_deserializes_string_or_list() {
        let single: RawCodexConfig =
            toml::from_str(r#"forced_chatgpt_workspace_id = "ws_1""#).unwrap();
        let many: RawCodexConfig =
            toml::from_str(r#"forced_chatgpt_workspace_id = ["ws_1", "ws_2"]"#).unwrap();

        assert_eq!(
            single.forced_chatgpt_workspace_id.map(Vec::<String>::from),
            Some(vec!["ws_1".to_string()])
        );
        assert_eq!(
            many.forced_chatgpt_workspace_id.map(Vec::<String>::from),
            Some(vec!["ws_1".to_string(), "ws_2".to_string()])
        );
    }

    #[test]
    fn image_base_url_is_derived_from_chatgpt_base_url() {
        assert_eq!(
            codex_image_base_url("https://chatgpt.com/backend-api/"),
            "https://chatgpt.com/backend-api/api/codex"
        );
    }

    #[test]
    fn command_requires_image_for_edit() {
        let err = Cli::try_parse_from([
            OsString::from("imagegen"),
            OsString::from("edit"),
            OsString::from("--prompt"),
            OsString::from("missing image"),
            OsString::from("--out"),
            OsString::from("out.png"),
        ])
        .expect_err("edit should require image");

        assert!(err.to_string().contains("--image <PATH>"));
    }

    #[test]
    fn hosted_stream_extracts_partial_image_payload() {
        let event = json!({
            "type": "response.image_generation_call.partial_image",
            "partial_image_b64": "abc123",
            "status": "generating",
        });

        assert_eq!(extract_hosted_image(&event), Some("abc123"));
        assert_eq!(extract_image_status(&event), Some("generating"));
    }

    #[test]
    fn hosted_stream_extracts_done_item_payload() {
        let event = json!({
            "type": "response.output_item.done",
            "item": {
                "type": "image_generation_call",
                "status": "completed",
                "result": "def456",
            },
        });

        assert_eq!(extract_hosted_image(&event), Some("def456"));
        assert_eq!(extract_image_status(&event), Some("completed"));
    }
}
