use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use codex_api::ImageEditRequest;
use codex_api::ImageGenerationRequest;
use codex_api::ImageResponse;
use codex_api::ImageUrl;

use crate::args::EditArgs;
use crate::args::GenerateArgs;
use crate::args::MAX_EDIT_IMAGES;
use crate::auth::authenticated_images_client;
use crate::batch::PromptBatch;
use crate::batch::PromptRequest;
use crate::config::CodexConfig;

pub(crate) async fn direct_image_api_generate_batch(
    config: &CodexConfig,
    args: &GenerateArgs,
    batch: &PromptBatch,
) -> Result<Vec<PathBuf>> {
    let client = authenticated_images_client(config).await?;
    let mut written = Vec::new();
    for request in batch.requests() {
        let response = client
            .generate(
                &generation_request_for_prompt(args, request),
                http::HeaderMap::new(),
            )
            .await
            .context("image generation request failed")?;
        write_images(&response, &request.outputs).await?;
        written.extend(request.outputs.iter().cloned());
    }
    Ok(written)
}

pub(crate) async fn direct_image_api_edit_batch(
    config: &CodexConfig,
    args: &EditArgs,
    batch: &PromptBatch,
) -> Result<Vec<PathBuf>> {
    let client = authenticated_images_client(config).await?;
    let image_urls = edit_input_images(args).await?.unwrap_or_default();
    let mut written = Vec::new();
    for prompt_request in batch.requests() {
        let request = edit_request_for_prompt(args, &image_urls, prompt_request);
        let response = client
            .edit(&request, http::HeaderMap::new())
            .await
            .context("image edit request failed")?;
        write_images(&response, &prompt_request.outputs).await?;
        written.extend(prompt_request.outputs.iter().cloned());
    }
    Ok(written)
}

#[cfg(test)]
pub(crate) fn generation_request(args: &GenerateArgs) -> ImageGenerationRequest {
    generation_request_for_values(args, args.prompt.clone(), args.n)
}

fn generation_request_for_prompt(
    args: &GenerateArgs,
    request: &PromptRequest,
) -> ImageGenerationRequest {
    generation_request_for_values(args, request.prompt.clone(), request.n)
}

fn generation_request_for_values(
    args: &GenerateArgs,
    prompt: String,
    n: Option<u64>,
) -> ImageGenerationRequest {
    ImageGenerationRequest {
        prompt,
        background: Some(args.background.into()),
        model: args.model.clone(),
        n,
        quality: Some(args.quality.into()),
        size: Some(args.size.clone()),
    }
}

#[cfg(test)]
pub(crate) async fn edit_request(args: &EditArgs) -> Result<ImageEditRequest> {
    if args.images.len() > MAX_EDIT_IMAGES {
        bail!("--image accepts at most {MAX_EDIT_IMAGES} files");
    }

    let image_urls = edit_input_images(args).await?.unwrap_or_default();
    let request = PromptRequest {
        prompt: args.prompt.clone(),
        n: args.n,
        outputs: vec![args.out.clone()],
    };
    Ok(edit_request_for_prompt(args, &image_urls, &request))
}

fn edit_request_for_prompt(
    args: &EditArgs,
    image_urls: &[String],
    request: &PromptRequest,
) -> ImageEditRequest {
    ImageEditRequest {
        images: image_urls
            .iter()
            .cloned()
            .map(|image_url| ImageUrl { image_url })
            .collect(),
        prompt: request.prompt.clone(),
        background: Some(args.background.into()),
        model: args.model.clone(),
        n: request.n,
        quality: Some(args.quality.into()),
        size: Some(args.size.clone()),
    }
}

pub(crate) async fn edit_input_images(args: &EditArgs) -> Result<Option<Vec<String>>> {
    if args.images.len() > MAX_EDIT_IMAGES {
        bail!("--image accepts at most {MAX_EDIT_IMAGES} files");
    }
    let mut images = Vec::with_capacity(args.images.len());
    for path in &args.images {
        images.push(image_data_url(path).await?);
    }
    Ok(Some(images))
}

async fn image_data_url(path: &Path) -> Result<String> {
    let mime = image_mime_type(path)?;
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read image `{}`", path.display()))?;
    Ok(format!(
        "data:{mime};base64,{}",
        BASE64_STANDARD.encode(bytes)
    ))
}

pub(crate) fn image_mime_type(path: &Path) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg" | "jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        unsupported => bail!(
            "unsupported image extension `{}`; expected png, jpg, jpeg, or webp",
            unsupported.unwrap_or("<none>")
        ),
    }
}

async fn write_images(response: &ImageResponse, outputs: &[PathBuf]) -> Result<()> {
    if response.data.len() != outputs.len() {
        bail!(
            "image API returned {} image(s), expected {}",
            response.data.len(),
            outputs.len()
        );
    }

    for (image, out) in response.data.iter().zip(outputs) {
        write_base64_image(&image.b64_json, out).await?;
    }
    Ok(())
}

pub(crate) async fn write_base64_image(b64_json: &str, out: &Path) -> Result<()> {
    let bytes = BASE64_STANDARD
        .decode(b64_json.trim().as_bytes())
        .context("image API returned invalid base64 payload")?;
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create output directory `{}`", parent.display()))?;
    }
    tokio::fs::write(out, bytes)
        .await
        .with_context(|| format!("failed to write output image `{}`", out.display()))?;
    Ok(())
}
