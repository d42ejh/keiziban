-- Your SQL goes here
-- CREATE TYPE user_type AS ENUM ('admin','moderator','normal');

CREATE TABLE users (
primary_key SERIAL PRIMARY KEY,
id VARCHAR (16) UNIQUE NOT NULL,
registered_at TIMESTAMPTZ NOT NULL,
argon2_password TEXT NOT NULL,
user_type INTEGER NOT NULL,
user_status INTEGER NOT NULL
);