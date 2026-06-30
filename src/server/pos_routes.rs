use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/pos/sale", post(create_pos_sale))
        .route("/api/pos/transactions", get(list_pos_transactions))
}

async fn create_pos_sale(State(_state): State<AppState>, Json(form): Json<serde_json::Value>) -> impl IntoResponse {
    let customer_name = form.get("customer_name").and_then(|v| v.as_str()).unwrap_or("Walk-in Customer");
    let items = form.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let payment_method = form.get("payment_method").and_then(|v| v.as_str()).unwrap_or("Cash");
    let paid_amount = form.get("paid_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);

    if items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one item is required." })));
    }

    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", [], |row| row.get(0)).unwrap_or(1);
    let invoice_no = format!("POS-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut total_amount = 0.0;
    for item in &items {
        let qty = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let price = item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        total_amount += qty * price;
    }

    let result = db.execute(
        "INSERT INTO invoices (invoice_no, customer_id, source_type, invoice_date, due_date, status, total_amount, paid_amount, balance_amount)
         VALUES (?1, 1, 'POS', ?2, ?2, 'Paid', ?3, ?3, 0)",
        rusqlite::params![invoice_no, today, total_amount],
    );

    match result {
        Ok(_) => {
            let inv_id = db.last_insert_rowid();
            for item in &items {
                let item_id = item.get("item_id").and_then(|v| v.as_i64()).unwrap_or(0);
                let qty = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let price = item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let amount = qty * price;
                db.execute(
                    "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount)
                     VALUES (?1, ?2, '', ?3, ?4, ?5)",
                    rusqlite::params![inv_id, item_id, qty, price, amount],
                ).ok();
                db.execute(
                    "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![qty, item_id],
                ).ok();
            }
            if paid_amount > 0.0 {
                let pseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM payments", [], |row| row.get(0)).unwrap_or(1);
                let pno = format!("PAY-{}-{:04}", chrono::Utc::now().format("%Y"), pseq);
                db.execute(
                    "INSERT INTO payments (payment_no, customer_id, invoice_id, payment_date, amount, payment_method) VALUES (?1, 1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![pno, inv_id, today, paid_amount, payment_method],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": inv_id, "invoice_no": invoice_no, "total_amount": total_amount } })))
        }
        Err(e) => {
            tracing::error!("Failed to create POS sale: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create POS sale." })))
        }
    }
}

async fn list_pos_transactions(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, invoice_no, customer_id, invoice_date, total_amount, paid_amount, status
         FROM invoices WHERE source_type = 'POS' ORDER BY created_at DESC LIMIT 100"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "invoice_no": row.get::<_, String>(1)?,
            "customer_id": row.get::<_, i64>(2)?,
            "invoice_date": row.get::<_, String>(3)?,
            "total_amount": row.get::<_, f64>(4)?,
            "paid_amount": row.get::<_, f64>(5)?,
            "status": row.get::<_, String>(6)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}
