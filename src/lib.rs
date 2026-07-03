use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use tokio::task::JoinSet;

mod args;
mod auth;
mod batch;
mod config;
mod hosted;
mod image_io;

pub use args::Cli;

use args::Command;
use args::TransportArg;
use batch::PromptBatch;
use config::CodexConfig;
use hosted::HostedImageArgs;
use hosted::hosted_image_generation;
use image_io::direct_image_api_edit_batch;
use image_io::direct_image_api_generate_batch;
use image_io::edit_input_images;
use image_io::write_base64_image;

pub async fn run(cli: Cli) -> Result<Vec<PathBuf>> {
    let codex_config = CodexConfig::load(cli.codex_home).await?;
    let outputs = match cli.command {
        Command::Generate(args) => {
            let batch = PromptBatch::new(
                &args.prompt,
                &args.out,
                &args.variants,
                args.variant_separator.as_deref(),
                args.n,
            )?;
            if resolve_transport(&codex_config, args.transport) == TransportArg::ImageApi {
                direct_image_api_generate_batch(&codex_config, &args, &batch).await?
            } else {
                hosted_image_batch(
                    &codex_config,
                    HostedImageArgs::from(&args),
                    &batch,
                    Vec::new(),
                    "image generation request failed",
                )
                .await?
            }
        }
        Command::Edit(args) => {
            let batch = PromptBatch::new(
                &args.prompt,
                &args.out,
                &args.variants,
                args.variant_separator.as_deref(),
                args.n,
            )?;
            if resolve_transport(&codex_config, args.transport) == TransportArg::ImageApi {
                direct_image_api_edit_batch(&codex_config, &args, &batch).await?
            } else {
                let input_images = edit_input_images(&args).await?.unwrap_or_default();
                hosted_image_batch(
                    &codex_config,
                    HostedImageArgs::from(&args),
                    &batch,
                    input_images,
                    "image edit request failed",
                )
                .await?
            }
        }
    };
    Ok(outputs)
}

async fn hosted_image_batch(
    config: &CodexConfig,
    base_args: HostedImageArgs,
    batch: &PromptBatch,
    input_images: Vec<String>,
    request_context: &'static str,
) -> Result<Vec<PathBuf>> {
    let jobs = batch.single_output_jobs();
    let input_images = Arc::new(input_images);
    let mut tasks = JoinSet::new();

    for (index, job) in jobs.into_iter().enumerate() {
        let config = config.clone();
        let args = base_args.single_output_prompt(job.prompt);
        let input_images = Arc::clone(&input_images);
        tasks.spawn(async move {
            let image = hosted_image_generation(&config, args, input_images.as_slice())
                .await
                .context(request_context)?;
            write_base64_image(&image, &job.out).await?;
            Ok::<_, anyhow::Error>((index, job.out))
        });
    }

    let mut outputs = Vec::new();
    while let Some(result) = tasks.join_next().await {
        outputs.push(result.context("hosted image task failed")??);
    }
    outputs.sort_by_key(|(index, _)| *index);
    Ok(outputs.into_iter().map(|(_, out)| out).collect())
}

fn resolve_transport(config: &CodexConfig, transport: Option<TransportArg>) -> TransportArg {
    transport.unwrap_or_else(|| {
        if config.defaults_to_image_api_transport() {
            TransportArg::ImageApi
        } else {
            TransportArg::CodexHosted
        }
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    use clap::CommandFactory;
    use clap::Parser;
    use codex_api::ImageBackground;
    use codex_api::ImageGenerationRequest;
    use codex_api::ImageQuality;
    use codex_model_provider_info::WireApi;
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
    fn top_level_help_summarizes_gpt_image_2_output_guidance() {
        let help = Cli::command().render_long_help().to_string();

        assert!(help.contains("gpt-image-2"));
        assert!(help.contains("3840x2160"));
        assert!(help.contains("2160x3840"));
        assert!(help.contains("background=transparent"));
        assert!(help.contains("up to 2 minutes"));
        assert!(help.contains("ALL required text"));
        assert!(help.contains("very likely to render"));
    }

    #[test]
    fn edit_help_explains_reference_prompting() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("edit")
            .expect("edit subcommand should exist")
            .render_long_help()
            .to_string();

        assert!(help.contains("Use repeated --image"));
        assert!(help.contains("what must be"));
        assert!(help.contains("preserved"));
    }

    #[test]
    fn generate_help_calls_out_modern_text_rendering() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("generate")
            .expect("generate subcommand should exist")
            .render_long_help()
            .to_string();

        assert!(help.contains("newspaper pages or screenshots"));
        assert!(help.contains("all prompt-provided text"));
        assert!(help.contains("very likely to get it right"));
    }

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

    #[test]
    fn generate_accepts_repeated_variants_and_separator() {
        let cli = Cli::parse_from([
            "imagegen",
            "generate",
            "--prompt",
            "product photo",
            "--variant",
            "red",
            "--variant",
            "blue",
            "--variant-separator",
            "\\n\\n",
            "--out",
            "out.png",
        ]);
        let Command::Generate(args) = cli.command else {
            panic!("expected generate command");
        };
        assert_eq!(args.variants, vec!["red", "blue"]);
        assert_eq!(args.variant_separator.as_deref(), Some("\\n\\n"));
    }

    #[test]
    fn transport_is_optional_and_explicit_choice_is_preserved() {
        let defaulted = Cli::parse_from([
            "imagegen",
            "generate",
            "--prompt",
            "a red square",
            "--out",
            "out.png",
        ]);
        let Command::Generate(defaulted_args) = defaulted.command else {
            panic!("expected generate command");
        };
        assert_eq!(defaulted_args.transport, None);

        let explicit = Cli::parse_from([
            "imagegen",
            "generate",
            "--prompt",
            "a red square",
            "--out",
            "out.png",
            "--transport",
            "codex-hosted",
        ]);
        let Command::Generate(explicit_args) = explicit.command else {
            panic!("expected generate command");
        };
        assert_eq!(explicit_args.transport, Some(TransportArg::CodexHosted));
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
            variants: Vec::new(),
            variant_separator: None,
            transport: Some(TransportArg::CodexHosted),
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
    fn default_provider_routes_codex_hosted_when_transport_is_omitted() {
        let config =
            CodexConfig::from_raw(PathBuf::from("/tmp/codex"), RawCodexConfig::default()).unwrap();

        assert!(!config.defaults_to_image_api_transport());
        assert_eq!(resolve_transport(&config, None), TransportArg::CodexHosted);
    }

    #[test]
    fn selected_custom_model_provider_routes_image_api_by_default() {
        let raw_config: RawCodexConfig = toml::from_str(
            r#"
model_provider = "custom"

[model_providers.custom]
name = "Custom Images"
base_url = "https://images.example.com/v1"
env_key = "CUSTOM_IMAGE_API_KEY"
wire_api = "responses"
supports_websockets = false
"#,
        )
        .unwrap();
        let config = CodexConfig::from_raw(PathBuf::from("/tmp/codex"), raw_config).unwrap();

        assert_eq!(config.model_provider_id, "custom");
        assert_eq!(config.model_provider.name, "Custom Images");
        assert_eq!(
            config.model_provider.base_url.as_deref(),
            Some("https://images.example.com/v1")
        );
        assert_eq!(
            config.model_provider.env_key.as_deref(),
            Some("CUSTOM_IMAGE_API_KEY")
        );
        assert_eq!(config.model_provider.wire_api, WireApi::Responses);
        assert!(config.defaults_to_image_api_transport());
        assert_eq!(resolve_transport(&config, None), TransportArg::ImageApi);
        assert_eq!(
            resolve_transport(&config, Some(TransportArg::CodexHosted)),
            TransportArg::CodexHosted
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
