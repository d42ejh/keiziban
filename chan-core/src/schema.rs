table! {
    boards (primary_key) {
        primary_key -> Int4,
        uuid -> Uuid,
        created_at -> Timestamptz,
        name -> Varchar,
        description -> Varchar,
    }
}

table! {
    logs (primary_key) {
        primary_key -> Int4,
        timestamp -> Timestamptz,
        message -> Text,
        link -> Nullable<Text>,
        link_title -> Nullable<Text>,
    }
}

table! {
    threadposts (primary_key) {
        primary_key -> Int4,
        uuid -> Uuid,
        number -> Int4,
        posted_at -> Timestamptz,
        poster_user_id -> Text,
        parent_thread_id -> Uuid,
        body_text -> Varchar,
    }
}

table! {
    threads (primary_key) {
        primary_key -> Int4,
        uuid -> Uuid,
        created_at -> Timestamptz,
        parent_board_id -> Uuid,
        title -> Varchar,
        creator_user_id -> Text,
    }
}

table! {
    users (primary_key) {
        primary_key -> Int4,
        id -> Varchar,
        registered_at -> Timestamptz,
        argon2_password -> Text,
        user_type -> Int4,
        user_status -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    boards,
    logs,
    threadposts,
    threads,
    users,
);
