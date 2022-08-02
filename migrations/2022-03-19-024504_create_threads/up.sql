-- Your SQL goes here

CREATE TABLE threads(
primary_key SERIAL PRIMARY KEY,
uuid UUID UNIQUE NOT NULL,
created_at TIMESTAMPTZ NOT NULL,
parent_board_id UUID NOT NULL,
title VARCHAR (47) NOT NULL,
creator_user_id TEXT NOT NULL,
CONSTRAINT fk_board_id FOREIGN KEY(parent_board_id) REFERENCES boards(uuid),
CONSTRAINT fk_user_id FOREIGN KEY(creator_user_id) REFERENCES users(id)
)