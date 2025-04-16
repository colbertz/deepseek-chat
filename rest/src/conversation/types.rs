use serde::Serialize;

#[derive(Serialize)]
pub struct Conversation {
    pub id: &'static str,
    pub title: &'static str,
    pub time: u64,
}
