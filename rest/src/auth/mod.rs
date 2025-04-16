pub mod types;

use types::AuthResponse;
use types::{AppState, AuthError, Claims, LoginRequest, RefreshRequest, User};

use axum::{Json, extract::State, http::HeaderMap};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    tracing::debug!("Login attempt for email: {}", credentials.email);

    // Verify user credentials against database
    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, email, role FROM users WHERE email = ?",
        credentials.email
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error when querying user: {}", e);
        AuthError::DatabaseError
    })?;

    let user = match user {
        Some(u) => {
            tracing::debug!("Found user: {:?}", u);
            u
        }
        None => {
            tracing::warn!("No user found for email: {}", credentials.email);
            return Err(AuthError::InvalidCredentials);
        }
    };

    let user_id = user.id.expect("Valid user should have ID");
    tracing::debug!("Querying password hash for user_id: {}", user_id);

    // Verify password (compare with bcrypt hash in auth table)
    let hash = sqlx::query_scalar!("SELECT password_hash FROM auth WHERE userid = ?", user_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error when querying password hash: {}", e);
            AuthError::DatabaseError
        })?;

    tracing::debug!("Password hash retrieved: {}", hash);
    tracing::debug!("Password retrieved: {}", credentials.password);
    let password_valid = bcrypt::verify(credentials.password, &hash).map_err(|e| {
        tracing::error!("BCrypt verification error: {}", e);
        AuthError::InvalidCredentials
    })?;

    if !password_valid {
        tracing::warn!("Password verification failed for user_id: {}", user_id);
        return Err(AuthError::InvalidCredentials);
    }

    tracing::debug!("Password verified successfully for user_id: {}", user_id);

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
    )
    .map_err(|_| AuthError::TokenCreation)?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
    )
    .map_err(|_| AuthError::TokenCreation)?;

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
    )
    .map_err(|_| AuthError::InvalidToken)?;

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
    )
    .map_err(|_| AuthError::TokenCreation)?;

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
    )
    .map_err(|_| AuthError::InvalidToken)?;

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
