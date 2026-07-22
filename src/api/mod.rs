//! Frontend API client for MiniERP.
//! Split into domain modules — see sub-modules for endpoint groups.

//! Frontend API client for MiniERP.
//!
//! Provides an `ApiClient` struct that calls the backend Axum server via HTTP.
//! Uses `reqwest` which is WASM-compatible (uses browser fetch API under the hood).
//!
//! # Architecture
//!
//! The server runs on `localhost:3001` by default. The frontend makes
//! cross-origin fetch requests (handled by CORS on the server side).
//!
//! ```ignore
//! let client = ApiClient::new();
//! client.set_token("jwt...");
//! let user = client.me().await?;
//! ```

use crate::models::*;
use dioxus::prelude::*;
use reqwest::Client as HttpClient;
use std::sync::atomic::{AtomicU16, Ordering};

/// Default port for the local MiniERP API server.
pub const DEFAULT_SERVER_PORT: u16 = 3001;

/// Base URL for API calls.
static SERVER_PORT: AtomicU16 = AtomicU16::new(DEFAULT_SERVER_PORT);

/// Set the server port (called during app init or from server auto-detection).
pub fn set_server_port(port: u16) {
    SERVER_PORT.store(port, Ordering::Relaxed);
}

pub(crate) fn base_url() -> String {
    format!("http://localhost:{}", SERVER_PORT.load(Ordering::Relaxed))
}

// ============================================================================
// ApiClient
// ============================================================================

/// HTTP client for the MiniERP backend API.
///
/// Stores a JWT token in memory and attaches it to every request as a
/// Bearer token header.
#[derive(Clone)]
pub struct ApiClient {
    inner: HttpClient,
    token: std::cell::RefCell<Option<String>>,
}

mod accounting;
mod admin;
mod customer;
mod inventory;
mod invoice;
mod manufacturing;
mod purchase;
mod report;

impl ApiClient {
    /// Create a new API client (no auth token).
    pub fn new() -> Self {
        Self {
            inner: HttpClient::new(),
            token: std::cell::RefCell::new(None),
        }
    }

    /// Set the JWT auth token for subsequent requests.
    pub fn set_token(&self, token: Option<String>) {
        *self.token.borrow_mut() = token;
    }

    /// Get the current JWT token.
    pub fn token(&self) -> Option<String> {
        self.token.borrow().clone()
    }

    /// Check if the client has a stored token.
    pub fn is_authenticated(&self) -> bool {
        self.token.borrow().is_some()
    }

    /// Build headers with auth token if present.
    fn headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        if let Some(token) = self.token.borrow().as_ref() {
            if let Ok(val) = header::HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(header::AUTHORIZATION, val);
            }
        }
        headers
    }


    /// POST /api/auth/login
    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, String> {
        let url = format!("{}/api/auth/login", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Login failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let data = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(data)
    }

    /// POST /api/auth/logout
    pub async fn logout(&self) -> Result<(), String> {
        let url = format!("{}/api/auth/logout", base_url());
        let _ = self.inner.post(&url).headers(self.headers()).send().await;
        Ok(())
    }

    /// GET /api/auth/me
    pub async fn me(&self) -> Result<UserProfile, String> {
        let url = format!("{}/api/auth/me", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if resp.status() == 401 {
            return Err("Unauthorized".to_string());
        }
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let data = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(data)
    }

    /// POST /api/auth/change-password
    pub async fn change_password(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/auth/change-password", base_url());
        let body = serde_json::json!({
            "current_password": current_password,
            "new_password": new_password,
        });
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }

}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the ApiClient from the auth context. Panics if not available.
pub fn use_api_client() -> Signal<ApiClient> {
    crate::auth::use_auth().api
}
