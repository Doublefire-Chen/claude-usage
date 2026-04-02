use crate::models::{CredentialsFile, OAuthCredentials};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("failed to read credentials: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse credentials: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("no credentials found")]
    NotFound,
    #[error("{0}")]
    Other(String),
}

/// Read Claude Code OAuth credentials.
/// Priority: ~/.claude/.credentials.json > macOS Keychain.
pub fn read_credentials() -> Result<CredentialsFile, CredentialsError> {
    if let Ok(creds) = read_from_file() {
        return Ok(creds);
    }
    #[cfg(target_os = "macos")]
    {
        return read_from_keychain();
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(CredentialsError::NotFound)
    }
}

#[cfg(target_os = "macos")]
fn read_from_keychain() -> Result<CredentialsFile, CredentialsError> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| CredentialsError::Other(format!("failed to run security command: {e}")))?;

    if !output.status.success() {
        return Err(CredentialsError::NotFound);
    }

    let json_str = String::from_utf8(output.stdout)
        .map_err(|e| CredentialsError::Other(format!("invalid utf8 from keychain: {e}")))?;
    let creds: CredentialsFile = serde_json::from_str(json_str.trim())?;
    Ok(creds)
}

fn credentials_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join(".credentials.json"))
}

fn read_from_file() -> Result<CredentialsFile, CredentialsError> {
    let path = credentials_path().ok_or(CredentialsError::NotFound)?;
    if !path.exists() {
        return Err(CredentialsError::NotFound);
    }
    let contents = std::fs::read_to_string(&path)?;
    let creds: CredentialsFile = serde_json::from_str(&contents)?;
    Ok(creds)
}

fn write_to_file(creds: &CredentialsFile) -> Result<(), CredentialsError> {
    let path = credentials_path().ok_or(CredentialsError::NotFound)?;
    let contents = serde_json::to_string_pretty(creds)?;
    std::fs::write(&path, contents)?;
    Ok(())
}

/// Convenience: extract just the access token.
/// Checks CLAUDE_CODE_OAUTH_TOKEN env var first.
pub fn read_access_token() -> Result<String, CredentialsError> {
    if let Ok(token) = std::env::var("CLAUDE_CODE_OAUTH_TOKEN") {
        return Ok(token);
    }
    let creds = read_credentials()?;
    Ok(creds.claude_ai_oauth.access_token)
}

/// Refresh the OAuth token using the refresh_token.
/// Updates the credentials file with the new access_token.
pub async fn refresh_access_token(client: &reqwest::Client) -> Result<String, CredentialsError> {
    let mut creds = read_credentials()?;
    let refresh_token = &creds.claude_ai_oauth.refresh_token;

    let resp = client
        .post("https://console.anthropic.com/v1/oauth/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|e| CredentialsError::Other(format!("refresh request failed: {e}")))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(CredentialsError::Other(format!("refresh failed: {body}")));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| CredentialsError::Other(format!("failed to parse refresh response: {e}")))?;

    creds.claude_ai_oauth.access_token = token_resp.access_token.clone();
    if let Some(rt) = token_resp.refresh_token {
        creds.claude_ai_oauth.refresh_token = rt;
    }
    if let Some(expires_in) = token_resp.expires_in {
        creds.claude_ai_oauth.expires_at =
            chrono::Utc::now().timestamp_millis() + (expires_in as i64 * 1000);
    }

    write_to_file(&creds)?;

    Ok(creds.claude_ai_oauth.access_token)
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}
