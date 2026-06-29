use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        // Sales Orders
        .route("/api/sales/sales-orders", get(list_sales_orders).post(create_sales_order))
        .route("/api/sales/sales-orders/{id}", get(get_sales_order).put(update_sales_order).delete(delete_sales_order))
        .route("/api/sales/sales-orders/{id}/cancel", post(cancel_sales_order))
        .route("/api/sales/sales-orders/{id}/convert", post(convert_sales_order))
        .route("/api/sales/sales-orders/{id}/cycle-chain", get(so_cycle_chain))
        // Quotations
        .route("/api/sales/quotations", get(list_quotations).post(create_quotation))
        .route("/api/sales/quotations/{id}", get(get_quotation).put(update_quotation).delete(delete_quotation))
        .route("/api/sales/quotations/{id}/convert", post(convert_quotation))
        .route("/api/sales/quotations/{id}/cycle-chain", get(quotation_cycle_chain))
        // Sales Dashboard
        .route("/api/sales/dashboard", get(sales_dashboard))
}

// ============================================================================
// Sales Orders
// ============================================================================

async fn list_sales_orders(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT so.id, so.so_no, so.customer_id, c.customer_name, so.so_date, so.status,
                so.source_type, so.source_id, so.total_amount, so.warehouse_id, so.notes,
                so.created_by, so.created_at, so.updated_at
         FROM sales_orders so LEFT JOIN customers c ON so.customer_id = c.id
         ORDER BY so.created_at DESC"
    ).unwrap();
    let items: Vec<SalesOrder> = stmt.query_map([], |row| {
        Ok(SalesOrder {
            id: row.get(0)?, so_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, so_date: row.get(4)?, status: row.get(5)?,
            source_type: row.get(6)?, source_id: row.get(7)?, total_amount: row.get(8)?,
            warehouse_id: row.get(9)?, notes: row.get(10)?, created_by: row.get(11)?,
            created_at: row.get(12)?, updated_at: row.get(13)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT so.id, so.so_no, so.customer_id, c.customer_name, so.so_date, so.status,
                so.source_type, so.source_id, so.total_amount, so.warehouse_id, so.notes,
                so.created_by, so.created_at, so.updated_at
         FROM sales_orders so LEFT JOIN customers c ON so.customer_id = c.id WHERE so.id = ?1",
        [id],
        |row| Ok(SalesOrder {
            id: row.get(0)?, so_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, so_date: row.get(4)?, status: row.get(5)?,
            source_type: row.get(6)?, source_id: row.get(7)?, total_amount: row.get(8)?,
            warehouse_id: row.get(9)?, notes: row.get(10)?, created_by: row.get(11)?,
            created_at: row.get(12)?, updated_at: row.get(13)?,
        }),
    );
    match result {
        Ok(so) => {
            let mut stmt = db.prepare(
                "SELECT si.id, si.so_id, si.item_id, i.item_name, i.item_code,
                        si.description, si.quantity, si.delivered_quantity, si.unit_price, si.amount
                 FROM sales_order_items si LEFT JOIN items i ON si.item_id = i.id
                 WHERE si.so_id = ?1"
            ).unwrap();
            let items: Vec<SalesOrderItem> = stmt.query_map([id], |row| {
                Ok(SalesOrderItem {
                    id: row.get(0)?, so_id: row.get(1)?, item_id: row.get(2)?,
                    item_name: row.get(3)?, item_code: row.get(4)?, description: row.get(5)?,
                    quantity: row.get(6)?, delivered_quantity: row.get(7)?,
                    unit_price: row.get(8)?, amount: row.get(9)?,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "sales_order": so, "items": items } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Sales order not found." }))),
    }
}

async fn create_sales_order(State(_state): State<AppState>, Json(form): Json<SalesOrderForm>) -> impl IntoResponse {
    if form.items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one item is required." })));
    }
    let db = db::get_db().lock().unwrap();
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM sales_orders", [], |row| row.get(0)).unwrap_or(1);
    let so_no = format!("SO-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let total: f64 = form.items.iter().map(|i| i.quantity * i.unit_price).sum();

    let result = db.execute(
        "INSERT INTO sales_orders (so_no, customer_id, so_date, status, total_amount, warehouse_id, notes)
         VALUES (?1, ?2, ?3, 'Pending', ?4, ?5, ?6)",
        rusqlite::params![so_no, form.customer_id, form.so_date, total, form.warehouse_id, form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let so_id = db.last_insert_rowid();
            for item in &form.items {
                let amount = item.quantity * item.unit_price;
                db.execute(
                    "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![so_id, item.item_id, item.description.as_deref().unwrap_or(""), item.quantity, item.unit_price, amount],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": so_id, "so_no": so_no } })))
        }
        Err(e) => { tracing::error!("Failed to create SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create sales order." }))) }
    }
}

async fn update_sales_order(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SalesOrderForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let total: f64 = form.items.iter().map(|i| i.quantity * i.unit_price).sum();
    let result = db.execute(
        "UPDATE sales_orders SET customer_id=?1, so_date=?2, total_amount=?3, warehouse_id=?4, notes=?5, updated_at=datetime('now') WHERE id=?6",
        rusqlite::params![form.customer_id, form.so_date, total, form.warehouse_id, form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            db.execute("DELETE FROM sales_order_items WHERE so_id = ?1", [id]).ok();
            for item in &form.items {
                let amount = item.quantity * item.unit_price;
                db.execute(
                    "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![id, item.item_id, item.description.as_deref().unwrap_or(""), item.quantity, item.unit_price, amount],
                ).ok();
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Sales order updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Sales order not found." }))),
        Err(e) => { tracing::error!("Failed to update SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update sales order." }))) }
    }
}

async fn delete_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    db.execute("DELETE FROM sales_order_items WHERE so_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM sales_orders WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Sales order deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Sales order not found." }))),
        Err(e) => { tracing::error!("Failed to delete SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete sales order." }))) }
    }
}

async fn cancel_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE sales_orders SET status = 'Cancelled', updated_at = datetime('now') WHERE id = ?1 AND status != 'Cancelled'", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Sales order cancelled." } }))),
        Ok(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Sales order not found or already cancelled." }))),
        Err(e) => { tracing::error!("Failed to cancel SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel." }))) }
    }
}

async fn convert_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let so = db.query_row(
        "SELECT id, customer_id, warehouse_id, total_amount FROM sales_orders WHERE id = ?1 AND status = 'Pending'",
        [id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<i64>>(2)?, row.get::<_, f64>(3)?)),
    );
    match so {
        Ok((so_id, customer_id, warehouse_id, total)) => {
            let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", [], |row| row.get(0)).unwrap_or(1);
            let inv_no = format!("INV-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            db.execute(
                "INSERT INTO invoices (invoice_no, customer_id, so_id, source_type, invoice_date, due_date, status, total_amount, paid_amount, balance_amount, warehouse_id)
                 VALUES (?1, ?2, ?3, 'SALES_ORDER', ?4, ?4, 'Unpaid', ?5, 0, ?5, ?6)",
                rusqlite::params![inv_no, customer_id, so_id, today, total, warehouse_id],
            ).ok();
            db.execute("UPDATE sales_orders SET status = 'Converted', updated_at = datetime('now') WHERE id = ?1", [id]).ok();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "SO converted to invoice.", "invoice_no": inv_no } })))
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Sales order not found or not in Pending status." }))),
    }
}

async fn so_cycle_chain(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let so = db.query_row("SELECT so_no, quotation_id FROM sales_orders WHERE id = ?1", [id], |row| {
        Ok(json!({ "so_no": row.get::<_, String>(0)?, "quotation_id": row.get::<_, Option<i64>>(1)? }))
    });
    match so {
        Ok(so_data) => (StatusCode::OK, Json(json!({ "success": true, "data": so_data }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Sales order not found." }))),
    }
}

// ============================================================================
// Quotations
// ============================================================================

async fn list_quotations(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT q.id, q.quotation_no, q.customer_id, c.customer_name, q.quotation_date,
                q.expiry_date, q.status, q.total_amount, q.notes, q.created_by, q.created_at, q.updated_at
         FROM quotations q LEFT JOIN customers c ON q.customer_id = c.id
         ORDER BY q.created_at DESC"
    ).unwrap();
    let items: Vec<Quotation> = stmt.query_map([], |row| {
        Ok(Quotation {
            id: row.get(0)?, quotation_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, quotation_date: row.get(4)?, expiry_date: row.get(5)?,
            status: row.get(6)?, total_amount: row.get(7)?, notes: row.get(8)?,
            created_by: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_quotation(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT q.id, q.quotation_no, q.customer_id, c.customer_name, q.quotation_date,
                q.expiry_date, q.status, q.total_amount, q.notes, q.created_by, q.created_at, q.updated_at
         FROM quotations q LEFT JOIN customers c ON q.customer_id = c.id WHERE q.id = ?1",
        [id],
        |row| Ok(Quotation {
            id: row.get(0)?, quotation_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, quotation_date: row.get(4)?, expiry_date: row.get(5)?,
            status: row.get(6)?, total_amount: row.get(7)?, notes: row.get(8)?,
            created_by: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?,
        }),
    );
    match result {
        Ok(q) => {
            let mut stmt = db.prepare(
                "SELECT qi.id, qi.quotation_id, qi.item_id, i.item_name, i.item_code,
                        qi.description, qi.quantity, qi.unit_price, qi.discount, qi.tax, qi.amount
                 FROM quotation_items qi LEFT JOIN items i ON qi.item_id = i.id
                 WHERE qi.quotation_id = ?1"
            ).unwrap();
            let items: Vec<QuotationItem> = stmt.query_map([id], |row| {
                Ok(QuotationItem {
                    id: row.get(0)?, quotation_id: row.get(1)?, item_id: row.get(2)?,
                    item_name: row.get(3)?, item_code: row.get(4)?, description: row.get(5)?,
                    quantity: row.get(6)?, unit_price: row.get(7)?, discount: row.get(8)?,
                    tax: row.get(9)?, amount: row.get(10)?,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "quotation": q, "items": items } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
    }
}

async fn create_quotation(State(_state): State<AppState>, Json(form): Json<QuotationForm>) -> impl IntoResponse {
    if form.items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one item is required." })));
    }
    let db = db::get_db().lock().unwrap();
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM quotations", [], |row| row.get(0)).unwrap_or(1);
    let qno = format!("QUO-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let total: f64 = form.items.iter().map(|i| {
        let sub = i.quantity * i.unit_price;
        sub - i.discount.unwrap_or(0.0) + sub * (i.tax_rate.unwrap_or(0.0) / 100.0)
    }).sum();

    let result = db.execute(
        "INSERT INTO quotations (quotation_no, customer_id, quotation_date, expiry_date, status, total_amount, notes)
         VALUES (?1, ?2, ?3, ?4, 'Draft', ?5, ?6)",
        rusqlite::params![qno, form.customer_id, form.quotation_date, form.expiry_date, total, form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let q_id = db.last_insert_rowid();
            for item in &form.items {
                let sub = item.quantity * item.unit_price;
                let disc = item.discount.unwrap_or(0.0);
                let tax = (sub - disc) * (item.tax_rate.unwrap_or(0.0) / 100.0);
                let amount = sub - disc + tax;
                db.execute(
                    "INSERT INTO quotation_items (quotation_id, item_id, description, quantity, unit_price, discount, tax, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![q_id, item.item_id, item.description.as_deref().unwrap_or(""),
                        item.quantity, item.unit_price, disc, tax, amount],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": q_id, "quotation_no": qno } })))
        }
        Err(e) => { tracing::error!("Failed to create quotation: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create quotation." }))) }
    }
}

async fn update_quotation(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<QuotationForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let total: f64 = form.items.iter().map(|i| {
        let sub = i.quantity * i.unit_price;
        sub - i.discount.unwrap_or(0.0) + sub * (i.tax_rate.unwrap_or(0.0) / 100.0)
    }).sum();
    let result = db.execute(
        "UPDATE quotations SET customer_id=?1, quotation_date=?2, expiry_date=?3, total_amount=?4, notes=?5, updated_at=datetime('now') WHERE id=?6",
        rusqlite::params![form.customer_id, form.quotation_date, form.expiry_date, total, form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            db.execute("DELETE FROM quotation_items WHERE quotation_id = ?1", [id]).ok();
            for item in &form.items {
                let sub = item.quantity * item.unit_price;
                let disc = item.discount.unwrap_or(0.0);
                let tax = (sub - disc) * (item.tax_rate.unwrap_or(0.0) / 100.0);
                let amount = sub - disc + tax;
                db.execute(
                    "INSERT INTO quotation_items (quotation_id, item_id, description, quantity, unit_price, discount, tax, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![id, item.item_id, item.description.as_deref().unwrap_or(""),
                        item.quantity, item.unit_price, disc, tax, amount],
                ).ok();
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Quotation updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
        Err(e) => { tracing::error!("Failed to update quotation: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update quotation." }))) }
    }
}

async fn delete_quotation(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    db.execute("DELETE FROM quotation_items WHERE quotation_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM quotations WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Quotation deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
        Err(e) => { tracing::error!("Failed to delete quotation: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete quotation." }))) }
    }
}

async fn convert_quotation(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let q = db.query_row(
        "SELECT id, customer_id, total_amount FROM quotations WHERE id = ?1 AND status = 'Draft'",
        [id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, f64>(2)?)),
    );
    match q {
        Ok((q_id, customer_id, total)) => {
            let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM sales_orders", [], |row| row.get(0)).unwrap_or(1);
            let so_no = format!("SO-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            db.execute(
                "INSERT INTO sales_orders (so_no, customer_id, so_date, status, source_type, source_id, total_amount)
                 VALUES (?1, ?2, ?3, 'Pending', 'QUOTATION', ?4, ?5)",
                rusqlite::params![so_no, customer_id, today, q_id, total],
            ).ok();
            db.execute("UPDATE quotations SET status = 'Converted', updated_at = datetime('now') WHERE id = ?1", [id]).ok();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Quotation converted to SO.", "so_no": so_no } })))
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Quotation not found or not in Draft status." }))),
    }
}

async fn quotation_cycle_chain(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let q = db.query_row("SELECT quotation_no, status FROM quotations WHERE id = ?1", [id], |row| {
        Ok(json!({ "quotation_no": row.get::<_, String>(0)?, "status": row.get::<_, String>(1)? }))
    });
    match q {
        Ok(data) => (StatusCode::OK, Json(json!({ "success": true, "data": data }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
    }
}

async fn sales_dashboard(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let total_invoices: i64 = db.query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0)).unwrap_or(0);
    let total_revenue: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled'", [], |row| row.get(0)).unwrap_or(0.0);
    let unpaid_amount: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE status IN ('Unpaid', 'Partially Paid')", [], |row| row.get(0)).unwrap_or(0.0);
    let today_sales: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE invoice_date = ?1", [&today], |row| row.get(0)).unwrap_or(0.0);
    let pending_sos: i64 = db.query_row("SELECT COUNT(*) FROM sales_orders WHERE status = 'Pending'", [], |row| row.get(0)).unwrap_or(0);
    let draft_quotations: i64 = db.query_row("SELECT COUNT(*) FROM quotations WHERE status = 'Draft'", [], |row| row.get(0)).unwrap_or(0);
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "total_invoices": total_invoices,
            "total_revenue": total_revenue,
            "unpaid_amount": unpaid_amount,
            "today_sales": today_sales,
            "pending_sales_orders": pending_sos,
            "draft_quotations": draft_quotations,
        }
    })))
}
