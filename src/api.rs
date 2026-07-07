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

fn base_url() -> String {
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

    // ── Auth endpoints ──

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

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let data = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(data)
    }

    /// POST /api/auth/logout
    pub async fn logout(&self) -> Result<(), String> {
        let url = format!("{}/api/auth/logout", base_url());
        let _ = self
            .inner
            .post(&url)
            .headers(self.headers())
            .send()
            .await;
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

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let data = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(data)
    }

    /// POST /api/auth/change-password
    pub async fn change_password(&self, current_password: &str, new_password: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/auth/change-password", base_url());
        let body = serde_json::json!({
            "current_password": current_password,
            "new_password": new_password,
        });
        let resp = self.inner.post(&url).headers(self.headers()).json(&body).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }

    // ── Dashboard endpoints ──

    /// GET /api/dashboard/summary
    pub async fn get_dashboard_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/top-customers
    pub async fn get_top_customers(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/top-customers", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/sales-summary
    pub async fn get_sales_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/sales-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/expense-summary
    pub async fn get_expense_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/expense-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/production-status
    pub async fn get_production_status(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/production-status", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/stock-movement-summary
    pub async fn get_stock_movement_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/stock-movement-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/kpi
    pub async fn get_dashboard_kpi(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/kpi", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/dashboard/ar-summary
    pub async fn get_ar_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/ar-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/sales/dashboard
    pub async fn get_sales_dashboard(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/dashboard", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Report endpoints ──

    /// GET /api/reports/ar-aging
    pub async fn get_ar_aging(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/ar-aging", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/customer-statements
    pub async fn get_customer_statements_report(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/customer-statements", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/top-debtors
    pub async fn get_top_debtors(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/top-debtors", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/dso
    pub async fn get_dso(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/dso", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/ar-summary
    pub async fn get_ar_summary_report(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/ar-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/sales-summary
    pub async fn get_sales_summary_report(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/sales-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/sales-by-customer
    pub async fn get_sales_by_customer(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/sales-by-customer", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/sales-by-item
    pub async fn get_sales_by_item(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/sales-by-item", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/stock-level
    pub async fn get_stock_level(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/stock-level", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/low-stock
    pub async fn get_low_stock_report(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/low-stock", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/stock-valuation
    pub async fn get_stock_valuation(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/stock-valuation", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/inventory-movement
    pub async fn get_inventory_movement(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/inventory-movement", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/profit-loss
    pub async fn get_profit_loss(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/profit-loss", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/cash-flow
    pub async fn get_cash_flow(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/cash-flow", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/purchase-summary
    pub async fn get_purchase_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/purchase-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/supplier-analysis
    pub async fn get_supplier_analysis(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/supplier-analysis", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/production-summary
    pub async fn get_production_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/production-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/bom-usage
    pub async fn get_bom_usage(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/bom-usage", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/expenses
    pub async fn get_expense_report(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/expenses", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/trial-balance
    pub async fn get_trial_balance(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/trial-balance", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/general-ledger
    pub async fn get_general_ledger(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/general-ledger", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/balance-sheet
    pub async fn get_balance_sheet(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/balance-sheet", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/income-statement
    pub async fn get_income_statement(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/income-statement", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/tax-summary
    pub async fn get_tax_summary(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/tax-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/batch-traceability/{item_id}
    pub async fn get_batch_traceability(&self, item_id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/batch-traceability/{}", base_url(), item_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/reports/custom
    pub async fn list_custom_reports(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/reports/custom", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/reports/custom
    pub async fn create_custom_report(&self, form: &CustomReportForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/reports/custom", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Inventory endpoints ──

    /// GET /api/inventory/items
    pub async fn list_items(&self) -> Result<Vec<Item>, String> {
        let url = format!("{}/api/inventory/items", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Item> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/inventory/items/:id
    pub async fn get_item(&self, id: i64) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// POST /api/inventory/items
    pub async fn create_item(&self, form: &ItemForm) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// PUT /api/inventory/items/:id
    pub async fn update_item(&self, id: i64, form: &ItemForm) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// DELETE /api/inventory/items/:id
    pub async fn delete_item(&self, id: i64) -> Result<(), String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        Ok(())
    }



    /// GET /api/inventory/warehouses
    pub async fn list_warehouses(&self) -> Result<Vec<Warehouse>, String> {
        let url = format!("{}/api/inventory/warehouses", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let warehouses: Vec<Warehouse> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouses)
    }

    /// GET /api/inventory/stock-movements
    pub async fn list_stock_movements(&self) -> Result<Vec<StockMovement>, String> {
        let url = format!("{}/api/inventory/stock-movements", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let movements: Vec<StockMovement> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(movements)
    }

    /// GET /api/inventory/stock-balances
    pub async fn list_stock_balances(&self) -> Result<Vec<StockBalance>, String> {
        let url = format!("{}/api/inventory/stock-balances", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let balances: Vec<StockBalance> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(balances)
    }

    /// GET /api/inventory/physical-counts
    pub async fn list_physical_counts(&self) -> Result<Vec<PhysicalCount>, String> {
        let url = format!("{}/api/inventory/physical-counts", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let counts: Vec<PhysicalCount> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(counts)
    }

    /// GET /api/inventory/warehouses/:id
    pub async fn get_warehouse(&self, id: i64) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// POST /api/inventory/warehouses
    pub async fn create_warehouse(&self, form: &WarehouseForm) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// PUT /api/inventory/warehouses/:id
    pub async fn update_warehouse(&self, id: i64, form: &WarehouseForm) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// DELETE /api/inventory/warehouses/:id
    pub async fn delete_warehouse(&self, id: i64) -> Result<(), String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        Ok(())
    }

    /// POST /api/inventory/stock-movements
    pub async fn create_stock_movement(&self, form: &StockMovementForm) -> Result<StockMovement, String> {
        let url = format!("{}/api/inventory/stock-movements", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let movement: StockMovement = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(movement)
    }

    /// GET /api/inventory/physical-counts/:id
    pub async fn get_physical_count(&self, id: i64) -> Result<PhysicalCount, String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let count: PhysicalCount = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(count)
    }

    /// POST /api/inventory/physical-counts
    pub async fn create_physical_count(&self, form: &PhysicalCountForm) -> Result<PhysicalCount, String> {
        let url = format!("{}/api/inventory/physical-counts", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let count: PhysicalCount = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(count)
    }

    /// GET /api/inventory/stock-summary
    pub async fn get_stock_summary(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/inventory/stock-summary", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/inventory/items-categories
    pub async fn list_item_categories(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/inventory/items-categories", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        serde_json::from_value(body["data"].clone()).map_err(|e| format!("Parse error: {}", e))
    }

    /// GET /api/inventory/items-low-stock
    pub async fn list_low_stock_items(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/inventory/items-low-stock", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/inventory/items-uom
    pub async fn list_uom(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/inventory/items-uom", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        serde_json::from_value(body["data"].clone()).map_err(|e| format!("Parse error: {}", e))
    }

    /// GET /api/inventory/physical-counts/{count_id}/items
    pub async fn get_physical_count_items(&self, count_id: i64) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/inventory/physical-counts/{}/items", base_url(), count_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// POST /api/inventory/physical-counts/{count_id}/complete
    pub async fn complete_physical_count(&self, count_id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/inventory/physical-counts/{}/complete", base_url(), count_id);
        let resp = self.inner.post(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/inventory/physical-counts/{count_id}/cancel
    pub async fn cancel_physical_count(&self, count_id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/inventory/physical-counts/{}/cancel", base_url(), count_id);
        let resp = self.inner.post(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/inventory/physical-counts/{id}
    pub async fn update_physical_count(&self, id: i64, form: &PhysicalCountForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Invoice endpoints ──

    /// GET /api/invoices
    pub async fn list_invoices(&self) -> Result<Vec<Invoice>, String> {
        let url = format!("{}/api/invoices", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let invoices: Vec<Invoice> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(invoices)
    }

    /// GET /api/invoices/:id
    pub async fn get_invoice(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }

    /// POST /api/invoices
    pub async fn create_invoice(&self, form: &InvoiceForm) -> Result<Invoice, String> {
        let url = format!("{}/api/invoices", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let invoice: Invoice = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(invoice)
    }

    /// PUT /api/invoices/:id
    pub async fn update_invoice(&self, id: i64, form: &InvoiceForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }

    /// PUT /api/invoices/:id/cancel
    pub async fn cancel_invoice(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}/cancel", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }

    /// GET /api/invoices/:id/payments
    pub async fn get_invoice_payments(&self, id: i64) -> Result<Vec<Payment>, String> {
        let url = format!("{}/api/invoices/{}/payments", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let payments: Vec<Payment> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(payments)
    }

    /// POST /api/invoices/:id/return
    pub async fn return_invoice(&self, id: i64, items: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}/return", base_url(), id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(items)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/invoices/returns
    pub async fn list_invoice_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/invoices/returns", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── POS endpoints ──

    /// POST /api/pos/sale
    pub async fn create_pos_sale(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/pos/sale", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/pos/transactions
    pub async fn list_pos_transactions(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/pos/transactions", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    // ── Mobile invoice endpoints ──

    /// POST /api/mobile-invoices/draft
    pub async fn create_mobile_draft(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/mobile-invoices/draft", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/mobile-invoices/items/search?q={query}
    pub async fn search_items(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/items/search?q={}", base_url(), query);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/mobile-invoices/customers/search?q={query}
    pub async fn search_customers(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/customers/search?q={}", base_url(), query);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/mobile-invoices/tax-rates
    pub async fn get_mobile_tax_rates(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/tax-rates", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/mobile-invoices/payment-terms
    pub async fn get_mobile_payment_terms(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/payment-terms", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// POST /api/mobile-invoices/submit
    pub async fn submit_mobile_invoice(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/mobile-invoices/submit", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Payment endpoints ──

    /// GET /api/payments
    pub async fn list_payments(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/payments", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/payments/:id
    pub async fn get_payment(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/payments/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/payments
    pub async fn create_payment(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/payments", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Customer endpoints ──

    /// GET /api/customers
    pub async fn list_customers(&self) -> Result<Vec<Customer>, String> {
        let url = format!("{}/api/customers", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let customers: Vec<Customer> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(customers)
    }

    /// GET /api/customers/{id}
    pub async fn get_customer(&self, id: i64) -> Result<Customer, String> {
        let url = format!("{}/api/customers/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let customer: Customer = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(customer)
    }

    /// GET /api/customers/{id}/ledger
    pub async fn get_customer_ledger(&self, id: i64) -> Result<Vec<CustomerLedgerEntry>, String> {
        let url = format!("{}/api/customers/{}/ledger", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let entries: Vec<CustomerLedgerEntry> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(entries)
    }

    /// GET /api/customers/{id}/balance
    pub async fn get_customer_balance(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/customers/{}/balance", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/customers/{id}/payments
    pub async fn get_customer_payments(&self, id: i64) -> Result<Vec<Payment>, String> {
        let url = format!("{}/api/customers/{}/payments", base_url(), id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Payment> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/customers
    pub async fn create_customer(&self, form: &CustomerForm) -> Result<Customer, String> {
        let url = format!("{}/api/customers", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let customer: Customer = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(customer)
    }

    /// GET /api/customers/{customer_id}/statement?from={from}&to={to}
    pub async fn get_customer_statement(&self, customer_id: i64, from: &str, to: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/customers/{}/statement?from={}&to={}", base_url(), customer_id, from, to);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// PUT /api/customers/:id
    pub async fn update_customer(&self, id: i64, form: &CustomerForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/customers/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/customers/recalculate-balances
    pub async fn recalculate_balances(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/customers/recalculate-balances", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Supplier endpoints ──

    /// GET /api/suppliers/:id
    pub async fn get_supplier(&self, id: i64) -> Result<Supplier, String> {
        let url = format!("{}/api/suppliers/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let supplier: Supplier = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(supplier)
    }

    /// GET /api/suppliers
    pub async fn list_suppliers(&self) -> Result<Vec<Supplier>, String> {
        let url = format!("{}/api/suppliers", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let suppliers: Vec<Supplier> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(suppliers)
    }

    /// POST /api/suppliers
    pub async fn create_supplier(&self, form: &SupplierForm) -> Result<Supplier, String> {
        let url = format!("{}/api/suppliers", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let supplier: Supplier = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(supplier)
    }

    /// PUT /api/suppliers/:id
    pub async fn update_supplier(&self, id: i64, form: &SupplierForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/suppliers/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Purchase endpoints ──

    /// GET /api/purchases
    pub async fn list_direct_purchases(&self) -> Result<Vec<DirectPurchase>, String> {
        let url = format!("{}/api/purchases", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let purchases: Vec<DirectPurchase> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(purchases)
    }

    /// GET /api/purchases/:id
    pub async fn get_direct_purchase(&self, id: i64) -> Result<DirectPurchase, String> {
        let url = format!("{}/api/purchases/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let purchase: DirectPurchase = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(purchase)
    }

    /// PUT /api/purchases/{id}
    pub async fn update_direct_purchase(&self, id: i64, form: &DirectPurchaseForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchases/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/receipts
    pub async fn list_receipts(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/receipts", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/purchase-returns
    pub async fn list_purchase_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-returns", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    // ── Catalog alias ──

    /// GET /api/inventory/items (alias for list_items, same endpoint)
    pub async fn list_items_catalog(&self) -> Result<Vec<Item>, String> {
        self.list_items().await
    }

    // ── Purchasing ──

    /// GET /api/purchase-orders
    pub async fn list_purchase_orders(&self) -> Result<Vec<PurchaseOrder>, String> {
        let url = format!("{}/api/purchase-orders", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let orders: Vec<PurchaseOrder> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(orders)
    }

    /// GET /api/purchase-orders/:id
    pub async fn get_purchase_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/purchase-orders
    pub async fn create_purchase_order(&self, form: &PurchaseOrderForm) -> Result<PurchaseOrder, String> {
        let url = format!("{}/api/purchase-orders", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let order: PurchaseOrder = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(order)
    }

    /// PUT /api/purchase-orders/:id
    pub async fn update_purchase_order(&self, id: i64, form: &PurchaseOrderForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/purchases
    pub async fn create_direct_purchase(&self, form: &DirectPurchaseForm) -> Result<DirectPurchase, String> {
        let url = format!("{}/api/purchases", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let purchase: DirectPurchase = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(purchase)
    }

    /// POST /api/purchase-orders/:id/status
    pub async fn update_po_status(&self, id: i64, status: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}/status", base_url(), id);
        let body_json = serde_json::json!({ "status": status });
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(&body_json)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/purchase-orders/:po_id/receipts
    pub async fn list_po_receipts(&self, po_id: i64) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-orders/{}/receipts", base_url(), po_id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/purchase-orders/pending
    pub async fn list_pending_pos(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-orders/pending", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/purchase-orders/:po_id/receipts
    pub async fn create_goods_receipt(&self, po_id: i64, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}/receipts", base_url(), po_id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Quotation endpoints ──

    /// GET /api/sales/quotations
    pub async fn list_quotations(&self) -> Result<Vec<Quotation>, String> {
        let url = format!("{}/api/sales/quotations", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Quotation> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/sales/quotations/{id}
    pub async fn get_quotation(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/quotations/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/sales/quotations
    pub async fn create_quotation(&self, form: &QuotationForm) -> Result<Quotation, String> {
        let url = format!("{}/api/sales/quotations", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let q: Quotation = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(q)
    }

    /// POST /api/sales/quotations/:id/convert
    pub async fn convert_quotation(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/quotations/{}/convert", base_url(), id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Sales Order endpoints ──

    /// GET /api/sales/sales-orders
    pub async fn list_sales_orders(&self) -> Result<Vec<SalesOrder>, String> {
        let url = format!("{}/api/sales/sales-orders", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<SalesOrder> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/sales/sales-orders/{id}
    pub async fn get_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/sales/sales-orders
    pub async fn create_sales_order(&self, form: &SalesOrderForm) -> Result<SalesOrder, String> {
        let url = format!("{}/api/sales/sales-orders", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let order: SalesOrder = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(order)
    }

    /// POST /api/sales/sales-orders/:id/cancel
    pub async fn cancel_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}/cancel", base_url(), id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/sales/sales-orders/:id/convert
    pub async fn convert_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}/convert", base_url(), id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/sales-returns
    pub async fn list_sales_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/sales-returns", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    // ── Employee endpoints ──

    /// GET /api/employees
    pub async fn list_employees(&self) -> Result<Vec<Employee>, String> {
        let url = format!("{}/api/employees", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let employees: Vec<Employee> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employees)
    }

    /// GET /api/employees/{id}
    pub async fn get_employee(&self, id: i64) -> Result<Employee, String> {
        let url = format!("{}/api/employees/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let employee: Employee = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employee)
    }

    /// POST /api/employees
    pub async fn create_employee(&self, form: &EmployeeForm) -> Result<Employee, String> {
        let url = format!("{}/api/employees", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let employee: Employee = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employee)
    }

    /// PUT /api/employees/{id}
    pub async fn update_employee(&self, id: i64, form: &EmployeeForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/employees/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/employees/{employee_id}/salary
    pub async fn pay_salary(&self, employee_id: i64, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/employees/{}/salary", base_url(), employee_id);
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Role endpoints ──

    /// GET /api/roles
    pub async fn list_roles(&self) -> Result<Vec<Role>, String> {
        let url = format!("{}/api/roles", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Role> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/roles/{id}
    pub async fn get_role(&self, id: i64) -> Result<Role, String> {
        let url = format!("{}/api/roles/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let item: Role = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// GET /api/roles/permissions
    pub async fn list_all_permissions(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/roles/permissions", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/roles/{role_id}/permissions
    pub async fn get_role_permissions(&self, role_id: i64) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/roles/{}/permissions", base_url(), role_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// PUT /api/roles/{role_id}/permissions
    pub async fn update_role_permissions(&self, role_id: i64, permissions: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles/{}/permissions", base_url(), role_id);
        let resp = self.inner.put(&url).headers(self.headers()).json(permissions).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/roles
    pub async fn create_role(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// DELETE /api/roles/{id}
    pub async fn delete_role(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles/{}", base_url(), id);
        let resp = self.inner.delete(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/roles/{id}/users
    pub async fn list_role_users(&self, role_id: i64) -> Result<Vec<User>, String> {
        let url = format!("{}/api/roles/{}/users", base_url(), role_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<User> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── User endpoints ──

    /// GET /api/users
    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        let url = format!("{}/api/users", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<User> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/users/{id}
    pub async fn get_user(&self, id: i64) -> Result<User, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let item: User = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// PUT /api/users/{id}
    pub async fn update_user(&self, id: i64, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/users
    pub async fn create_user(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// DELETE /api/users/{id}
    pub async fn delete_user(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self.inner.delete(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/users/{id}/reset-password
    pub async fn reset_user_password(&self, id: i64, new_password: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}/reset-password", base_url(), id);
        let body_json = serde_json::json!({ "password": new_password });
        let resp = self.inner.put(&url).headers(self.headers()).json(&body_json).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/users/{id}/toggle-status
    pub async fn toggle_user_status(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}/toggle-status", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Expense endpoints ──

    /// GET /api/expenses
    pub async fn list_expenses(&self) -> Result<Vec<Expense>, String> {
        let url = format!("{}/api/expenses", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Expense> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/expenses
    pub async fn create_expense(&self, form: &ExpenseForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/expenses", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/expenses/categories
    pub async fn list_expense_categories(&self) -> Result<Vec<ExpenseCategory>, String> {
        let url = format!("{}/api/expenses/categories", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<ExpenseCategory> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── BOM endpoints ──

    /// GET /api/bom
    pub async fn list_boms(&self) -> Result<Vec<Bom>, String> {
        let url = format!("{}/api/bom", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Bom> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/bom
    pub async fn create_bom(&self, form: &BomForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/bom", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/bom/{id}
    pub async fn get_bom(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/bom/{}", base_url(), id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/bom/{id}
    pub async fn update_bom(&self, id: i64, form: &BomForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/bom/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PATCH /api/bom/{id}/toggle-active
    pub async fn toggle_bom_active(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/bom/{}/toggle-active", base_url(), id);
        let resp = self.inner.request(reqwest::Method::PATCH, &url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/bom/by-item/{item_id}
    pub async fn get_bom_by_item(&self, item_id: i64) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/bom/by-item/{}", base_url(), item_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    // ── Production endpoints ──

    /// GET /api/production/productions
    pub async fn list_production_orders(&self) -> Result<Vec<Production>, String> {
        let url = format!("{}/api/production/productions", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Production> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/production/productions
    pub async fn create_production(&self, form: &ProductionForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/production/productions", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/production/productions/{id}
    pub async fn get_production(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/production/productions/{}", base_url(), id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/production/productions/summary/item/{item_id}
    pub async fn get_production_item_summary(&self, item_id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/production/productions/summary/item/{}", base_url(), item_id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/production/productions/{id}
    pub async fn update_production(&self, id: i64, form: &ProductionForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/production/productions/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Forecast endpoints ──

    /// GET /api/forecasts
    pub async fn list_forecasts(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/forecasts", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// POST /api/forecasts/run
    pub async fn run_forecast(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/forecasts/run", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/forecasts/runs
    pub async fn list_forecast_runs(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/forecasts/runs", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/forecasts/accuracy
    pub async fn get_forecast_accuracy(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/forecasts/accuracy", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/forecasts/config
    pub async fn list_forecast_configs(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/forecasts/config", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/forecasts/config/{id}
    pub async fn get_forecast_config(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/forecasts/config/{}", base_url(), id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/forecasts/config
    pub async fn create_forecast_config(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/forecasts/config", base_url());
        let resp = self.inner.post(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/forecasts/config/{id}
    pub async fn update_forecast_config(&self, id: i64, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/forecasts/config/{}", base_url(), id);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/forecasts/seasonal-events
    pub async fn list_seasonal_events(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/forecasts/seasonal-events", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/forecasts/seasonal-events/{id}
    pub async fn get_seasonal_event(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/forecasts/seasonal-events/{}", base_url(), id);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Accounting endpoints ──

    /// GET /api/accounting/accounts
    /// Response: { success: bool, data: Vec<ChartOfAccount> }
    pub async fn list_accounts(&self) -> Result<Vec<ChartOfAccount>, String> {
        let url = format!("{}/api/accounting/accounts", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        if body.get("success").and_then(|v| v.as_bool()) != Some(true) {
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }
        let items: Vec<ChartOfAccount> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/accounting/periods
    pub async fn list_accounting_periods(&self) -> Result<Vec<AccountingPeriod>, String> {
        let url = format!("{}/api/accounting/periods", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<AccountingPeriod> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Activity Log endpoints ──

    /// GET /api/activity-logs
    pub async fn list_activity_logs(&self) -> Result<Vec<ActivityLog>, String> {
        let url = format!("{}/api/activity-logs", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<ActivityLog> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Accounting endpoints (balances) ──

    /// GET /api/accounting/accounts/balances
    pub async fn list_account_balances(&self) -> Result<Vec<AccountBalance>, String> {
        let url = format!("{}/api/accounting/accounts/balances", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<AccountBalance> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Integration endpoints ──

    /// GET /api/integrations
    pub async fn list_integrations(&self) -> Result<Vec<Integration>, String> {
        let url = format!("{}/api/integrations", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Integration> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/integrations/{service}
    pub async fn get_integration(&self, service: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/integrations/{}", base_url(), service);
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/integrations/{service}
    pub async fn update_integration(&self, service: &str, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/integrations/{}", base_url(), service);
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Settings endpoints ──

    /// GET /api/settings
    pub async fn get_settings(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/settings", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/settings
    pub async fn update_settings(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/settings", base_url());
        let resp = self.inner.put(&url).headers(self.headers()).json(form).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Tax & Payment endpoints ──

    /// GET /api/tax-rates
    pub async fn list_tax_rates(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/tax-rates", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/payment-terms
    pub async fn list_payment_terms(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/payment-terms", base_url());
        let resp = self.inner.get(&url).headers(self.headers()).send().await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"].as_str().unwrap_or("Request failed").to_string());
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
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
