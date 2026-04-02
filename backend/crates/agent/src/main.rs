use shared::credentials;
use shared::models::UsageResponse;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info, warn};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const DEFAULT_POLL_INTERVAL_SECS: u64 = 900;

enum FetchResult {
    Ok(UsageResponse),
    RateLimited,
    Error(String),
}

async fn fetch_usage(client: &reqwest::Client, token: &str) -> FetchResult {
    let resp = match client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return FetchResult::Error(format!("request failed: {e}")),
    };

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return FetchResult::Error("unauthorized (401) — token may be expired".into());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return FetchResult::RateLimited;
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return FetchResult::Error(format!("unexpected status {status}: {body}"));
    }

    match resp.json::<UsageResponse>().await {
        Ok(u) => FetchResult::Ok(u),
        Err(e) => FetchResult::Error(format!("failed to parse response: {e}")),
    }
}

async fn poll_once(client: &reqwest::Client, pool: &sqlx::PgPool) -> bool {
    let token = match credentials::read_access_token() {
        Ok(t) => t,
        Err(e) => {
            error!("failed to read credentials: {e}");
            return false;
        }
    };

    let usage = match fetch_usage(client, &token).await {
        FetchResult::Ok(u) => u,
        FetchResult::RateLimited => {
            warn!("rate limited (429) — refreshing token");
            let new_token = match credentials::refresh_access_token(client).await {
                Ok(t) => t,
                Err(e) => {
                    error!("failed to refresh token: {e}");
                    return false;
                }
            };
            info!("token refreshed successfully");
            match fetch_usage(client, &new_token).await {
                FetchResult::Ok(u) => u,
                FetchResult::RateLimited => {
                    warn!("still rate limited after refresh — skipping");
                    return false;
                }
                FetchResult::Error(e) => {
                    warn!("{e}");
                    return false;
                }
            }
        }
        FetchResult::Error(e) => {
            warn!("{e}");
            return false;
        }
    };

    let five_hour_usage = usage.five_hour.as_ref().map(|b| b.utilization);
    let seven_day_usage = usage.seven_day.as_ref().map(|b| b.utilization);
    let seven_day_sonnet = usage.seven_day_sonnet.as_ref().map(|b| b.utilization);
    let five_hour_resets = usage.five_hour.as_ref().and_then(|b| b.resets_at);
    let seven_day_resets = usage.seven_day.as_ref().and_then(|b| b.resets_at);

    let creds = credentials::read_credentials().ok();
    let sub_type = creds
        .as_ref()
        .and_then(|c| c.claude_ai_oauth.subscription_type.as_deref());

    match shared::db::insert_usage_snapshot(
        pool,
        five_hour_usage,
        seven_day_usage,
        seven_day_sonnet,
        five_hour_resets,
        seven_day_resets,
        sub_type,
    )
    .await
    {
        Ok(id) => info!(
            id,
            five_hour = five_hour_usage,
            seven_day = seven_day_usage,
            seven_day_sonnet = seven_day_sonnet,
            "stored usage snapshot"
        ),
        Err(e) => error!("failed to insert snapshot: {e}"),
    }

    true
}

#[tokio::main]
async fn main() {
    // Load .env from the same directory as the binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let _ = dotenvy::from_path(dir.join(".env"));
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent=info,shared=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let poll_interval_secs: u64 = std::env::var("POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_POLL_INTERVAL_SECS);

    info!("connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let client = reqwest::Client::new();

    info!(interval_secs = poll_interval_secs, "agent started");

    // Poll immediately on start
    poll_once(&client, &pool).await;

    loop {
        // Sleep until the next aligned time (e.g., :00, :15, :30, :45 for 900s)
        let now = chrono::Utc::now().timestamp() as u64;
        let next = ((now / poll_interval_secs) + 1) * poll_interval_secs;
        let sleep_secs = next - now;
        info!(next_poll_in_secs = sleep_secs, "waiting for next aligned interval");

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(sleep_secs)) => {
                poll_once(&client, &pool).await;
            }
            _ = signal::ctrl_c() => {
                info!("shutting down");
                break;
            }
        }
    }
}
