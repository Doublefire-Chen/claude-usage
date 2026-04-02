mod auth_routes;
mod middleware;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use middleware::AuthUser;
use serde::{Deserialize, Serialize};
use shared::models::UsageSnapshot;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub ssh_user: String,
    pub ssh_host: String,
    pub ssh_port: u16,
}

#[derive(Serialize)]
struct UsageSnapshotResponse {
    id: i64,
    timestamp: DateTime<Utc>,
    five_hour_usage: Option<f64>,
    seven_day_usage: Option<f64>,
    seven_day_sonnet_usage: Option<f64>,
    five_hour_resets_at: Option<DateTime<Utc>>,
    seven_day_resets_at: Option<DateTime<Utc>>,
    subscription_type: Option<String>,
}

impl From<UsageSnapshot> for UsageSnapshotResponse {
    fn from(s: UsageSnapshot) -> Self {
        Self {
            id: s.id,
            timestamp: s.timestamp,
            five_hour_usage: s.five_hour_usage,
            seven_day_usage: s.seven_day_usage,
            seven_day_sonnet_usage: s.seven_day_sonnet_usage,
            five_hour_resets_at: s.five_hour_resets_at,
            seven_day_resets_at: s.seven_day_resets_at,
            subscription_type: s.subscription_type,
        }
    }
}

#[derive(Deserialize)]
struct HistoryQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

async fn get_current(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Option<UsageSnapshotResponse>>, StatusCode> {
    shared::db::get_latest_snapshot(&state.pool)
        .await
        .map(|opt| Json(opt.map(UsageSnapshotResponse::from)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_history(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<Vec<UsageSnapshotResponse>>, StatusCode> {
    let to = params.to.unwrap_or_else(Utc::now);
    let from = params.from.unwrap_or_else(|| to - Duration::hours(24));

    shared::db::get_snapshots_in_range(&state.pool, from, to)
        .await
        .map(|rows| Json(rows.into_iter().map(UsageSnapshotResponse::from).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn health() -> &'static str {
    "ok"
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
                .unwrap_or_else(|_| "server=info,shared=info".into()),
        )
        .init();

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".into());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let ssh_user = std::env::var("SSH_USER").unwrap_or_else(|_| "claude-auth".into());
    let ssh_host = std::env::var("SSH_HOST").unwrap_or_else(|_| "localhost".into());
    let ssh_port: u16 = std::env::var("SSH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(22);

    info!("connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState { pool, ssh_user, ssh_host, ssh_port };

    let app = Router::new()
        .route("/health", get(health))
        .route("/usage/current", get(get_current))
        .route("/usage/history", get(get_history))
        .route("/auth/challenge", post(auth_routes::create_challenge))
        .route("/auth/status", get(auth_routes::check_status))
        .route("/auth/logout", post(auth_routes::logout))
        .route("/internal/auth/verify", post(auth_routes::verify))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(frontend_url.parse().unwrap()))
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .expect("failed to bind");
    info!("server listening on {listen_addr}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("server error");
}

use std::net::SocketAddr;
