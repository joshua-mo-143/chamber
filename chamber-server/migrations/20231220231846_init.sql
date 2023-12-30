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
    nonce NUMERIC NOT NULL UNIQUE,
    ciphertext BYTEA NOT NULL UNIQUE,
	tags TEXT[] not null DEFAULT array[]::TEXT[],
	access_level INT NOT NULL DEFAULT 0,
	role_whitelist TEXT[] not null DEFAULT array[]::TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- password is "this"; make sure you revoke this user when you have everything set up!
INSERT INTO users (username, password, access_level) values ('root', '$argon2id$v=19$m=16,t=2,p=1$aEFxcjZlUlYwS21nVTNWWA$S92gSdO/RSqgRgAUlNe3Rw', 9001);
