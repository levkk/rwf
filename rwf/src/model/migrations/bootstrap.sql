SET LOCAL client_min_messages TO WARNING;

CREATE TABLE IF NOT EXISTS rwf_migrations (
	id BIGSERIAL PRIMARY KEY,
	version BIGINT NOT NULL,
	name VARCHAR UNIQUE NOT NULL,
	applied_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS rwf_jobs (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    args JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    start_after TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    attempts INT NOT NULL DEFAULT 0,
    retries BIGINT NOT NULL DEFAULT 25,
    completed_at TIMESTAMPTZ,
    error VARCHAR
);

-- Pending jobs
CREATE INDEX IF NOT EXISTS rum_jobs_pending_idx ON rum_jobs USING btree(start_after, created_at) WHERE
    completed_at IS NULL
    AND started_at IS NULL
    AND attempts < retries;

-- Running jobs
CREATE INDEX IF NOT EXISTS rum_jobs_runnin_idx ON rum_jobs USING btree(start_after, created_at) WHERE
    completed_at IS NULL
    AND started_at IS NOT NULL
    AND attempts < retries;

CREATE INDEX IF NOT EXISTS rum_jobs_name_completed_at_idx ON rum_jobs USING btree(name, completed_at);