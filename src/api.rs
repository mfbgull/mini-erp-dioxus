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
