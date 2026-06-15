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
use crate::config::CodexConfig;

pub(crate) async fn direct_image_api_generate(
    config: &CodexConfig,
    args: &GenerateArgs,
) -> Result<PathBuf> {
    let client = authenticated_images_client(config).await?;
    let response = client
        .generate(&generation_request(args), http::HeaderMap::new())
        .await
        .context("image generation request failed")?;
    write_single_image(&response, &args.out).await?;
    Ok(args.out.clone())
}

pub(crate) async fn direct_image_api_edit(
    config: &CodexConfig,
    args: &EditArgs,
) -> Result<PathBuf> {
    let client = authenticated_images_client(config).await?;
    let request = edit_request(args).await?;
    let response = client
        .edit(&request, http::HeaderMap::new())
        .await
        .context("image edit request failed")?;
    write_single_image(&response, &args.out).await?;
    Ok(args.out.clone())
}

pub(crate) fn generation_request(args: &GenerateArgs) -> ImageGenerationRequest {
    ImageGenerationRequest {
        prompt: args.prompt.clone(),
        background: Some(args.background.into()),
        model: args.model.clone(),
        n: args.n,
        quality: Some(args.quality.into()),
        size: Some(args.size.clone()),
    }
}

pub(crate) async fn edit_request(args: &EditArgs) -> Result<ImageEditRequest> {
    if args.images.len() > MAX_EDIT_IMAGES {
        bail!("--image accepts at most {MAX_EDIT_IMAGES} files");
    }

    let images = edit_input_images(args)
        .await?
        .unwrap_or_default()
        .into_iter()
        .map(|image_url| ImageUrl { image_url })
        .collect();

    Ok(ImageEditRequest {
        images,
        prompt: args.prompt.clone(),
        background: Some(args.background.into()),
        model: args.model.clone(),
        n: args.n,
        quality: Some(args.quality.into()),
        size: Some(args.size.clone()),
    })
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

async fn write_single_image(response: &ImageResponse, out: &Path) -> Result<()> {
    let image = response
        .data
        .first()
        .context("image API returned no image data")?;
    write_base64_image(&image.b64_json, out).await
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
