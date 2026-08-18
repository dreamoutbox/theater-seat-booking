use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AUTH_COOKIE_NAME, AuthenticatedUser, generate_jwt, hash_password, verify_password},
    models::{User, UserRole, UserSummary},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserSummary,
    pub token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid email address" })),
        ));
    }
    if payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Password must be at least 6 characters" })),
        ));
    }

    let role = payload.role.unwrap_or(UserRole::Customer);

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(&email)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    if exists {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Email is already registered" })),
        ));
    }

    let password_hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    let user: User = sqlx::query_as(
        "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3)
         RETURNING id, email, password_hash, role, created_at",
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(role)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let token =
        generate_jwt(user.id, &user.email, user.role, &state.config.jwt_secret).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let cookie_header = format!(
        "{}={}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}",
        AUTH_COOKIE_NAME,
        token,
        7 * 24 * 3600
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_header).unwrap(),
    );

    let response_body = AuthResponse {
        user: user.into(),
        token,
    };

    Ok((StatusCode::CREATED, headers, Json(response_body)).into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let email = payload.email.trim().to_lowercase();

    let user_opt: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, role, created_at FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid email or password" })),
            ));
        }
    };

    let valid = verify_password(&payload.password, &user.password_hash).unwrap_or(false);
    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid email or password" })),
        ));
    }

    let token =
        generate_jwt(user.id, &user.email, user.role, &state.config.jwt_secret).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let cookie_header = format!(
        "{}={}; HttpOnly; Path=/; SameSite=Lax; Max-Age={}",
        AUTH_COOKIE_NAME,
        token,
        7 * 24 * 3600
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_header).unwrap(),
    );

    let response_body = AuthResponse {
        user: user.into(),
        token,
    };

    Ok((StatusCode::OK, headers, Json(response_body)).into_response())
}

pub async fn logout() -> Response {
    let cookie_header = format!(
        "{}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0",
        AUTH_COOKIE_NAME
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie_header).unwrap(),
    );

    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true, "message": "Logged out successfully" })),
    )
        .into_response()
}

pub async fn me(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<UserSummary>, (StatusCode, Json<serde_json::Value>)> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, role, created_at FROM users WHERE id = $1",
    )
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User not found" })),
        )),
    }
}
