/// Board without child threads
struct BoardInfo {
    name: String,
    description: String,
    uuid: uuid::Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub mod list;
pub mod view;
