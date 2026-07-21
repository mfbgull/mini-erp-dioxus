use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
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
        // Returns
        .route("/api/sales-returns", get(list_sales_returns))
}

// ============================================================================
// Sales Orders
// ============================================================================

async fn list_sales_orders(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT so.id, so.so_no, so.customer_id, c.customer_name, c.customer_code, so.so_date, so.status,
                so.delivery_date, so.source_type, so.source_id, so.total_amount, so.warehouse_id, so.notes,
                so.created_by, so.created_at, so.updated_at,
                (SELECT COUNT(*) FROM sales_order_items soi WHERE soi.so_id = so.id) AS item_count
         FROM sales_orders so LEFT JOIN customers c ON so.customer_id = c.id
         ORDER BY so.created_at DESC"
    ).unwrap();
    let items: Vec<SalesOrder> = stmt.query_map([], |row| {
        Ok(SalesOrder {
            id: row.get(0)?, so_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, customer_code: row.get(4)?, so_date: row.get(5)?,
            status: row.get(6)?, delivery_date: row.get(7)?,
            source_type: row.get(8)?, source_id: row.get(9)?, total_amount: row.get(10)?,
            warehouse_id: row.get(11)?, notes: row.get(12)?, created_by: row.get(13)?,
            created_at: row.get(14)?, updated_at: row.get(15)?,
            item_count: row.get(16)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT so.id, so.so_no, so.customer_id, c.customer_name, c.customer_code, so.so_date, so.status,
                so.delivery_date, so.source_type, so.source_id, so.total_amount, so.warehouse_id, so.notes,
                so.created_by, so.created_at, so.updated_at,
                (SELECT COUNT(*) FROM sales_order_items soi WHERE soi.so_id = so.id) AS item_count
         FROM sales_orders so LEFT JOIN customers c ON so.customer_id = c.id WHERE so.id = ?1",
        [id],
        |row| Ok(SalesOrder {
            id: row.get(0)?, so_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, customer_code: row.get(4)?, so_date: row.get(5)?,
            status: row.get(6)?, delivery_date: row.get(7)?,
            source_type: row.get(8)?, source_id: row.get(9)?, total_amount: row.get(10)?,
            warehouse_id: row.get(11)?, notes: row.get(12)?, created_by: row.get(13)?,
            created_at: row.get(14)?, updated_at: row.get(15)?,
            item_count: row.get(16)?,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
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
                if let Err(e) = db.execute(
                    "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![so_id, item.item_id, item.description.as_deref().unwrap_or(""), item.quantity, item.unit_price, amount],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create SO item: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create sales order item." })));
                }
            }
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit SO: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": so_id, "so_no": so_no } })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create sales order." }))) }
    }
}

async fn update_sales_order(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SalesOrderForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    db.execute("DELETE FROM sales_order_items WHERE so_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM sales_orders WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Sales order deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Sales order not found." }))),
        Err(e) => { tracing::error!("Failed to delete SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete sales order." }))) }
    }
}

async fn cancel_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE sales_orders SET status = 'Cancelled', updated_at = datetime('now') WHERE id = ?1 AND status != 'Cancelled'", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Sales order cancelled." } }))),
        Ok(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Sales order not found or already cancelled." }))),
        Err(e) => { tracing::error!("Failed to cancel SO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel." }))) }
    }
}

async fn convert_sales_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    let so = db.query_row(
        "SELECT id, customer_id, warehouse_id FROM sales_orders WHERE id = ?1 AND status = 'Pending'",
        [id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<i64>>(2)?)),
    );
    match so {
        Ok((so_id, customer_id, warehouse_id)) => {
            let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", [], |row| row.get(0)).unwrap_or(1);
            let inv_no = format!("INV-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let wh_id = warehouse_id.unwrap_or(1);

            // SELECT sales_order_items to copy
            let so_items: Vec<(i64, String, f64, f64, f64)> = {
                let mut stmt = db.prepare(
                    "SELECT item_id, COALESCE(description, ''), quantity, unit_price, amount FROM sales_order_items WHERE so_id = ?1"
                ).unwrap();
                stmt.query_map([id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                }).unwrap().filter_map(|r| r.ok()).collect()
            };

            // Compute total from line items
            let total: f64 = so_items.iter().map(|(_, _, _, _, amt)| amt).sum();

            // Insert invoice header
            if let Err(e) = db.execute(
                "INSERT INTO invoices (invoice_no, customer_id, so_id, source_type, invoice_date, due_date, status, total_amount, paid_amount, balance_amount, warehouse_id)
                 VALUES (?1, ?2, ?3, 'SALES_ORDER', ?4, ?4, 'Unpaid', ?5, 0, ?5, ?6)",
                rusqlite::params![inv_no, customer_id, so_id, today, total, wh_id],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to insert invoice: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
            }
            let inv_id = db.last_insert_rowid();

            // Copy line items and create stock movements
            for (item_id, description, quantity, unit_price, amount) in &so_items {
                // Insert invoice item
                if let Err(e) = db.execute(
                    "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount, tax_rate)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                    rusqlite::params![inv_id, item_id, description, quantity, unit_price, amount],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to copy SO items to invoice: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }

                // Create stock movement OUT
                let unit_cost: f64 = db.query_row(
                    "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1", [*item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);
                let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
                if let Err(e) = db.execute(
                    "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                     VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'INVOICE', ?6, ?7)",
                    rusqlite::params![mno, item_id, wh_id, quantity, unit_cost, inv_no, format!("SO Conversion {}", inv_no)],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create stock movement: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }

                // Update stock
                if let Err(e) = db.execute(
                    "UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![quantity, item_id, wh_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update stock_balances: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![quantity, item_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update current_stock: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
            }

            // Customer ledger entry
            {
                let last_balance: f64 = db.query_row(
                    "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                    [customer_id], |row| row.get(0),
                ).unwrap_or(0.0);
                if let Err(e) = db.execute(
                    "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                     VALUES (?1, ?2, 'INVOICE', ?3, ?4, 0, ?5)",
                    rusqlite::params![customer_id, &today, inv_no, total, last_balance + total],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert customer ledger: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE customers SET current_balance = current_balance + ?1 WHERE id = ?2",
                    rusqlite::params![total, customer_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update customer balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
            }

            // Journal entry: debit AR (2), credit Revenue (11)
            {
                if let Err(e) = db.execute(
                    "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('invoice', ?1, ?2)",
                    rusqlite::params![inv_id, &today],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert journal entry: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
                let je_id = db.last_insert_rowid();
                if let Err(e) = db.execute(
                    "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                     VALUES (?1, 2, ?2, 0, ?3, ?4),
                            (?1, 11, 0, ?2, ?5, ?4)",
                    rusqlite::params![je_id, total, format!("Invoice {}", inv_no), &today, format!("Revenue - Invoice {}", inv_no)],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert journal lines: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
                }
            }

            // Update SO status
            if let Err(e) = db.execute("UPDATE sales_orders SET status = 'Converted', updated_at = datetime('now') WHERE id = ?1", [id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update SO status: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert SO (transaction rolled back)." })));
            }

            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit SO conversion: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit (transaction rolled back)." })));
            }

            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "SO converted to invoice.", "invoice_no": inv_no, "item_count": so_items.len() } })))
        }
        Err(_) => {
            let _ = db.execute_batch("ROLLBACK");
            (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Sales order not found or not in Pending status." })))
        }
    }
}

async fn so_cycle_chain(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
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
                if let Err(e) = db.execute(
                    "INSERT INTO quotation_items (quotation_id, item_id, description, quantity, unit_price, discount, tax, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![q_id, item.item_id, item.description.as_deref().unwrap_or(""),
                        item.quantity, item.unit_price, disc, tax, amount],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create quotation item: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create quotation item." })));
                }
            }
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit quotation: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": q_id, "quotation_no": qno } })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create quotation: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create quotation." }))) }
    }
}

async fn update_quotation(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<QuotationForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    db.execute("DELETE FROM quotation_items WHERE quotation_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM quotations WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Quotation deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
        Err(e) => { tracing::error!("Failed to delete quotation: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete quotation." }))) }
    }
}

async fn convert_quotation(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    let q = db.query_row(
        "SELECT id, customer_id FROM quotations WHERE id = ?1 AND status = 'Draft'",
        [id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    );
    match q {
        Ok((q_id, customer_id)) => {
            let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM sales_orders", [], |row| row.get(0)).unwrap_or(1);
            let so_no = format!("SO-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

            // SELECT quotation_items to copy
            let q_items: Vec<(i64, String, f64, f64, f64, f64)> = {
                let mut stmt = db.prepare(
                    "SELECT item_id, COALESCE(description, ''), quantity, unit_price, discount, amount FROM quotation_items WHERE quotation_id = ?1"
                ).unwrap();
                stmt.query_map([id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
                }).unwrap().filter_map(|r| r.ok()).collect()
            };

            // Compute total from line items
            let total: f64 = q_items.iter().map(|(_, _, _, _, _, amt)| amt).sum();

            // Insert SO header
            if let Err(e) = db.execute(
                "INSERT INTO sales_orders (so_no, customer_id, so_date, status, source_type, source_id, total_amount)
                 VALUES (?1, ?2, ?3, 'Pending', 'QUOTATION', ?4, ?5)",
                rusqlite::params![so_no, customer_id, today, q_id, total],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to insert SO: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert quotation (transaction rolled back)." })));
            }
            let so_id = db.last_insert_rowid();

            // Copy line items
            for (item_id, description, quantity, unit_price, _discount, amount) in &q_items {
                if let Err(e) = db.execute(
                    "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![so_id, item_id, description, quantity, unit_price, amount],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to copy quotation items to SO: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert quotation (transaction rolled back)." })));
                }
            }

            // Update quotation status
            if let Err(e) = db.execute("UPDATE quotations SET status = 'Converted', updated_at = datetime('now') WHERE id = ?1", [id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update quotation status: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to convert quotation (transaction rolled back)." })));
            }

            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit quotation conversion: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit (transaction rolled back)." })));
            }

            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Quotation converted to SO.", "so_no": so_no, "item_count": q_items.len() } })))
        }
        Err(_) => {
            let _ = db.execute_batch("ROLLBACK");
            (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Quotation not found or not in Draft status." })))
        }
    }
}

async fn quotation_cycle_chain(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let q = db.query_row("SELECT quotation_no, status FROM quotations WHERE id = ?1", [id], |row| {
        Ok(json!({ "quotation_no": row.get::<_, String>(0)?, "status": row.get::<_, String>(1)? }))
    });
    match q {
        Ok(data) => (StatusCode::OK, Json(json!({ "success": true, "data": data }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Quotation not found." }))),
    }
}

async fn list_sales_returns(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT i.id, i.invoice_no, i.customer_id, c.customer_name, i.returned_amount, i.invoice_date
         FROM invoices i LEFT JOIN customers c ON i.customer_id = c.id
         WHERE i.returned_amount > 0 ORDER BY i.created_at DESC"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "invoice_no": row.get::<_, String>(1)?,
            "customer_id": row.get::<_, i64>(2)?,
            "customer_name": row.get::<_, Option<String>>(3)?,
            "returned_amount": row.get::<_, f64>(4)?,
            "invoice_date": row.get::<_, String>(5)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn sales_dashboard(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
