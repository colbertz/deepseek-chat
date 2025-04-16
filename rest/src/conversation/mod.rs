mod types;

use axum::Json;
use types::Conversation;

use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_conversations() -> Json<Vec<Conversation>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        * 1000;

    Json(vec![
        Conversation {
            id: "1",
            title: "Tailwind实现左右布局教程",
            time: now - 1000 * 60 * 60 * 20,
        },
        Conversation {
            id: "2",
            title: "Ubuntu setup es mysql minio",
            time: now - 1000 * 60 * 60 * 20,
        },
        Conversation {
            id: "3",
            title: "FFMPEG",
            time: now - 1000 * 60 * 60 * 20,
        },
        Conversation {
            id: "4",
            title: "Embedding and RAG",
            time: now - 1000 * 60 * 60 * 20,
        },
        Conversation {
            id: "5",
            title: "AI工具Claude、Cursor、v0",
            time: now - 1000 * 60 * 60 * 24 * 3,
        },
        Conversation {
            id: "6",
            title: "用户请教佛法智慧",
            time: now - 1000 * 60 * 60 * 24 * 5,
        },
        Conversation {
            id: "7",
            title: "佛说十二因缘详解",
            time: now - 1000 * 60 * 60 * 24 * 20,
        },
        Conversation {
            id: "8",
            title: "风的三重奏: 生命哲思与艺术解读",
            time: now - 1000 * 60 * 60 * 24 * 25,
        },
    ])
}
