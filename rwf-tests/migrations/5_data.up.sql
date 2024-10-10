INSERT INTO orders (user_id, name, optional) VALUES (2, 'test', 'optional');
INSERT INTO order_items (order_id, product_id, amount) VALUES (1, 1, 5.0), (1, 2, 6.0);
INSERT INTO products (name, avg_price) VALUES ('apples', 6.0), ('doodles', 7.0);
INSERT INTO users (id, name) VALUES (2, 'test');