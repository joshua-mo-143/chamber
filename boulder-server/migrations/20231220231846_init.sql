-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
	access_level INT not null DEFAULT 0,
	roles TEXT[] NOT NULL DEFAULT array[]::TEXT[],
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS secrets (
    id SERIAL PRIMARY KEY,
    key VARCHAR NOT NULL UNIQUE,
    nonce BYTEA NOT NULL UNIQUE,
    ciphertext BYTEA NOT NULL UNIQUE,
	tags TEXT[] not null DEFAULT array[]::TEXT[],
	access_level INT NOT NULL DEFAULT 0,
	role_whitelist TEXT[] not null DEFAULT array[]::TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (username, password, access_level) values ('root', 'password', 9001);

CREATE TABLE IF NOT EXISTS core (
	id SERIAL PRIMARY KEY,
	unseal_key VARCHAR NOT NULL,
	crypto_key BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
