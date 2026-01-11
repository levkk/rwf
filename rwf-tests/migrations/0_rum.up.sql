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
CREATE INDEX IF NOT EXISTS rwf_jobs_pending_idx ON rwf_jobs USING btree(start_after, created_at) WHERE
    completed_at IS NULL
    AND started_at IS NULL
    AND attempts < retries;

-- Running jobs
CREATE INDEX IF NOT EXISTS rwf_jobs_runnin_idx ON rwf_jobs USING btree(start_after, created_at) WHERE
    completed_at IS NULL
    AND started_at IS NOT NULL
    AND attempts < retries;

CREATE INDEX IF NOT EXISTS rwf_jobs_name_completed_at_idx ON rwf_jobs USING btree(name, completed_at);

CREATE TABLE IF NOT EXISTS rwf_requests (
    id BIGSERIAL PRIMARY KEY,
    path VARCHAR NOT NULL,
    method VARCHAR NOT NULL DEFAULT 'GET',
    query JSONB NOT NULL DEFAULT '{}'::jsonb,
    code INTEGER NOT NULL DEFAULT 200,
    client_ip INET,
    client_id UUID NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS rwf_requests_path_created_at ON rwf_requests USING btree(created_at, path, client_id);

CREATE INDEX IF NOT EXISTS rwf_requests_errors ON rwf_requests USING btree(created_at, code, client_id) WHERE code >= 400;

CREATE INDEX IF NOT EXISTS rwf_requests_too_slow ON rwf_requests USING btree(created_at, duration, client_id) WHERE duration >= 1000.0; -- the unit is milliseconds

CREATE TABLE IF NOT EXISTS rwf_static_file_metas (
	id BIGSERIAL PRIMARY KEY,
	path VARCHAR NOT NULL UNIQUE,
	etag VARCHAR NOT NULL,
	modified TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
