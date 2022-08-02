-- Your SQL goes here
CREATE TABLE threadposts(
primary_key SERIAL PRIMARY KEY,
uuid UUID NOT NULL,
number INTEGER NOT NULL,
posted_at TIMESTAMPTZ NOT NULL,
poster_user_id TEXT NOT NULL,
parent_thread_id UUID NOT NULL,
body_text VARCHAR (4096) NOT NULL,
CONSTRAINT fk_thread_id FOREIGN KEY(parent_thread_id) REFERENCES threads(uuid),
CONSTRAINT fk_user_id FOREIGN KEY(poster_user_id) REFERENCES users(id)
)