use serde::Serialize;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Serialize)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub time: DateTime<Utc>,
    pub filepath: String,
}

#[derive(FromRow)]
pub struct DbConversation {
    pub id: i64,
    pub title: String,
    pub updatetime: String,
    pub filepath: String,
}
