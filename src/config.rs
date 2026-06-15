use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthKeyringBackendKind;
use codex_login::AuthManagerConfig;
use serde::Deserialize;

const DEFAULT_RESPONSE_MODEL: &str = "gpt-5.5";
const DEFAULT_CHATGPT_BASE_URL: &str = "https://chatgpt.com/backend-api";

#[derive(Debug, Clone)]
pub(crate) struct CodexConfig {
    pub(crate) codex_home: PathBuf,
    pub(crate) cli_auth_credentials_store: AuthCredentialsStoreMode,
    pub(crate) auth_keyring_backend_kind: AuthKeyringBackendKind,
    pub(crate) forced_chatgpt_workspace_id: Option<Vec<String>>,
    pub(crate) model: String,
    pub(crate) chatgpt_base_url: String,
    pub(crate) responses_base_url: String,
    pub(crate) openai_base_url: Option<String>,
}

impl CodexConfig {
    pub(crate) async fn load(codex_home_override: Option<PathBuf>) -> Result<Self> {
        let codex_home = codex_home_override
            .or_else(|| std::env::var_os("CODEX_HOME").map(PathBuf::from))
            .or_else(default_codex_home)
            .context("unable to determine Codex home; set CODEX_HOME or pass --codex-home")?;
        let raw_config = load_raw_config(&codex_home).await?;
        let chatgpt_base_url = raw_config
            .chatgpt_base_url
            .unwrap_or_else(|| DEFAULT_CHATGPT_BASE_URL.to_string());
        let responses_base_url = codex_responses_base_url(&chatgpt_base_url);
        let openai_base_url = raw_config
            .openai_base_url
            .or_else(|| Some(codex_image_base_url(&chatgpt_base_url)));
        Ok(Self {
            codex_home,
            cli_auth_credentials_store: raw_config.cli_auth_credentials_store.unwrap_or_default(),
            auth_keyring_backend_kind: AuthKeyringBackendKind::default(),
            forced_chatgpt_workspace_id: raw_config.forced_chatgpt_workspace_id.map(Into::into),
            model: raw_config
                .model
                .unwrap_or_else(|| DEFAULT_RESPONSE_MODEL.to_string()),
            chatgpt_base_url,
            responses_base_url,
            openai_base_url,
        })
    }
}

impl AuthManagerConfig for CodexConfig {
    fn codex_home(&self) -> PathBuf {
        self.codex_home.clone()
    }

    fn cli_auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode {
        self.cli_auth_credentials_store
    }

    fn auth_keyring_backend_kind(&self) -> AuthKeyringBackendKind {
        self.auth_keyring_backend_kind
    }

    fn forced_chatgpt_workspace_id(&self) -> Option<Vec<String>> {
        self.forced_chatgpt_workspace_id.clone()
    }

    fn chatgpt_base_url(&self) -> String {
        self.chatgpt_base_url.clone()
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawCodexConfig {
    pub(crate) model: Option<String>,
    pub(crate) cli_auth_credentials_store: Option<AuthCredentialsStoreMode>,
    pub(crate) forced_chatgpt_workspace_id: Option<ForcedWorkspaceIds>,
    pub(crate) chatgpt_base_url: Option<String>,
    pub(crate) openai_base_url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum ForcedWorkspaceIds {
    One(String),
    Many(Vec<String>),
}

impl From<ForcedWorkspaceIds> for Vec<String> {
    fn from(value: ForcedWorkspaceIds) -> Self {
        match value {
            ForcedWorkspaceIds::One(id) => vec![id],
            ForcedWorkspaceIds::Many(ids) => ids,
        }
    }
}

async fn load_raw_config(codex_home: &Path) -> Result<RawCodexConfig> {
    let path = codex_home.join("config.toml");
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RawCodexConfig::default());
        }
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read `{}`", path.display()));
        }
    };
    toml::from_str(&contents).with_context(|| format!("failed to parse `{}`", path.display()))
}

fn default_codex_home() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".codex"))
}

pub(crate) fn codex_image_base_url(chatgpt_base_url: &str) -> String {
    format!("{}/api/codex", chatgpt_base_url.trim_end_matches('/'))
}

fn codex_responses_base_url(chatgpt_base_url: &str) -> String {
    format!("{}/codex", chatgpt_base_url.trim_end_matches('/'))
}
