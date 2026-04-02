use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::AppState;

#[derive(Serialize)]
pub struct ChallengeResponse {
    token: String,
    command: String,
}

pub async fn create_challenge(
    State(state): State<AppState>,
) -> Result<Json<ChallengeResponse>, StatusCode> {
    let token = shared::auth::generate_challenge_token();

    shared::db::create_auth_challenge(&state.pool, &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let port_part = if state.ssh_port == 22 {
        String::new()
    } else {
        format!(" -p {}", state.ssh_port)
    };
    let command = format!(
        "ssh {}@{}{} auth={}",
        state.ssh_user, state.ssh_host, port_part, token
    );

    Ok(Json(ChallengeResponse { token, command }))
}

#[derive(Deserialize)]
pub struct StatusQuery {
    token: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

pub async fn check_status(
    State(state): State<AppState>,
    Query(params): Query<StatusQuery>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let challenge = shared::db::get_auth_challenge(&state.pool, &params.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if challenge.status != "authenticated" {
        return Ok(Json(StatusResponse { status: "pending".into(), session_token: None, username: None }));
    }

    // Challenge is authenticated — create a session
    let session_token = shared::auth::generate_session_token();
    shared::db::create_session(&state.pool, &session_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StatusResponse {
        status: "authenticated".into(),
        session_token: Some(session_token),
        username: challenge.username,
    }))
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    token: String,
}

pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> StatusCode {
    let _ = shared::db::delete_session(&state.pool, &body.token).await;
    StatusCode::OK
}

// Internal endpoint called by auth-handler binary (localhost only)
#[derive(Deserialize)]
pub struct VerifyRequest {
    token: String,
    username: Option<String>,
}

pub async fn verify(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<VerifyRequest>,
) -> Result<StatusCode, StatusCode> {
    // Only allow from localhost
    if !addr.ip().is_loopback() {
        return Err(StatusCode::FORBIDDEN);
    }

    let ok = shared::db::authenticate_challenge(&state.pool, &body.token, body.username.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if ok {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
