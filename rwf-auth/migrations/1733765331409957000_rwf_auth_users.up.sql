CREATE TABLE rwf_auth_users (
    id BIGSERIAL PRIMARY KEY,
    identifier VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);

CREATE INDEX ON rwf_auth_users USING btree (created_at);
