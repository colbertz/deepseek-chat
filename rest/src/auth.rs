use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    DatabaseError,
    TokenCreation,
    InvalidToken,
    MissingToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_config: JwtConfig,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry: i64,
    pub refresh_expiry: i64,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Verify user credentials against database
    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, email, role FROM users WHERE email = ?",
        credentials.email
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| AuthError::DatabaseError)?;

    let user = user.ok_or(AuthError::InvalidCredentials)?;

    let user_id = user.id.expect("Valid user should have ID");
    // Verify password (compare with bcrypt hash in auth table)
    let hash = sqlx::query_scalar!(
        "SELECT password_hash FROM auth WHERE user_id = ?",
        user_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| AuthError::DatabaseError)?;

    let password_valid = bcrypt::verify(credentials.password, &hash)
        .map_err(|_| AuthError::InvalidCredentials)?;
    
    if !password_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Generate tokens
    let now = Utc::now();
    let access_exp = now + Duration::seconds(state.jwt_config.access_expiry);
    let refresh_exp = now + Duration::seconds(state.jwt_config.refresh_expiry);

    let user_id = user.id.expect("Valid user should have ID");
    let role = user.role.clone();
    
    let access_claims = Claims {
        sub: user_id.to_string(),
        exp: access_exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.clone(),
    };

    let refresh_claims = Claims {
        sub: user_id.to_string(),
        exp: refresh_exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
    ).map_err(|_| AuthError::TokenCreation)?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
    ).map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_expiry,
    }))
}

#[axum::debug_handler]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Verify refresh token
    let token_data = decode::<Claims>(
        &request.refresh_token,
        &DecodingKey::from_secret(state.jwt_config.secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| AuthError::InvalidToken)?;

    // Generate new access token
    let now = Utc::now();
    let exp = now + Duration::seconds(state.jwt_config.access_expiry);

    let claims = Claims {
        sub: token_data.claims.sub,
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        role: token_data.claims.role,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
    ).map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: request.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_expiry,
    }))
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<User>, AuthError> {
    // Extract and verify token
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingToken)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_config.secret.as_ref()),
        &Validation::default(),
    ).map_err(|_| AuthError::InvalidToken)?;

    // Check if token is expired
    let now = Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        return Err(AuthError::InvalidToken);
    }

    // Get user from database
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, role FROM users WHERE id = ?",
        token_data.claims.sub
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| AuthError::DatabaseError)?;

    Ok(Json(user))
}
