CREATE TABLE app_logs (
    id bigserial primary key,
    ts timestamptz not null default NOW(),
    data text not null
);

INSERT INTO app_logs(data) VALUES ('TTest Entr'), ('Is it working'), ('Last on page one'), ('This is going to be on page two');