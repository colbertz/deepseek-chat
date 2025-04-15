use axum::{
    Json, Router,
    http::Method,
    routing::{get, post},
};

use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::cors::{Any, CorsLayer};

mod auth;
use auth::{AppState, JwtConfig, get_current_user, login, refresh_token};

#[derive(Serialize)]
struct Conversation {
    id: &'static str,
    title: &'static str,
    time: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use auth::{AppState, AuthResponse, JwtConfig};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use bcrypt;
    use tower::ServiceExt; // Required for oneshot() in tests

    #[tokio::test]
    async fn test_login_success() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (user_id INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("admin123", 4).unwrap();
        sqlx::query("INSERT INTO users (id, username, role) VALUES (1, 'admin', 'admin')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (user_id, password_hash) VALUES (1, ?)")
            .bind(&hash)
            .execute(&pool)
            .await
            .unwrap();

        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        let app = Router::new()
            .route("/auth/login", post(login))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"username": "admin", "password": "admin123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        let app = Router::new()
            .route("/auth/login", post(login))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username": "admin", "password": "wrong"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        // First login to get tokens
        let login_app = Router::new()
            .route("/auth/login", post(login))
            .with_state(Arc::clone(&state));

        let login_response = login_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"email": "test@example.com", "password": "password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

        // Now test refresh
        let refresh_app = Router::new()
            .route("/auth/refresh", post(refresh_token))
            .with_state(state);

        let refresh_response = refresh_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"refresh_token": "{}"}}"#,
                        auth_response.refresh_token
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(refresh_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_current_user() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        // First login to get token
        let login_app = Router::new()
            .route("/auth/login", post(login))
            .with_state(Arc::clone(&state));

        let login_response = login_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"email": "test@example.com", "password": "password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

        // Test getting current user
        let user_app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let user_response = user_app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header(
                        "Authorization",
                        format!("Bearer {}", auth_response.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(user_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_expired_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: -1, // Expired immediately
                refresh_expiry: 86400,
            },
        });

        let app = Router::new()
            .route("/auth/login", post(login))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"email": "test@example.com", "password": "password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        let app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header("Authorization", "Bearer invalid.token.here")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        let app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_role_based_access() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (user_id INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("user123", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, 'user@example.com', 'user')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (user_id, password_hash) VALUES (1, ?)")
            .bind(&hash)
            .execute(&pool)
            .await
            .unwrap();

        let state = Arc::new(AppState {
            pool,
            jwt_config: JwtConfig {
                secret: "test-secret".to_string(),
                access_expiry: 3600,
                refresh_expiry: 86400,
            },
        });

        // Login as regular user
        let login_app = Router::new()
            .route("/auth/login", post(login))
            .with_state(Arc::clone(&state));

        let login_response = login_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"email": "user@example.com", "password": "user123"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

        // Test getting current user
        let user_app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let user_response = user_app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header(
                        "Authorization",
                        format!("Bearer {}", auth_response.access_token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(user_response.status(), StatusCode::OK);
    }
}

#[tokio::main]
async fn main() {
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
        .route("/", get(root))
        .route("/conversations", get(get_conversations))
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

// 基本路由处理器
async fn root() -> &'static str {
    "Hello, Axum!"
}

async fn get_conversations() -> Json<Vec<Conversation>> {
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
