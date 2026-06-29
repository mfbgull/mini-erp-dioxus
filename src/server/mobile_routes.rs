use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/mobile-invoices/draft", post(save_draft).get(list_drafts))
        .route("/api/mobile-invoices/draft/{id}", get(get_draft).put(update_draft).delete(delete_draft))
        .route("/api/mobile-invoices/items/search", get(search_items))
        .route("/api/mobile-invoices/customers/search", get(search_customers))
        .route("/api/mobile-invoices/tax-rates", get(list_tax_rates))
        .route("/api/mobile-invoices/payment-terms", get(list_payment_terms))
        .route("/api/mobile-invoices/submit", post(submit_from_mobile))
}

async fn save_draft(State(_state): State<AppState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let session_id = body.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
    let customer_id = body.get("customer_id").and_then(|v| v.as_i64());
    let items_data = body.get("items_data").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let expires = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let result = db.execute(
        "INSERT INTO invoice_drafts (session_id, customer_id, items_data, status, expires_at) VALUES (?1, ?2, ?3, 'active', ?4)",
        rusqlite::params![session_id, customer_id, items_data, expires],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to save draft: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to save draft." }))) }
    }
}

async fn list_drafts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, session_id, customer_id, items_data, status, expires_at, created_at FROM invoice_drafts WHERE status = 'active' ORDER BY created_at DESC").unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "session_id": row.get::<_, String>(1)?,
            "customer_id": row.get::<_, Option<i64>>(2)?, "items_data": row.get::<_, String>(3)?,
            "status": row.get::<_, String>(4)?, "expires_at": row.get::<_, String>(5)?,
            "created_at": row.get::<_, String>(6)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_draft(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, session_id, customer_id, items_data, status, expires_at, created_at FROM invoice_drafts WHERE id = ?1",
        [id],
        |row| Ok(json!({
            "id": row.get::<_, i64>(0)?, "session_id": row.get::<_, String>(1)?,
            "customer_id": row.get::<_, Option<i64>>(2)?, "items_data": row.get::<_, String>(3)?,
            "status": row.get::<_, String>(4)?, "expires_at": row.get::<_, String>(5)?,
            "created_at": row.get::<_, String>(6)?,
        })),
    );
    match result {
        Ok(d) => (StatusCode::OK, Json(json!({ "success": true, "data": d }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Draft not found." }))),
    }
}

async fn update_draft(State(_state): State<AppState>, Path(id): Path<i64>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items_data = body.get("items_data").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let customer_id = body.get("customer_id").and_then(|v| v.as_i64());
    let result = db.execute(
        "UPDATE invoice_drafts SET customer_id = ?1, items_data = ?2 WHERE id = ?3",
        rusqlite::params![customer_id, items_data, id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Draft updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Draft not found." }))),
        Err(e) => { tracing::error!("Failed to update draft: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update draft." }))) }
    }
}

async fn delete_draft(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("DELETE FROM invoice_drafts WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Draft deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Draft not found." }))),
        Err(e) => { tracing::error!("Failed to delete draft: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete draft." }))) }
    }
}

async fn search_items(State(_state): State<AppState>, axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let db = db::get_db().lock().unwrap();
    let like = format!("%{}%", q);
    let mut stmt = db.prepare(
        "SELECT id, item_code, item_name, selling_price, current_stock FROM items
         WHERE is_active = 1 AND (item_name LIKE ?1 OR item_code LIKE ?1) ORDER BY item_name LIMIT 20"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([&like], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "item_code": row.get::<_, String>(1)?,
            "item_name": row.get::<_, String>(2)?, "selling_price": row.get::<_, f64>(3)?,
            "current_stock": row.get::<_, f64>(4)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn search_customers(State(_state): State<AppState>, axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let db = db::get_db().lock().unwrap();
    let like = format!("%{}%", q);
    let mut stmt = db.prepare(
        "SELECT id, customer_code, customer_name, phone, current_balance FROM customers
         WHERE is_active = 1 AND (customer_name LIKE ?1 OR customer_code LIKE ?1 OR phone LIKE ?1)
         ORDER BY customer_name LIMIT 20"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([&like], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "customer_code": row.get::<_, String>(1)?,
            "customer_name": row.get::<_, String>(2)?, "phone": row.get::<_, String>(3)?,
            "current_balance": row.get::<_, f64>(4)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_tax_rates(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, rate FROM tax_rates WHERE is_active = 1").unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({ "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?, "rate": row.get::<_, f64>(2)? }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_payment_terms(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, days FROM payment_terms WHERE is_active = 1").unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({ "id": row.get::<_, i64>(0)?, "name": row.get::<_, String>(1)?, "days": row.get::<_, i64>(2)? }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn submit_from_mobile(State(_state): State<AppState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let customer_id = body.get("customer_id").and_then(|v| v.as_i64()).unwrap_or(0);
    let items = body.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let payment_method = body.get("payment_method").and_then(|v| v.as_str()).unwrap_or("Cash");
    let paid_amount = body.get("paid_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    if items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "No items provided." })));
    }

    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", [], |row| row.get(0)).unwrap_or(1);
    let invoice_no = format!("INV-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let mut total = 0.0;
    for item in &items {
        let qty = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let price = item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        total += qty * price;
    }

    let status = if paid_amount >= total { "Paid" } else if paid_amount > 0.0 { "Partially Paid" } else { "Unpaid" };
    let result = db.execute(
        "INSERT INTO invoices (invoice_no, customer_id, source_type, invoice_date, due_date, status, total_amount, paid_amount, balance_amount)
         VALUES (?1, ?2, 'MOBILE', ?3, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![invoice_no, customer_id, today, status, total, paid_amount, total - paid_amount],
    );
    match result {
        Ok(_) => {
            let inv_id = db.last_insert_rowid();
            for item in &items {
                let item_id = item.get("item_id").and_then(|v| v.as_i64()).unwrap_or(0);
                let qty = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let price = item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                db.execute(
                    "INSERT INTO invoice_items (invoice_id, item_id, quantity, unit_price, amount) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![inv_id, item_id, qty, price, qty * price],
                ).ok();
            }
            if paid_amount > 0.0 {
                let pseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM payments", [], |row| row.get(0)).unwrap_or(1);
                let pno = format!("PAY-{}-{:04}", chrono::Utc::now().format("%Y"), pseq);
                db.execute(
                    "INSERT INTO payments (payment_no, customer_id, invoice_id, payment_date, amount, payment_method) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![pno, customer_id, inv_id, today, paid_amount, payment_method],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": inv_id, "invoice_no": invoice_no } })))
        }
        Err(e) => { tracing::error!("Failed to submit mobile invoice: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to submit invoice." }))) }
    }
}
