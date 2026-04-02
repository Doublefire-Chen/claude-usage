use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Raw credentials JSON stored by Claude Code.
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: OAuthCredentials,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

/// Response from GET https://api.anthropic.com/api/oauth/usage
#[derive(Debug, Deserialize, Serialize)]
pub struct UsageResponse {
    pub five_hour: Option<UsageBucket>,
    pub seven_day: Option<UsageBucket>,
    pub seven_day_sonnet: Option<UsageBucket>,
    pub seven_day_opus: Option<UsageBucket>,
    pub seven_day_oauth_apps: Option<UsageBucket>,
    pub seven_day_cowork: Option<UsageBucket>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsageBucket {
    pub utilization: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
}

/// Row in the usage_snapshots table.
#[derive(Debug, sqlx::FromRow)]
pub struct UsageSnapshot {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub five_hour_usage: Option<f64>,
    pub seven_day_usage: Option<f64>,
    pub seven_day_sonnet_usage: Option<f64>,
    pub five_hour_resets_at: Option<DateTime<Utc>>,
    pub seven_day_resets_at: Option<DateTime<Utc>>,
    pub subscription_type: Option<String>,
}

/// Row in the auth_challenges table.
#[derive(Debug, sqlx::FromRow)]
pub struct AuthChallenge {
    pub id: i64,
    pub token: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub authenticated_at: Option<DateTime<Utc>>,
    pub username: Option<String>,
}

/// Row in the sessions table.
#[derive(Debug, sqlx::FromRow)]
pub struct Session {
    pub id: i64,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
