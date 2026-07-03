use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_api::ReqwestTransport;
use codex_api::ResponsesApiRequest;
use codex_client::HttpTransport;
use codex_client::RequestBody;
use codex_client::sse_stream;
use codex_login::default_client::build_reqwest_client;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use http::HeaderValue;
use serde_json::Value;
use serde_json::json;
use tokio::sync::mpsc;

use crate::args::BackgroundArg;
use crate::args::EditArgs;
use crate::args::GenerateArgs;
use crate::args::QualityArg;
use crate::auth::authenticated_responses_session;
use crate::config::CodexConfig;

pub(crate) async fn hosted_image_generation(
    config: &CodexConfig,
    args: HostedImageArgs,
    input_images: &[String],
) -> Result<String> {
    if args.n.is_some_and(|n| n != 1) {
        bail!("Codex-hosted image generation supports only one output image");
    }

    let (api_provider, api_auth) = authenticated_responses_session(config).await?;
    let input = hosted_response_input(&args.prompt, input_images);
    let request = ResponsesApiRequest {
        model: config.model.clone(),
        instructions: "Use the image generation tool to create the requested image.".to_string(),
        input,
        tools: vec![hosted_image_tool(&args)],
        tool_choice: "required".to_string(),
        parallel_tool_calls: false,
        reasoning: None,
        store: false,
        stream: true,
        include: Vec::new(),
        service_tier: None,
        prompt_cache_key: None,
        text: None,
        client_metadata: None,
    };
    let body = serde_json::to_value(&request).context("failed to encode responses request")?;
    let mut request = api_provider.build_request(http::Method::POST, "responses");
    request.body = Some(RequestBody::Json(body));
    request.headers.insert(
        http::header::ACCEPT,
        HeaderValue::from_static("text/event-stream"),
    );
    let request = api_auth
        .apply_auth(request)
        .await
        .context("failed to apply Codex responses auth")?;
    let stream = ReqwestTransport::new(build_reqwest_client())
        .stream(request)
        .await
        .context("failed to start Codex responses stream")?;

    let (tx, mut rx) = mpsc::channel(32);
    sse_stream(stream.bytes, api_provider.stream_idle_timeout, tx);

    let mut last_image = None;
    let mut last_status = None;
    while let Some(frame) = rx.recv().await {
        let frame = frame.context("Codex responses stream failed")?;
        let event: Value =
            serde_json::from_str(&frame).context("Codex responses stream sent invalid JSON")?;
        if let Some(image) = extract_hosted_image(&event) {
            last_image = Some(image.to_string());
        }
        if let Some(status) = extract_image_status(&event) {
            last_status = Some(status.to_string());
        }
        if event.get("type").and_then(Value::as_str) == Some("response.completed")
            && let Some(image) = last_image
        {
            return Ok(image);
        }
    }

    if let Some(image) = last_image {
        return Ok(image);
    }

    bail!(
        "Codex responses stream ended without a completed image{}",
        last_status
            .map(|status| format!("; last image status: {status}"))
            .unwrap_or_default()
    )
}

pub(crate) fn extract_hosted_image(event: &Value) -> Option<&str> {
    if event.get("type").and_then(Value::as_str)
        == Some("response.image_generation_call.partial_image")
    {
        return event.get("partial_image_b64").and_then(Value::as_str);
    }

    let item = event.get("item")?;
    if item.get("type").and_then(Value::as_str) != Some("image_generation_call") {
        return None;
    }
    item.get("result").and_then(Value::as_str)
}

pub(crate) fn extract_image_status(event: &Value) -> Option<&str> {
    if let Some(item) = event.get("item")
        && item.get("type").and_then(Value::as_str) == Some("image_generation_call")
    {
        return item.get("status").and_then(Value::as_str);
    }
    event.get("status").and_then(Value::as_str)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedImageArgs {
    prompt: String,
    model: String,
    background: BackgroundArg,
    quality: QualityArg,
    size: String,
    n: Option<u64>,
}

impl HostedImageArgs {
    pub(crate) fn single_output_prompt(&self, prompt: String) -> Self {
        Self {
            prompt,
            model: self.model.clone(),
            background: self.background,
            quality: self.quality,
            size: self.size.clone(),
            n: None,
        }
    }
}

impl From<&GenerateArgs> for HostedImageArgs {
    fn from(value: &GenerateArgs) -> Self {
        Self {
            prompt: value.prompt.clone(),
            model: value.model.clone(),
            background: value.background,
            quality: value.quality,
            size: value.size.clone(),
            n: value.n,
        }
    }
}

impl From<&EditArgs> for HostedImageArgs {
    fn from(value: &EditArgs) -> Self {
        Self {
            prompt: value.prompt.clone(),
            model: value.model.clone(),
            background: value.background,
            quality: value.quality,
            size: value.size.clone(),
            n: value.n,
        }
    }
}

fn hosted_image_tool(args: &HostedImageArgs) -> serde_json::Value {
    json!({
        "type": "image_generation",
        "action": "auto",
        "model": args.model,
        "background": args.background.to_string(),
        "quality": args.quality.to_string(),
        "size": args.size,
        "output_format": "png",
    })
}

fn hosted_response_input(prompt: &str, input_images: &[String]) -> Vec<ResponseItem> {
    let mut content = vec![ContentItem::InputText {
        text: prompt.to_string(),
    }];
    content.extend(
        input_images
            .iter()
            .cloned()
            .map(|image_url| ContentItem::InputImage {
                image_url,
                detail: None,
            }),
    );
    vec![ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content,
        phase: None,
    }]
}
