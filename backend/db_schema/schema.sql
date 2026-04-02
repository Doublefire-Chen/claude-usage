-- Database schema for claude-usage
-- Run this after creating the database:
--   psql -U claude_usage -d claude_usage -f db_schema.sql

CREATE TABLE IF NOT EXISTS usage_snapshots (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    five_hour_usage DOUBLE PRECISION,
    seven_day_usage DOUBLE PRECISION,
    seven_day_sonnet_usage DOUBLE PRECISION,
    five_hour_resets_at TIMESTAMPTZ,
    seven_day_resets_at TIMESTAMPTZ,
    subscription_type TEXT
);

CREATE TABLE IF NOT EXISTS auth_challenges (
    id BIGSERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    authenticated_at TIMESTAMPTZ,
    username TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);
