DROP TABLE IF EXISTS order_items;

CREATE TABLE order_items (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    amount DOUBLE PRECISION NOT NULL DEFAULT 5.0
);