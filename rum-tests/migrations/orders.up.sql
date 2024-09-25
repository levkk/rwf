
DROP TABLE IF EXISTS orders;

CREATE TABLE orders (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT NOT NULL,
        name VARCHAR NOT NULL,
        optional VARCHAR
);