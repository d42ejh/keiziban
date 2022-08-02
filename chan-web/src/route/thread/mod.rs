/// Thread without posts
pub struct ThreadInfo {
    pub title: String,
    pub uuid: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub creator_user_id: String,
}

pub mod creation;
pub mod removal;
pub mod view;
