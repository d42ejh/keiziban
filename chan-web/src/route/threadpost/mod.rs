pub struct ThreadPostInfo<'a> {
    pub uuid: &'a uuid::Uuid,
    pub number: u16,
    pub posted_at: &'a chrono::DateTime<chrono::Utc>,
    pub poster_user_id: &'a str,
    pub body_text: &'a str,
}
pub mod creation;
pub mod removal;
