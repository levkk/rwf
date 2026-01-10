CREATE TABLE oidc_users (
	id bigserial primary key,
	sub uuid not null unique,
	name varchar(31) not null,
	email varchar(127) not null,
	access text not null,
	refresh text not null,
	expire timestamptz not null
);
