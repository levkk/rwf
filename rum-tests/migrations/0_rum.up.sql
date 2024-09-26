CREATE TABLE rum_jobs (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    args JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    start_after TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    attempts BIGINT NOT NULL DEFAULT 0,
    retries BIGINT NOT NULL DEFAULT 25,
    completed_at TIMESTAMPTZ,
    error VARCHAR
);