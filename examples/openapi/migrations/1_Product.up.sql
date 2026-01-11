CREATE TABLE products (
	id bigserial primary key,
	name varchar(255) not null,
	price double not null,
	stock bigint not null
);
