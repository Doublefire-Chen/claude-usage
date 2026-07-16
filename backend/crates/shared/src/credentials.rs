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

/// Where a credential set was read from. Refreshed tokens are written back to
/// the same store so the agent never forks the token chain away from the one
/// Claude Code maintains (the Keychain, on macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    File,
    Keychain,
}

const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

/// Read Claude Code OAuth credentials.
/// Reads both ~/.claude/.credentials.json and the macOS Keychain and returns
/// whichever token expires later — Claude Code only maintains the Keychain
/// entry on macOS, so a leftover credentials file can hold a long-expired token.
pub fn read_credentials() -> Result<CredentialsFile, CredentialsError> {
    read_credentials_with_source().map(|(creds, _)| creds)
}

pub fn read_credentials_with_source(
) -> Result<(CredentialsFile, CredentialSource), CredentialsError> {
    let file_creds = read_from_file().ok().map(|c| (c, CredentialSource::File));
    #[cfg(target_os = "macos")]
    let keychain_creds = read_from_keychain()
        .ok()
        .map(|c| (c, CredentialSource::Keychain));
    #[cfg(not(target_os = "macos"))]
    let keychain_creds: Option<(CredentialsFile, CredentialSource)> = None;

    pick_freshest(file_creds, keychain_creds).ok_or(CredentialsError::NotFound)
}

fn pick_freshest(
    a: Option<(CredentialsFile, CredentialSource)>,
    b: Option<(CredentialsFile, CredentialSource)>,
) -> Option<(CredentialsFile, CredentialSource)> {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a.0.claude_ai_oauth.expires_at >= b.0.claude_ai_oauth.expires_at {
                Some(a)
            } else {
                Some(b)
            }
        }
        (a, b) => a.or(b),
    }
}

#[cfg(target_os = "macos")]
fn read_from_keychain() -> Result<CredentialsFile, CredentialsError> {
    let json_str = keychain_read(KEYCHAIN_SERVICE)?;
    let creds: CredentialsFile = serde_json::from_str(json_str.trim())?;
    Ok(creds)
}

#[cfg(target_os = "macos")]
fn keychain_read(service: &str) -> Result<String, CredentialsError> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .map_err(|e| CredentialsError::Other(format!("failed to run security command: {e}")))?;

    if !output.status.success() {
        return Err(CredentialsError::NotFound);
    }

    String::from_utf8(output.stdout)
        .map_err(|e| CredentialsError::Other(format!("invalid utf8 from keychain: {e}")))
}

/// Update (or create) a generic password item. The secret is fed to
/// `security -i` over stdin so it never appears in the process argument list.
#[cfg(target_os = "macos")]
fn keychain_set(service: &str, account: &str, secret: &str) -> Result<(), CredentialsError> {
    use std::io::Write as _;

    fn escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
    let cmd = format!(
        "add-generic-password -U -a \"{}\" -s \"{}\" -w \"{}\"\n",
        escape(account),
        escape(service),
        escape(secret),
    );

    let mut child = std::process::Command::new("security")
        .arg("-i")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| CredentialsError::Other(format!("failed to run security command: {e}")))?;

    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(cmd.as_bytes())
        .map_err(|e| CredentialsError::Other(format!("failed to write to security stdin: {e}")))?;

    let output = child
        .wait_with_output()
        .map_err(|e| CredentialsError::Other(format!("failed to wait for security: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CredentialsError::Other(format!(
            "keychain write failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

/// Account name of the existing Keychain item — `add-generic-password -U`
/// only updates in place when both service and account match.
#[cfg(target_os = "macos")]
fn keychain_account(service: &str) -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_keychain_account(&String::from_utf8_lossy(&output.stdout))
        .or_else(|| std::env::var("USER").ok())
}

/// Extract the account from `security find-generic-password` attribute output,
/// e.g. a line like `    "acct"<blob>="developer"`.
fn parse_keychain_account(attrs: &str) -> Option<String> {
    let rest = attrs.split("\"acct\"<blob>=\"").nth(1)?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(target_os = "macos")]
fn write_to_keychain(creds: &CredentialsFile) -> Result<(), CredentialsError> {
    let account = keychain_account(KEYCHAIN_SERVICE)
        .ok_or_else(|| CredentialsError::Other("keychain item account not found".into()))?;
    let json = serde_json::to_string(creds)?;
    keychain_set(KEYCHAIN_SERVICE, &account, &json)
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

/// Persist refreshed credentials to the store they were read from. If the
/// Keychain write fails, fall back to the file — the refresh may have rotated
/// the refresh token, so dropping the new credentials is never an option.
fn write_back(creds: &CredentialsFile, source: CredentialSource) -> Result<(), CredentialsError> {
    match source {
        CredentialSource::File => write_to_file(creds),
        CredentialSource::Keychain => {
            #[cfg(target_os = "macos")]
            {
                match write_to_keychain(creds) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        tracing::warn!(
                            "failed to write refreshed credentials to keychain, \
                             falling back to file: {e}"
                        );
                        write_to_file(creds)
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                write_to_file(creds)
            }
        }
    }
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
/// Writes the new credentials back to the store they were read from.
pub async fn refresh_access_token(client: &reqwest::Client) -> Result<String, CredentialsError> {
    let (mut creds, source) = read_credentials_with_source()?;
    let refresh_token = &creds.claude_ai_oauth.refresh_token;

    let resp = client
        .post("https://console.anthropic.com/v1/oauth/token")
        .json(&serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
        }))
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

    write_back(&creds, source)?;

    Ok(creds.claude_ai_oauth.access_token)
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OAuthCredentials;

    fn creds(access_token: &str, expires_at: i64) -> CredentialsFile {
        CredentialsFile {
            claude_ai_oauth: OAuthCredentials {
                access_token: access_token.into(),
                refresh_token: "rt".into(),
                expires_at,
                subscription_type: None,
                rate_limit_tier: None,
                extra: serde_json::Map::new(),
            },
            extra: serde_json::Map::new(),
        }
    }

    #[test]
    fn stale_file_does_not_shadow_fresh_keychain() {
        let file = (creds("stale", 1_000), CredentialSource::File);
        let keychain = (creds("fresh", 2_000), CredentialSource::Keychain);
        let (picked, source) = pick_freshest(Some(file), Some(keychain)).unwrap();
        assert_eq!(picked.claude_ai_oauth.access_token, "fresh");
        assert_eq!(source, CredentialSource::Keychain);
    }

    #[test]
    fn fresher_file_wins_over_keychain() {
        let file = (creds("fresh", 2_000), CredentialSource::File);
        let keychain = (creds("stale", 1_000), CredentialSource::Keychain);
        let (picked, source) = pick_freshest(Some(file), Some(keychain)).unwrap();
        assert_eq!(picked.claude_ai_oauth.access_token, "fresh");
        assert_eq!(source, CredentialSource::File);
    }

    #[test]
    fn single_source_is_used() {
        let (picked, source) =
            pick_freshest(Some((creds("file", 1), CredentialSource::File)), None).unwrap();
        assert_eq!(picked.claude_ai_oauth.access_token, "file");
        assert_eq!(source, CredentialSource::File);

        let (picked, source) =
            pick_freshest(None, Some((creds("kc", 1), CredentialSource::Keychain))).unwrap();
        assert_eq!(picked.claude_ai_oauth.access_token, "kc");
        assert_eq!(source, CredentialSource::Keychain);

        assert!(pick_freshest(None, None).is_none());
    }

    #[test]
    fn credentials_round_trip_preserves_unknown_fields() {
        // Shaped like the Keychain entry Claude Code writes, including fields
        // our structs don't model.
        let original: serde_json::Value = serde_json::json!({
            "claudeAiOauth": {
                "accessToken": "sk-ant-oat01-abc",
                "refreshToken": "sk-ant-ort01-def",
                "expiresAt": 1784229865153i64,
                "refreshTokenExpiresAt": 1815765865153i64,
                "scopes": ["user:inference", "user:profile"],
                "subscriptionType": "max",
                "rateLimitTier": "default_claude_max_5x"
            }
        });

        let parsed: CredentialsFile = serde_json::from_value(original.clone()).unwrap();
        let round_tripped = serde_json::to_value(&parsed).unwrap();
        assert_eq!(round_tripped, original);
    }

    #[test]
    fn absent_optional_fields_stay_absent() {
        let json = serde_json::json!({
            "claudeAiOauth": {
                "accessToken": "a",
                "refreshToken": "r",
                "expiresAt": 1i64
            }
        });
        let parsed: CredentialsFile = serde_json::from_value(json.clone()).unwrap();
        let round_tripped = serde_json::to_value(&parsed).unwrap();
        assert_eq!(round_tripped, json, "None fields must not serialize as null");
    }

    #[test]
    fn parses_account_from_security_attributes() {
        let attrs = r#"keychain: "/Users/developer/Library/Keychains/login.keychain-db"
version: 512
class: "genp"
attributes:
    "acct"<blob>="developer"
    "svce"<blob>="Claude Code-credentials"
"#;
        assert_eq!(
            parse_keychain_account(attrs),
            Some("developer".to_string())
        );
        assert_eq!(parse_keychain_account("no match"), None);
    }

    /// Real Keychain round-trip against a throwaway item (macOS only).
    #[cfg(target_os = "macos")]
    #[test]
    fn keychain_set_and_read_round_trip() {
        let service = format!("claude-usage-test-{}", std::process::id());
        let secret = r#"{"claudeAiOauth":{"accessToken":"sk-test \"quoted\"","expiresAt":1}}"#;

        keychain_set(&service, "test-account", secret).expect("keychain write failed");
        // -U update path: overwrite the same item
        let secret2 = r#"{"claudeAiOauth":{"accessToken":"updated","expiresAt":2}}"#;
        keychain_set(&service, "test-account", secret2).expect("keychain update failed");

        let read = keychain_read(&service).expect("keychain read failed");
        // cleanup before asserting so a failure doesn't leave the item behind
        let _ = std::process::Command::new("security")
            .args(["delete-generic-password", "-s", &service])
            .output();
        assert_eq!(read.trim(), secret2);
    }
}
