use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::UserRole;

pub const AUTH_COOKIE_NAME: &str = "theater_auth_token";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Password hashing failed: {e}"))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| format!("Invalid hash format: {e}"))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_jwt(
    user_id: Uuid,
    email: &str,
    role: UserRole,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

pub struct OptionalAuthenticatedUser(pub Option<AuthenticatedUser>);

fn extract_token_from_parts(parts: &Parts) -> Option<String> {
    // 1. Check Cookie first
    if let Some(cookie_header) = parts.headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().split('=').collect();
                if parts.len() == 2 && parts[0] == AUTH_COOKIE_NAME {
                    return Some(parts[1].to_string());
                }
            }
        }
    }

    // 2. Fallback to Authorization: Bearer <token>
    if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].trim().to_string());
            }
        }
    }

    None
}

// Extractor for optional authenticated user (e.g. seat map endpoint to identify "is_mine" holds)
impl<S> FromRequestParts<S> for OptionalAuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "theater_booking_poc_jwt_secret_key_default_12345".to_string());

        if let Some(token) = extract_token_from_parts(parts) {
            if let Ok(claims) = validate_jwt(&token, &secret) {
                return Ok(OptionalAuthenticatedUser(Some(AuthenticatedUser {
                    id: claims.sub,
                    email: claims.email,
                    role: claims.role,
                })));
            }
        }

        Ok(OptionalAuthenticatedUser(None))
    }
}

// Extractor for mandatory authenticated user
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "theater_booking_poc_jwt_secret_key_default_12345".to_string());

        let token = extract_token_from_parts(parts).ok_or(AuthError::MissingCredentials)?;

        let claims = validate_jwt(&token, &secret).map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthenticatedUser {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    InvalidToken,
    Forbidden(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            AuthError::MissingCredentials => {
                (StatusCode::UNAUTHORIZED, "Authentication required".to_string())
            }
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())
            }
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        (
            status,
            Json(serde_json::json!({
                "error": err_msg
            })),
        )
            .into_response()
    }
}

// Helper to check user role in handlers or middleware
pub fn check_role(user: &AuthenticatedUser, allowed_roles: &[UserRole]) -> Result<(), AuthError> {
    if allowed_roles.contains(&user.role) {
        Ok(())
    } else {
        Err(AuthError::Forbidden(format!(
            "Access denied for role '{}'",
            user.role.as_str()
        )))
    }
}
