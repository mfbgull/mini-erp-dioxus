//! Authentication route handlers for the MiniERP API server.
//!
//! Endpoints:
//! - `POST /api/auth/login` — Username/password → JWT token
//! - `POST /api/auth/logout` — Clear session
//! - `GET /api/auth/me` — Current user profile from JWT

use crate::models::*;
use crate::server::db;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;

// ============================================================================
// Auth Router
// ============================================================================

/// Build the auth routes sub-router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/logout", post(logout_handler))
        .route("/api/auth/me", get(me_handler))
        .route("/api/auth/change-password", post(change_password_handler))
}

// ============================================================================
// Shared State
// ============================================================================

/// Application state shared across all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
}

impl AppState {
    pub fn new() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "mini-erp-dev-secret-key-change-in-production".to_string());
        Self { jwt_secret }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// JWT Helper
// ============================================================================

fn get_jwt_secret(state: &AppState) -> &str {
    &state.jwt_secret
}

fn create_token(user: &User, state: &AppState) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let claims = JwtClaims {
        sub: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: "mini-erp".to_string(),
        aud: "mini-erp-client".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret(state).as_bytes()),
    )
}

fn verify_token(token: &str, state: &AppState) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.set_issuer(&["mini-erp"]);
    validation.set_audience(&["mini-erp-client"]);
    validation.validate_exp = true;

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret(state).as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/auth/login
async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate input
    if req.username.trim().is_empty() || req.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Username and password are required." })),
        );
    }

    // Look up user
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let user_result = db.query_row(
        "SELECT id, username, email, password_hash, full_name, role, role_id, is_active
         FROM users WHERE username = ?1 AND is_active = 1",
        [&req.username],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                full_name: row.get(4)?,
                role: row.get(5)?,
                role_id: row.get(6)?,
                is_active: row.get::<_, i64>(7)? != 0,
            })
        },
    );

    match user_result {
        Ok(user) => {
            // Verify password
            match bcrypt::verify(&req.password, &user.password_hash) {
                Ok(true) => {
                    // Generate JWT
                    match create_token(&user, &state) {
                        Ok(token) => {
                            let profile = UserProfile {
                                id: user.id,
                                username: user.username,
                                full_name: user.full_name,
                                email: user.email,
                                role: user.role,
                                role_id: user.role_id,
                                is_active: user.is_active,
                            };
                            (
                                StatusCode::OK,
                                Json(json!({
                                    "success": true,
                                    "data": {
                                        "user": profile,
                                        "token": token,
                                    }
                                })),
                            )
                        }
                        Err(e) => {
                            tracing::error!("Failed to create JWT: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(json!({ "success": false, "error": "Authentication error." })),
                            )
                        }
                    }
                }
                Ok(false) => {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "success": false, "error": "Invalid username or password." })),
                    )
                }
                Err(e) => {
                    tracing::error!("Password verification error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "success": false, "error": "Authentication error." })),
                    )
                }
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "error": "Invalid username or password." })),
            )
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Server error. Please try again." })),
            )
        }
    }
}

/// POST /api/auth/logout
async fn logout_handler() -> impl IntoResponse {
    // JWT is stateless — the client should discard the token.
    // In a production app, we'd add the token to a blacklist.
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": { "message": "Logged out successfully." }
        })),
    )
}

/// GET /api/auth/me
async fn me_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract Bearer token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth_header {
        Some(token) => {
            match verify_token(&token, &state) {
                Ok(claims) => {
                    // Look up user from DB to get current profile
                    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
                    let user_result = db.query_row(
                        "SELECT id, username, email, full_name, role, role_id, is_active
                         FROM users WHERE id = ?1 AND is_active = 1",
                        [claims.sub],
                        |row| {
                            Ok(UserProfile {
                                id: row.get(0)?,
                                username: row.get(1)?,
                                full_name: row.get(2)?,
                                email: row.get(3)?,
                                role: row.get(4)?,
                                role_id: row.get(5)?,
                                is_active: row.get::<_, i64>(6)? != 0,
                            })
                        },
                    );

                    match user_result {
                        Ok(profile) => (
                            StatusCode::OK,
                            Json(json!({ "success": true, "data": profile })),
                        ),
                        Err(_) => (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({ "success": false, "error": "User not found." })),
                        ),
                    }
                }
                Err(_) => (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "success": false, "error": "Invalid or expired token." })),
                ),
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "No authorization token provided." })),
        ),
    }
}

/// POST /api/auth/change-password
async fn change_password_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let token = match auth_header {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({ "success": false, "error": "No authorization token provided." }))),
    };

    let claims = match verify_token(&token, &state) {
        Ok(c) => c,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(json!({ "success": false, "error": "Invalid or expired token." }))),
    };

    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT password_hash FROM users WHERE id = ?1",
        [claims.sub],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(hash) => {
            match bcrypt::verify(&req.current_password, &hash) {
                Ok(true) => {
                    let new_hash = bcrypt::hash(&req.new_password, 12).expect("Failed to hash password");
                    db.execute(
                        "UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2",
                        rusqlite::params![new_hash, claims.sub],
                    ).ok();
                    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Password changed successfully." } })))
                }
                _ => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Current password is incorrect." }))),
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "User not found." }))),
    }
}
