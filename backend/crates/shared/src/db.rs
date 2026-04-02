use crate::models::{AuthChallenge, Session, UsageSnapshot};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

pub async fn insert_usage_snapshot(
    pool: &PgPool,
    five_hour_usage: Option<f64>,
    seven_day_usage: Option<f64>,
    seven_day_sonnet_usage: Option<f64>,
    five_hour_resets_at: Option<DateTime<Utc>>,
    seven_day_resets_at: Option<DateTime<Utc>>,
    subscription_type: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO usage_snapshots
            (timestamp, five_hour_usage, seven_day_usage, seven_day_sonnet_usage, five_hour_resets_at, seven_day_resets_at, subscription_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(Utc::now())
    .bind(five_hour_usage)
    .bind(seven_day_usage)
    .bind(seven_day_sonnet_usage)
    .bind(five_hour_resets_at)
    .bind(seven_day_resets_at)
    .bind(subscription_type)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn get_latest_snapshot(pool: &PgPool) -> Result<Option<UsageSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, UsageSnapshot>(
        "SELECT * FROM usage_snapshots ORDER BY timestamp DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_snapshots_in_range(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<UsageSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, UsageSnapshot>(
        "SELECT * FROM usage_snapshots WHERE timestamp >= $1 AND timestamp <= $2 ORDER BY timestamp ASC",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}

// --- Auth challenges ---

pub async fn create_auth_challenge(pool: &PgPool, token: &str) -> Result<AuthChallenge, sqlx::Error> {
    let expires_at = Utc::now() + Duration::minutes(5);
    sqlx::query_as::<_, AuthChallenge>(
        "INSERT INTO auth_challenges (token, expires_at) VALUES ($1, $2) RETURNING *",
    )
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_auth_challenge(pool: &PgPool, token: &str) -> Result<Option<AuthChallenge>, sqlx::Error> {
    sqlx::query_as::<_, AuthChallenge>(
        "SELECT * FROM auth_challenges WHERE token = $1 AND expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn authenticate_challenge(pool: &PgPool, token: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE auth_challenges SET status = 'authenticated', authenticated_at = NOW() WHERE token = $1 AND status = 'pending' AND expires_at > NOW()",
    )
    .bind(token)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// --- Sessions ---

pub async fn create_session(pool: &PgPool, token: &str) -> Result<Session, sqlx::Error> {
    let expires_at = Utc::now() + Duration::hours(24);
    sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (token, expires_at) VALUES ($1, $2) RETURNING *",
    )
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_session(pool: &PgPool, token: &str) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE token = $1 AND expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}
