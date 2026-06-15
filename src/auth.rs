use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_api::ImagesClient;
use codex_api::Provider;
use codex_api::ReqwestTransport;
use codex_api::SharedAuthProvider;
use codex_login::AuthManager;
use codex_login::default_client::build_reqwest_client;
use codex_model_provider::create_model_provider;
use codex_model_provider_info::ModelProviderInfo;

use crate::config::CodexConfig;

pub(crate) async fn authenticated_responses_session(
    config: &CodexConfig,
) -> Result<(Provider, SharedAuthProvider)> {
    let auth_manager =
        AuthManager::shared_from_config(config, /*enable_codex_api_key_env*/ false).await;
    if !auth_manager.current_auth_uses_codex_backend() {
        bail!("Codex ChatGPT/backend authentication not found; run `codex login` first");
    }

    let provider_info =
        ModelProviderInfo::create_openai_provider(Some(config.responses_base_url.clone()));
    let provider = create_model_provider(provider_info, Some(auth_manager));
    let api_provider = provider
        .api_provider()
        .await
        .context("failed to resolve Codex responses API provider")?;
    let api_auth = provider
        .api_auth()
        .await
        .context("failed to resolve Codex responses API auth")?;
    Ok((api_provider, api_auth))
}

pub(crate) async fn authenticated_images_client(
    config: &CodexConfig,
) -> Result<ImagesClient<ReqwestTransport>> {
    let auth_manager =
        AuthManager::shared_from_config(config, /*enable_codex_api_key_env*/ false).await;
    if !auth_manager.current_auth_uses_codex_backend() {
        bail!("Codex ChatGPT/backend authentication not found; run `codex login` first");
    }

    let provider_info = ModelProviderInfo::create_openai_provider(config.openai_base_url.clone());
    let provider = create_model_provider(provider_info, Some(auth_manager));
    let api_provider = provider
        .api_provider()
        .await
        .context("failed to resolve Codex image API provider")?;
    let api_auth = provider
        .api_auth()
        .await
        .context("failed to resolve Codex image API auth")?;
    Ok(ImagesClient::new(
        ReqwestTransport::new(build_reqwest_client()),
        api_provider,
        api_auth,
    ))
}
