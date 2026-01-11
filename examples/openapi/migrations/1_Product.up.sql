CREATE TABLE products (
	id bigserial primary key,
	name varchar(255) not null,
	price DOUBLE PRECISION not null,
	stock bigint not null
);
