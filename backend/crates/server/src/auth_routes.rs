use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
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
}

pub async fn check_status(
    State(state): State<AppState>,
    Query(params): Query<StatusQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<StatusResponse>), StatusCode> {
    let challenge = shared::db::get_auth_challenge(&state.pool, &params.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if challenge.status != "authenticated" {
        return Ok((jar, Json(StatusResponse { status: "pending".into() })));
    }

    // Challenge is authenticated — create a session
    let session_token = shared::auth::generate_session_token();
    shared::db::create_session(&state.pool, &session_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build(("session", session_token))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    Ok((jar.add(cookie), Json(StatusResponse { status: "authenticated".into() })))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<CookieJar, StatusCode> {
    if let Some(cookie) = jar.get("session") {
        let _ = shared::db::delete_session(&state.pool, cookie.value()).await;
    }
    let removal = Cookie::build(("session", ""))
        .path("/")
        .http_only(true)
        .removal()
        .build();
    Ok(jar.add(removal))
}

// Internal endpoint called by auth-handler binary (localhost only)
#[derive(Deserialize)]
pub struct VerifyRequest {
    token: String,
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

    let ok = shared::db::authenticate_challenge(&state.pool, &body.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if ok {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
