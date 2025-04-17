mod types;

use std::sync::Arc;
use axum::{Json, extract::{State, Path}};
use chrono::NaiveDateTime;
use std::fs;
use crate::AppState;
use types::{Conversation, DbConversation};


pub async fn get_conversation_content(
    Path(id): Path<i64>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let db_conversation = sqlx::query_as::<_, DbConversation>(
        "SELECT id, title, updatetime, filepath FROM conversation WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .unwrap();

    let filepath = "conversations/".to_string() + &db_conversation.filepath;
    println!("Filepath: {}", filepath);
    let file_content = fs::read(&filepath)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", db_conversation.filepath, e));
    
    let file_content = String::from_utf8(file_content)
        .unwrap_or_else(|e| panic!("Failed to decode file {} as UTF-8: {}", db_conversation.filepath, e));
    
    let json_value: serde_json::Value = serde_json::from_str(&file_content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON from file {}: {}", db_conversation.filepath, e));

    Json(json_value)
}

pub async fn get_conversations(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Conversation>> {
    let db_conversations = sqlx::query_as::<_, DbConversation>(
        "SELECT id, title, updatetime, filepath FROM conversation ORDER BY updatetime DESC"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();

    let conversations = db_conversations.into_iter().map(|db_conv| {
        let datetime = NaiveDateTime::parse_from_str(&db_conv.updatetime, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
            
        Conversation {
            id: db_conv.id,
            title: db_conv.title,
            time: datetime,
            filepath: db_conv.filepath,
        }
    }).collect();

    Json(conversations)
}
