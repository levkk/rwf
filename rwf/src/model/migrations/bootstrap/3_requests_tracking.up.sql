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