use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::CookieJar;

use crate::AppState;

/// Extractor that validates the session cookie.
/// Add as a parameter to any handler that requires authentication.
pub struct AuthUser;

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let token = jar
            .get("session")
            .map(|c| c.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        shared::db::get_session(&state.pool, token)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser)
    }
}
