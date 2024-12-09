SET
    LOCAL client_min_messages TO WARNING;

CREATE TABLE IF NOT EXISTS rwf_migrations (
    id BIGSERIAL PRIMARY KEY,
    version BIGINT NOT NULL,
    name VARCHAR UNIQUE NOT NULL,
    applied_at TIMESTAMPTZ
);
