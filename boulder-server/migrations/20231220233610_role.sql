-- Add migration script here
ALTER TABLE users ADD COLUMN IF NOT EXISTS role role not null default 'guest';
