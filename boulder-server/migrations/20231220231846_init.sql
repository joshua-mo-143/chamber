-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
	access_level INT not null DEFAULT 0,
	roles VARCHAR[] NOT NULL DEFAULT array[]::varchar[]
created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS secrets (
    id SERIAL PRIMARY KEY,
    key VARCHAR NOT NULL UNIQUE,
    nonce BYTEA NOT NULL UNIQUE,
    ciphertext BYTEA NOT NULL UNIQUE,
	access_level INT NOT NULL DEFAULT 0,
	role_whitelist VARCHAR[] not null DEFAULT array[]::varchar[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
