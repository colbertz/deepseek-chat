use axum::{
    Router,
    http::Method,
    routing::{get, post},
};

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub mod auth;
pub use auth::types::{AppState, JwtConfig};
use auth::{get_current_user, login, refresh_token};

mod conversation;
use conversation::{get_conversations, get_conversation_content};

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize database pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Initialize JWT config
    let jwt_config = JwtConfig {
        secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env"),
        access_expiry: std::env::var("JWT_ACCESS_EXPIRY")
            .expect("JWT_ACCESS_EXPIRY must be set in .env")
            .parse()
            .expect("JWT_ACCESS_EXPIRY must be a number"),
        refresh_expiry: std::env::var("JWT_REFRESH_EXPIRY")
            .expect("JWT_REFRESH_EXPIRY must be set in .env")
            .parse()
            .expect("JWT_REFRESH_EXPIRY must be a number"),
    };

    let state = Arc::new(AppState { pool, jwt_config });

    // 构建路由
    let app = Router::new()
        .route("/", get(|| async { "Hello, Axum!" }))
        .route("/conversations", get(get_conversations))
        .route("/conversations/{id}", get(get_conversation_content))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/me", get(get_current_user))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any)
                .allow_headers(Any),
        );

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Server running on http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}
