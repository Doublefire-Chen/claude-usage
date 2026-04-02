use crate::models::CredentialsFile;
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
/// Priority: CLAUDE_CODE_OAUTH_TOKEN env var > ~/.claude/.credentials.json > macOS Keychain.
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

fn read_from_file() -> Result<CredentialsFile, CredentialsError> {
    let path = dirs::home_dir()
        .map(|h| h.join(".claude").join(".credentials.json"))
        .ok_or(CredentialsError::NotFound)?;
    if !path.exists() {
        return Err(CredentialsError::NotFound);
    }
    let contents = std::fs::read_to_string(&path)?;
    let creds: CredentialsFile = serde_json::from_str(&contents)?;
    Ok(creds)
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
