-- Your SQL goes here

CREATE TABLE logs(
primary_key SERIAL PRIMARY KEY,
timestamp TIMESTAMPTZ NOT NULL,
message TEXT NOT NULL,
link TEXT,
link_title TEXT
)