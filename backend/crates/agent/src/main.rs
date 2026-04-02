use shared::credentials;
use shared::models::UsageResponse;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info, warn};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const DEFAULT_POLL_INTERVAL_SECS: u64 = 300;

async fn fetch_usage(client: &reqwest::Client, token: &str) -> Result<UsageResponse, String> {
    let resp = client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("unauthorized (401) — token may be expired, waiting for Claude Code to refresh".into());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("rate limited (429) — backing off".into());
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("unexpected status {status}: {body}"));
    }

    resp.json::<UsageResponse>()
        .await
        .map_err(|e| format!("failed to parse response: {e}"))
}

async fn poll_once(client: &reqwest::Client, pool: &sqlx::PgPool) {
    let token = match credentials::read_access_token() {
        Ok(t) => t,
        Err(e) => {
            error!("failed to read credentials: {e}");
            return;
        }
    };

    let usage = match fetch_usage(client, &token).await {
        Ok(u) => u,
        Err(e) => {
            warn!("{e}");
            return;
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
}

#[tokio::main]
async fn main() {
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
    let interval = Duration::from_secs(poll_interval_secs);

    info!(interval_secs = poll_interval_secs, "agent started");

    // Poll immediately on start, then on interval
    poll_once(&client, &pool).await;

    let mut ticker = tokio::time::interval(interval);
    ticker.tick().await; // consume the immediate first tick

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                poll_once(&client, &pool).await;
            }
            _ = signal::ctrl_c() => {
                info!("shutting down");
                break;
            }
        }
    }
}
