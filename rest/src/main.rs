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

    // test for pwd crypt
    #[test]
    fn test_bcrypt() {
        let password = "pwd";
        let hash = bcrypt::hash(password,10).unwrap();
        println!("hash: {}", hash);
        assert!(bcrypt::verify(password, &hash).unwrap());
    }

    #[tokio::test]
    async fn test_login_success() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("admin123", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, '263074289@qq.com', 'admin')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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
                        r#"{"email": "263074289@qq.com", "password": "admin123"}"#,
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
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
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
                    .body(Body::from(r#"{"email": "263074289@qq.com", "password": "wrong"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_refresh_token() {
        // #1 - Setup test data
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("password", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, 'test@example.com', 'user')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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

        // #2 - Perform login and keep access_token1
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

        assert_eq!(login_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response1: AuthResponse = serde_json::from_slice(&body).unwrap();
        let access_token1 = auth_response1.access_token.clone();

        std::thread::sleep(std::time::Duration::from_millis(2000));

        // #3 - Perform refresh and keep access_token2
        let refresh_app = Router::new()
            .route("/auth/refresh", post(refresh_token))
            .with_state(Arc::clone(&state));

        let refresh_response = refresh_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/refresh")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"refresh_token": "{}"}}"#,
                        auth_response1.refresh_token
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(refresh_response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(refresh_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response2: AuthResponse = serde_json::from_slice(&body).unwrap();
        let access_token2 = auth_response2.access_token.clone();

        // #4 - Ensure tokens are different
        assert_ne!(access_token1, access_token2, "Refresh token should generate new access token");

        // #5 - Test auth/me with new token
        let me_app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let me_response = me_app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header("Authorization", format!("Bearer {}", access_token2))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(me_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_current_user() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("password", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, '263074289@qq.com', 'admin')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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
                        r#"{"email": "263074289@qq.com", "password": "password"}"#,
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
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("password", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, 'test@example.com', 'user')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
            .bind(&hash)
            .execute(&pool)
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

        // First login (should succeed despite immediate expiry)
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

        assert_eq!(login_response.status(), StatusCode::OK);

        // Get token from login response
        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();

        // Now test expired token case
        let app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header("Authorization", format!("Bearer {}", auth_response.access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error_response["error"], "Invalid token");
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("password", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, '263074289@qq.com', 'admin')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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

        // First login to get valid token
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
                        r#"{"email": "263074289@qq.com", "password": "password"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
        let token = auth_response.access_token+"invalid"; 

        // Now test with invalid token
        let user_app = Router::new()
            .route("/auth/me", get(get_current_user))
            .with_state(state);

        let user_response = user_app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .header("Authorization", token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(user_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_token() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT, role TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("password", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, 'test@example.com', 'user')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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

        // First login to get valid token (though we won't use it)
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

        assert_eq!(login_response.status(), StatusCode::OK);

        // Now test missing token case
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
        
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let error_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error_response["error"], "Missing authorization token");
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
        sqlx::query("CREATE TABLE auth (userid INTEGER, password_hash TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let hash = bcrypt::hash("user123", 4).unwrap();
        sqlx::query("INSERT INTO users (id, email, role) VALUES (1, 'user@example.com', 'user')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO auth (userid, password_hash) VALUES (1, ?)")
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
