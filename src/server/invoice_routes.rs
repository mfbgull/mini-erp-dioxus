use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use std::collections::HashMap;

/// Compute the full line item amount including discount and tax.
fn calculate_line_item_amount(
    quantity: f64,
    unit_price: f64,
    discount_type: Option<&str>,
    discount_value: Option<f64>,
    tax_rate: Option<f64>,
) -> f64 {
    let base = quantity * unit_price;
    let discount = match discount_type {
        Some("percentage") => base * (discount_value.unwrap_or(0.0) / 100.0),
        Some("fixed") => discount_value.unwrap_or(0.0),
        _ => 0.0,
    };
    let taxable = (base - discount).max(0.0);
    let tax = taxable * (tax_rate.unwrap_or(0.0) / 100.0);
    taxable + tax
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/invoices", get(list_invoices).post(create_invoice))
        .route("/api/invoices/{id}", get(get_invoice).put(update_invoice))
        .route("/api/invoices/{id}/cancel", put(cancel_invoice))
        .route("/api/invoices/{id}/return", post(return_invoice))
        .route("/api/invoices/{id}/payments", get(invoice_payments))
        .route("/api/invoices/returns", get(list_returns))
        .route("/api/invoices/recalculate-totals", post(recalculate_invoice_totals))
}

async fn list_invoices(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT i.id, i.invoice_no, i.customer_id, c.customer_name, c.customer_code, i.so_id, i.quotation_id,
                i.source_type, i.invoice_date, i.due_date, i.status, i.total_amount,
                i.paid_amount, i.balance_amount, i.returned_amount, i.discount_scope,
                i.discount_type, i.discount_value, i.tax_rate, i.notes, i.warehouse_id,
                i.created_by, i.created_at, i.updated_at,
                (SELECT COUNT(*) FROM invoice_items ii WHERE ii.invoice_id = i.id) AS item_count
         FROM invoices i LEFT JOIN customers c ON i.customer_id = c.id
         ORDER BY i.created_at DESC"
    ).unwrap();
    let items: Vec<Invoice> = stmt.query_map([], |row| {
        Ok(Invoice {
            id: row.get(0)?, invoice_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, customer_code: row.get(4)?, so_id: row.get(5)?, quotation_id: row.get(6)?,
            source_type: row.get(7)?, invoice_date: row.get(8)?, due_date: row.get(9)?,
            status: row.get(10)?, total_amount: row.get(11)?, paid_amount: row.get(12)?,
            balance_amount: row.get(13)?, returned_amount: row.get(14)?,
            discount_scope: row.get(15)?, discount_type: row.get(16)?, discount_value: row.get(17)?,
            tax_rate: row.get(18)?, notes: row.get(19)?, warehouse_id: row.get(20)?,
            created_by: row.get(21)?, created_at: row.get(22)?, updated_at: row.get(23)?,
            item_count: row.get(24)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_invoice(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT i.id, i.invoice_no, i.customer_id, c.customer_name, c.customer_code, i.so_id, i.quotation_id,
                i.source_type, i.invoice_date, i.due_date, i.status, i.total_amount,
                i.paid_amount, i.balance_amount, i.returned_amount, i.discount_scope,
                i.discount_type, i.discount_value, i.tax_rate, i.notes, i.warehouse_id,
                i.created_by, i.created_at, i.updated_at,
                (SELECT COUNT(*) FROM invoice_items ii WHERE ii.invoice_id = i.id) AS item_count
         FROM invoices i LEFT JOIN customers c ON i.customer_id = c.id WHERE i.id = ?1",
        [id],
        |row| Ok(Invoice {
            id: row.get(0)?, invoice_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, customer_code: row.get(4)?, so_id: row.get(5)?, quotation_id: row.get(6)?,
            source_type: row.get(7)?, invoice_date: row.get(8)?, due_date: row.get(9)?,
            status: row.get(10)?, total_amount: row.get(11)?, paid_amount: row.get(12)?,
            balance_amount: row.get(13)?, returned_amount: row.get(14)?,
            discount_scope: row.get(15)?, discount_type: row.get(16)?, discount_value: row.get(17)?,
            tax_rate: row.get(18)?, notes: row.get(19)?, warehouse_id: row.get(20)?,
            created_by: row.get(21)?, created_at: row.get(22)?, updated_at: row.get(23)?,
            item_count: row.get(24)?,
        }),
    );
    match result {
        Ok(inv) => {
            let mut stmt = db.prepare(
                "SELECT ii.id, ii.invoice_id, ii.item_id, i.item_name, i.item_code,
                        ii.description, ii.quantity, ii.returned_qty, ii.unit_price, ii.amount,
                        ii.tax_rate, ii.discount_type, ii.discount_value
                 FROM invoice_items ii LEFT JOIN items i ON ii.item_id = i.id
                 WHERE ii.invoice_id = ?1"
            ).unwrap();
            let items: Vec<InvoiceItem> = stmt.query_map([id], |row| {
                Ok(InvoiceItem {
                    id: row.get(0)?, invoice_id: row.get(1)?, item_id: row.get(2)?,
                    item_name: row.get(3)?, item_code: row.get(4)?, description: row.get(5)?,
                    quantity: row.get(6)?, returned_qty: row.get(7)?, unit_price: row.get(8)?,
                    amount: row.get(9)?, tax_rate: row.get(10)?,
                    discount_type: row.get(11)?, discount_value: row.get(12)?,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "invoice": inv, "items": items } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Invoice not found." }))),
    }
}

async fn create_invoice(State(_state): State<AppState>, Json(form): Json<InvoiceForm>) -> impl IntoResponse {
    if form.items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one item is required." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Begin transaction — all operations must succeed or all rollback
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", [], |row| row.get(0)).unwrap_or(1);
    let invoice_no = format!("INV-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    let mut total_amount = 0.0;
    for item in &form.items {
        total_amount += calculate_line_item_amount(
            item.quantity, item.unit_price,
            item.discount_type.as_deref(), item.discount_value, item.tax_rate,
        );
    }
    let discount = match form.discount_type.as_deref() {
        Some("percentage") => total_amount * (form.discount_value.unwrap_or(0.0) / 100.0),
        Some("fixed") => form.discount_value.unwrap_or(0.0),
        _ => 0.0,
    };
    total_amount -= discount;
    let due_date = form.due_date.clone().unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let result = db.execute(
        "INSERT INTO invoices (invoice_no, customer_id, source_type, invoice_date, due_date, status,
            total_amount, paid_amount, balance_amount, discount_scope, discount_type, discount_value,
            tax_rate, notes, warehouse_id)
         VALUES (?1, ?2, ?3, ?4, ?5, 'Unpaid', ?6, 0, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![invoice_no, form.customer_id, form.source_type.as_deref().unwrap_or("DIRECT"),
            form.invoice_date, due_date, total_amount, form.discount_scope, form.discount_type,
            form.discount_value, form.tax_rate, form.notes.as_deref().unwrap_or(""), form.warehouse_id],
    );

    match result {
        Ok(_) => {
            let inv_id = db.last_insert_rowid();
            let mut txn_ok = true;
            for item in &form.items {
                let amount = calculate_line_item_amount(
                    item.quantity, item.unit_price,
                    item.discount_type.as_deref(), item.discount_value, item.tax_rate,
                );
                if let Err(e) = db.execute(
                    "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount, tax_rate, discount_type, discount_value)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![inv_id, item.item_id, item.description.as_deref().unwrap_or(""),
                        item.quantity, item.unit_price, amount, item.tax_rate.unwrap_or(0.0),
                        item.discount_type, item.discount_value],
                ) {
                    tracing::error!("Failed to insert invoice item: {}", e);
                    txn_ok = false;
                    break;
                }

                // Create stock movement (OUT) for this item
                let warehouse_id = form.warehouse_id.unwrap_or(1);
                let unit_cost: f64 = db.query_row(
                    "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                    [item.item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);

                // Get movement number
                let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);

                if let Err(e) = db.execute(
                    "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                     VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'INVOICE', ?6, ?7)",
                    rusqlite::params![mno, item.item_id, warehouse_id, item.quantity, unit_cost, invoice_no, format!("Invoice {}", invoice_no)],
                ) {
                    tracing::error!("Failed to create stock movement: {}", e);
                    txn_ok = false;
                    break;
                }

                // Update stock_balances
                if let Err(e) = db.execute(
                    "UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![item.quantity, item.item_id, warehouse_id],
                ) {
                    tracing::error!("Failed to update stock_balances: {}", e);
                    txn_ok = false;
                    break;
                }

                // Update items.current_stock
                if let Err(e) = db.execute(
                    "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![item.quantity, item.item_id],
                ) {
                    tracing::error!("Failed to update items.current_stock: {}", e);
                    txn_ok = false;
                    break;
                }
            }

            if !txn_ok {
                let _ = db.execute_batch("ROLLBACK");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
            }

            // Insert customer ledger entry for invoice
            {
                let last_balance: f64 = db.query_row(
                    "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                    [form.customer_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);
                let new_balance = last_balance + total_amount;
                if let Err(e) = db.execute(
                    "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                     VALUES (?1, ?2, 'INVOICE', ?3, ?4, 0, ?5)",
                    rusqlite::params![form.customer_id, &form.invoice_date,
                        invoice_no, total_amount, new_balance],
                ) {
                    tracing::error!("Failed to insert customer ledger: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                }
                // Update customer current_balance
                if let Err(e) = db.execute(
                    "UPDATE customers SET current_balance = current_balance + ?1 WHERE id = ?2",
                    rusqlite::params![total_amount, form.customer_id],
                ) {
                    tracing::error!("Failed to update customer balance: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                }
            }
            // Auto-journal: debit AR (account_id=2), credit Revenue (account_id=11)
            {
                if let Err(e) = db.execute(
                    "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('invoice', ?1, ?2)",
                    rusqlite::params![inv_id, &form.invoice_date],
                ) {
                    tracing::error!("Failed to insert journal entry: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                }
                let je_id = db.last_insert_rowid();
                if let Err(e) = db.execute(
                    "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                     VALUES (?1, 2, ?2, 0, ?3, ?4),
                            (?1, 11, 0, ?2, ?5, ?4)",
                    rusqlite::params![je_id, total_amount, format!("Invoice {}", invoice_no), &form.invoice_date, format!("Revenue - Invoice {}", invoice_no)],
                ) {
                    tracing::error!("Failed to insert journal lines: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                }
            }
            // Record payment if provided
            if form.record_payment.unwrap_or(false) {
                if let Some(pay_amt) = form.payment_amount {
                    if pay_amt > 0.0 {
                        let pseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM payments", [], |row| row.get(0)).unwrap_or(1);
                        let pno = format!("PAY-{}-{:04}", chrono::Utc::now().format("%Y"), pseq);
                        if let Err(e) = db.execute(
                            "INSERT INTO payments (payment_no, customer_id, invoice_id, payment_date, amount, payment_method)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            rusqlite::params![pno, form.customer_id, inv_id, form.invoice_date, pay_amt,
                                form.payment_method.as_deref().unwrap_or("Cash")],
                        ) {
                            tracing::error!("Failed to insert payment: {}", e);
                            let _ = db.execute_batch("ROLLBACK");
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                        }
                        let paid = pay_amt.min(total_amount);
                        let bal = total_amount - paid;
                        let status = if bal <= 0.0 { "Paid" } else { "Partially Paid" };
                        if let Err(e) = db.execute("UPDATE invoices SET paid_amount=?1, balance_amount=?2, status=?3 WHERE id=?4",
                            rusqlite::params![paid, bal, status, inv_id]) {
                            tracing::error!("Failed to update invoice payment status: {}", e);
                            let _ = db.execute_batch("ROLLBACK");
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                        }
                        // Insert customer ledger entry for payment
                        {
                            let last_balance: f64 = db.query_row(
                                "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                                [form.customer_id],
                                |row| row.get(0),
                            ).unwrap_or(0.0);
                            let new_balance = last_balance - pay_amt;
                            if let Err(e) = db.execute(
                                "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                                 VALUES (?1, ?2, 'PAYMENT', ?3, 0, ?4, ?5)",
                                rusqlite::params![form.customer_id, &form.invoice_date,
                                    pno, pay_amt, new_balance],
                            ) {
                                tracing::error!("Failed to insert payment ledger entry: {}", e);
                                let _ = db.execute_batch("ROLLBACK");
                                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                            }
                            if let Err(e) = db.execute(
                                "UPDATE customers SET current_balance = current_balance - ?1 WHERE id = ?2",
                                rusqlite::params![pay_amt, form.customer_id],
                            ) {
                                tracing::error!("Failed to update customer balance for payment: {}", e);
                                let _ = db.execute_batch("ROLLBACK");
                                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice (transaction rolled back)." })));
                            }
                        }
                    }
                }
            }

            // Commit the transaction
            if let Err(e) = db.execute_batch("COMMIT") {
                tracing::error!("Failed to commit transaction: {}", e);
                let _ = db.execute_batch("ROLLBACK");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit invoice (transaction rolled back)." })));
            }

            let inv = db.query_row(
                "SELECT i.id, i.invoice_no, i.customer_id, c.customer_name, c.customer_code, i.so_id, i.quotation_id,
                        i.source_type, i.invoice_date, i.due_date, i.status, i.total_amount,
                        i.paid_amount, i.balance_amount, i.returned_amount, i.discount_scope,
                        i.discount_type, i.discount_value, i.tax_rate, i.notes, i.warehouse_id,
                        i.created_by, i.created_at, i.updated_at,
                        (SELECT COUNT(*) FROM invoice_items ii WHERE ii.invoice_id = i.id) AS item_count
                 FROM invoices i LEFT JOIN customers c ON i.customer_id = c.id WHERE i.id = ?1",
                [inv_id],
                |row| Ok(Invoice {
                    id: row.get(0)?, invoice_no: row.get(1)?, customer_id: row.get(2)?,
                    customer_name: row.get(3)?, customer_code: row.get(4)?, so_id: row.get(5)?, quotation_id: row.get(6)?,
                    source_type: row.get(7)?, invoice_date: row.get(8)?, due_date: row.get(9)?,
                    status: row.get(10)?, total_amount: row.get(11)?, paid_amount: row.get(12)?,
                    balance_amount: row.get(13)?, returned_amount: row.get(14)?,
                    discount_scope: row.get(15)?, discount_type: row.get(16)?, discount_value: row.get(17)?,
                    tax_rate: row.get(18)?, notes: row.get(19)?, warehouse_id: row.get(20)?,
                    created_by: row.get(21)?, created_at: row.get(22)?, updated_at: row.get(23)?,
                    item_count: row.get(24)?,
                }),
            ).unwrap();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": inv })))
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to create invoice: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create invoice." })))
        }
    }
}

async fn update_invoice(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<InvoiceForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // Capture old values before update for ledger adjustment
    let old_total: f64 = db.query_row(
        "SELECT COALESCE(total_amount, 0) FROM invoices WHERE id = ?1", [id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    let old_customer_id: i64 = db.query_row(
        "SELECT COALESCE(customer_id, 0) FROM invoices WHERE id = ?1", [id],
        |row| row.get(0),
    ).unwrap_or(0);

    let mut total_amount = 0.0;
    for item in &form.items {
        total_amount += calculate_line_item_amount(
            item.quantity, item.unit_price,
            item.discount_type.as_deref(), item.discount_value, item.tax_rate,
        );
    }
    let discount = match form.discount_type.as_deref() {
        Some("percentage") => total_amount * (form.discount_value.unwrap_or(0.0) / 100.0),
        Some("fixed") => form.discount_value.unwrap_or(0.0),
        _ => 0.0,
    };
    total_amount -= discount;

    let result = db.execute(
        "UPDATE invoices SET customer_id=?1, source_type=?2, invoice_date=?3, due_date=?4,
         total_amount=?5, discount_scope=?6, discount_type=?7, discount_value=?8,
         tax_rate=?9, notes=?10, updated_at=datetime('now') WHERE id=?11",
        rusqlite::params![form.customer_id, form.source_type.as_deref().unwrap_or("DIRECT"),
            form.invoice_date, form.due_date, total_amount, form.discount_scope,
            form.discount_type, form.discount_value, form.tax_rate,
            form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            // Delete removed payments and recalculate paid_amount
            if let Some(ref deleted_ids) = form.deleted_payment_ids {
                for pid in deleted_ids {
                    let _ = db.execute("DELETE FROM payment_allocations WHERE payment_id = ?1", [*pid]);
                    let _ = db.execute("DELETE FROM payments WHERE id = ?1", [*pid]);
                }
            }
            // Recalculate paid_amount from remaining payments
            let paid_amount: f64 = db.query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE invoice_id = ?1", [id],
                |row| row.get(0),
            ).unwrap_or(0.0);
            let balance_amount = total_amount - paid_amount;
            let status = if paid_amount <= 0.0 { "Unpaid" }
                else if paid_amount >= total_amount { "Paid" }
                else { "Partially Paid" };
            if let Err(e) = db.execute(
                "UPDATE invoices SET paid_amount=?1, balance_amount=?2, status=?3 WHERE id=?4",
                rusqlite::params![paid_amount, balance_amount, status, id],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update invoice payment status: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
            }

            // Before deleting old items, capture their quantities for stock delta calculation
            let mut old_item_map: HashMap<i64, f64> = HashMap::new();
            {
                let mut old_stmt = db.prepare(
                    "SELECT item_id, quantity FROM invoice_items WHERE invoice_id = ?1"
                ).unwrap();
                let old_rows = old_stmt.query_map([id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
                }).unwrap();
                for r in old_rows.flatten() {
                    *old_item_map.entry(r.0).or_insert(0.0) += r.1;
                }
            }

            if let Err(e) = db.execute("DELETE FROM invoice_items WHERE invoice_id = ?1", [id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to delete old invoice items: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
            }
            for item in &form.items {
                let amount = calculate_line_item_amount(
                    item.quantity, item.unit_price,
                    item.discount_type.as_deref(), item.discount_value, item.tax_rate,
                );
                if let Err(e) = db.execute(
                    "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount, tax_rate, discount_type, discount_value)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![id, item.item_id, item.description.as_deref().unwrap_or(""),
                        item.quantity, item.unit_price, amount, item.tax_rate.unwrap_or(0.0),
                        item.discount_type, item.discount_value],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert new invoice items: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                }
            }

            // --- Stock delta adjustment (Bug F5) ---
            let warehouse_id = form.warehouse_id.unwrap_or(1);
            let invoice_no: String = db.query_row(
                "SELECT invoice_no FROM invoices WHERE id = ?1", [id],
                |row| row.get(0),
            ).unwrap_or_default();

            // Build map of new quantities
            let mut new_item_map: HashMap<i64, f64> = HashMap::new();
            for item in &form.items {
                *new_item_map.entry(item.item_id).or_insert(0.0) += item.quantity;
            }

            // Adjust stock for items that were in old but not in new (removed items)
            for (item_id, old_qty) in &old_item_map {
                if !new_item_map.contains_key(item_id) && *old_qty > 0.0 {
                    let unit_cost: f64 = db.query_row(
                        "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                        [*item_id],
                        |row| row.get(0),
                    ).unwrap_or(0.0);
                    let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                    let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
                    if let Err(e) = db.execute(
                        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                         VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'INVOICE_EDIT', ?6, ?7)",
                        rusqlite::params![mno, item_id, warehouse_id, old_qty, unit_cost, invoice_no, format!("Stock returned - item removed from invoice {}", invoice_no)],
                    ) {
                        tracing::error!("Failed to create stock movement for removed item: {}", e);
                        let _ = db.execute_batch("ROLLBACK");
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                    }
                    if let Err(e) = db.execute(
                        "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                        rusqlite::params![old_qty, item_id, warehouse_id],
                    ) {
                        tracing::error!("Failed to update stock_balances for removed item: {}", e);
                        let _ = db.execute_batch("ROLLBACK");
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                    }
                    if let Err(e) = db.execute(
                        "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                        rusqlite::params![old_qty, item_id],
                    ) {
                        tracing::error!("Failed to update current_stock for removed item: {}", e);
                        let _ = db.execute_batch("ROLLBACK");
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                    }
                }
            }

            // Adjust stock for items present in both old and new (delta)
            for (item_id, new_qty) in &new_item_map {
                let old_qty = old_item_map.get(item_id).copied().unwrap_or(0.0);
                let delta = old_qty - new_qty;
                if delta.abs() < f64::EPSILON {
                    continue;
                }
                let unit_cost: f64 = db.query_row(
                    "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                    [*item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);

                let (movement_type, adj_qty) = if delta > 0.0 {
                    // Quantity decreased -> stock returns IN
                    ("IN", delta)
                } else {
                    // Quantity increased -> stock goes OUT
                    ("OUT", delta.abs())
                };

                let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
                if let Err(e) = db.execute(
                    "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'INVOICE_EDIT', ?7, ?8)",
                    rusqlite::params![mno, item_id, warehouse_id, movement_type, adj_qty, unit_cost, invoice_no, format!("Stock adjustment for invoice {}", invoice_no)],
                ) {
                    tracing::error!("Failed to create stock movement for delta: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                }

                let stock_adj = if delta > 0.0 { adj_qty } else { -adj_qty };
                if let Err(e) = db.execute(
                    "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![stock_adj, item_id, warehouse_id],
                ) {
                    tracing::error!("Failed to update stock_balances for delta: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![stock_adj, item_id],
                ) {
                    tracing::error!("Failed to update current_stock for delta: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice (transaction rolled back)." })));
                }
            }

            // --- Customer ledger adjustment (Task 3.6) ---
            // If total_amount changed or customer changed, update ledger and balance
            let total_delta = total_amount - old_total;
            let new_customer_id = form.customer_id;

            if old_customer_id != new_customer_id {
                // Customer changed: reverse ledger for old customer, add for new customer
                if old_total > 0.0 && old_customer_id > 0 {
                    let last_bal_old: f64 = db.query_row(
                        "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                        [old_customer_id], |row| row.get(0),
                    ).unwrap_or(0.0);
                    let inv_no: String = db.query_row(
                        "SELECT invoice_no FROM invoices WHERE id = ?1", [id], |row| row.get(0),
                    ).unwrap_or_default();
                    let _ = db.execute(
                        "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                         VALUES (?1, datetime('now'), 'INVOICE_EDIT', ?2, 0, ?3, ?4)",
                        rusqlite::params![old_customer_id, format!("EDIT-{}", inv_no), old_total, last_bal_old - old_total],
                    );
                    let _ = db.execute(
                        "UPDATE customers SET current_balance = current_balance - ?1 WHERE id = ?2",
                        rusqlite::params![old_total, old_customer_id],
                    );
                }
                if total_amount > 0.0 && new_customer_id > 0 {
                    let last_bal_new: f64 = db.query_row(
                        "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                        [new_customer_id], |row| row.get(0),
                    ).unwrap_or(0.0);
                    let inv_no: String = db.query_row(
                        "SELECT invoice_no FROM invoices WHERE id = ?1", [id], |row| row.get(0),
                    ).unwrap_or_default();
                    let _ = db.execute(
                        "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                         VALUES (?1, datetime('now'), 'INVOICE_EDIT', ?2, ?3, 0, ?4)",
                        rusqlite::params![new_customer_id, format!("EDIT-{}", inv_no), total_amount, last_bal_new + total_amount],
                    );
                    let _ = db.execute(
                        "UPDATE customers SET current_balance = current_balance + ?1 WHERE id = ?2",
                        rusqlite::params![total_amount, new_customer_id],
                    );
                }
            } else if total_delta.abs() > 0.01 && new_customer_id > 0 {
                // Same customer, total changed: adjust ledger by delta
                let last_balance: f64 = db.query_row(
                    "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                    [new_customer_id], |row| row.get(0),
                ).unwrap_or(0.0);
                let inv_no: String = db.query_row(
                    "SELECT invoice_no FROM invoices WHERE id = ?1", [id], |row| row.get(0),
                ).unwrap_or_default();
                if total_delta > 0.0 {
                    // Total increased: add debit entry
                    let _ = db.execute(
                        "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                         VALUES (?1, datetime('now'), 'INVOICE_EDIT', ?2, ?3, 0, ?4)",
                        rusqlite::params![new_customer_id, format!("EDIT-{}", inv_no), total_delta, last_balance + total_delta],
                    );
                    let _ = db.execute(
                        "UPDATE customers SET current_balance = current_balance + ?1 WHERE id = ?2",
                        rusqlite::params![total_delta, new_customer_id],
                    );
                } else {
                    // Total decreased: add credit entry
                    let _ = db.execute(
                        "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                         VALUES (?1, datetime('now'), 'INVOICE_EDIT', ?2, 0, ?3, ?4)",
                        rusqlite::params![new_customer_id, format!("EDIT-{}", inv_no), total_delta.abs(), last_balance + total_delta],
                    );
                    let _ = db.execute(
                        "UPDATE customers SET current_balance = current_balance + ?1 WHERE id = ?2",
                        rusqlite::params![total_delta, new_customer_id],
                    );
                }
            }

            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit transaction: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit invoice update (transaction rolled back)." })));
            }

            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Invoice updated." } })))
        }
        Ok(_) => {
            let _ = db.execute_batch("ROLLBACK");
            (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Invoice not found." })))
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update invoice: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update invoice." })))
        }
    }
}

async fn cancel_invoice(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // 1. Check invoice exists and is not already cancelled
    let invoice_exists: bool = db.query_row(
        "SELECT COUNT(*) FROM invoices WHERE id = ?1 AND status != 'Cancelled'", [id],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;
    if !invoice_exists {
        let _ = db.execute_batch("ROLLBACK");
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Invoice not found or already cancelled." })));
    }

    // 2. Get invoice details
    let total_amount: f64 = db.query_row("SELECT total_amount FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(0.0);
    let returned_amount: f64 = db.query_row("SELECT returned_amount FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(0.0);
    let customer_id: i64 = db.query_row("SELECT customer_id FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(0);
    let invoice_no: String = db.query_row("SELECT invoice_no FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or_default();
    let warehouse_id: i64 = db.query_row("SELECT COALESCE(warehouse_id, 1) FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(1);

    // 3. For each invoice item: restore unreturned stock via IN movement
    {
        let mut stmt = db.prepare(
            "SELECT item_id, quantity, returned_qty FROM invoice_items WHERE invoice_id = ?1"
        ).unwrap();
        let items = stmt.query_map([id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?, row.get::<_, f64>(2)?))
        }).unwrap();
        for item_result in items {
            if let Ok((item_id, quantity, returned_qty)) = item_result {
                let remaining = quantity - returned_qty;
                if remaining <= 0.0 {
                    continue;
                }
                let unit_cost: f64 = db.query_row(
                    "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                    [item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);

                let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
                if let Err(e) = db.execute(
                    "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                     VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'INVOICE_CANCEL', ?6, ?7)",
                    rusqlite::params![mno, item_id, warehouse_id, remaining, unit_cost, invoice_no, format!("Invoice cancelled {}", invoice_no)],
                ) {
                    tracing::error!("Failed to create stock movement on cancel: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![remaining, item_id, warehouse_id],
                ) {
                    tracing::error!("Failed to update stock_balances on cancel: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![remaining, item_id],
                ) {
                    tracing::error!("Failed to update current_stock on cancel: {}", e);
                    let _ = db.execute_batch("ROLLBACK");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
                }
            }
        }
    }

    // 4. Reverse customer ledger: credit entry for net_amount
    let net_amount = total_amount - returned_amount;
    if net_amount > 0.0 {
        let last_balance: f64 = db.query_row(
            "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
            [customer_id],
            |row| row.get(0),
        ).unwrap_or(0.0);
        let new_balance = last_balance - net_amount;
        if let Err(e) = db.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (?1, datetime('now'), 'INVOICE_CANCEL', ?2, 0, ?3, ?4)",
            rusqlite::params![customer_id, format!("CANCEL-{}", invoice_no), net_amount, new_balance],
        ) {
            tracing::error!("Failed to insert cancel ledger entry: {}", e);
            let _ = db.execute_batch("ROLLBACK");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
        }
        if let Err(e) = db.execute(
            "UPDATE customers SET current_balance = current_balance - ?1 WHERE id = ?2",
            rusqlite::params![net_amount, customer_id],
        ) {
            tracing::error!("Failed to update customer balance on cancel: {}", e);
            let _ = db.execute_batch("ROLLBACK");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
        }
    }

    // 5. Reverse GL: debit Revenue (11), credit AR (2)
    {
        if let Err(e) = db.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('invoice_cancel', ?1, datetime('now'))",
            [id],
        ) {
            tracing::error!("Failed to insert cancel journal entry: {}", e);
            let _ = db.execute_batch("ROLLBACK");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
        }
        let je_id = db.last_insert_rowid();
        if let Err(e) = db.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 11, ?2, 0, ?3, datetime('now')),
                    (?1, 2, 0, ?2, ?4, datetime('now'))",
            rusqlite::params![je_id, net_amount, format!("Cancelled Invoice {}", invoice_no), format!("AR Reversal - Invoice {}", invoice_no)],
        ) {
            tracing::error!("Failed to insert cancel journal lines: {}", e);
            let _ = db.execute_batch("ROLLBACK");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
        }
    }

    // 6. Set invoice status = 'Cancelled', balance_amount = 0
    if let Err(e) = db.execute(
        "UPDATE invoices SET status = 'Cancelled', balance_amount = 0, updated_at = datetime('now') WHERE id = ?1",
        [id],
    ) {
        tracing::error!("Failed to update invoice status on cancel: {}", e);
        let _ = db.execute_batch("ROLLBACK");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
    }

    // 7. Commit
    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit cancel transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to cancel invoice (transaction rolled back)." })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Invoice cancelled and reversed." } })))
}

async fn return_invoice(State(_state): State<AppState>, Path(id): Path<i64>, Json(req): Json<InvoiceReturnRequest>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // Get invoice details
    let invoice_no: String = db.query_row("SELECT invoice_no FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or_default();
    let warehouse_id: i64 = db.query_row("SELECT COALESCE(warehouse_id, 1) FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(1);

    for ret_item in &req.items {
        // Update returned_qty
        if let Err(e) = db.execute(
            "UPDATE invoice_items SET returned_qty = returned_qty + ?1 WHERE invoice_id = ?2 AND item_id = ?3",
            rusqlite::params![ret_item.quantity, id, ret_item.item_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update returned_qty: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }

        // Get unit cost
        let unit_cost: f64 = db.query_row(
            "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
            [ret_item.item_id],
            |row| row.get(0),
        ).unwrap_or(0.0);

        // Create stock movement (IN - goods returning from customer)
        let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
        let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
        if let Err(e) = db.execute(
            "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
             VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'INVOICE_RETURN', ?6, ?7)",
            rusqlite::params![mno, ret_item.item_id, warehouse_id, ret_item.quantity, unit_cost, invoice_no, format!("Invoice Return {}", invoice_no)],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to create return stock movement: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }

        // Update stock_balances
        if let Err(e) = db.execute(
            "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
            rusqlite::params![ret_item.quantity, ret_item.item_id, warehouse_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update stock_balances on return: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }

        // Update items.current_stock
        if let Err(e) = db.execute(
            "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![ret_item.quantity, ret_item.item_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update items.current_stock on return: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }
    }

    let total_returned: f64 = db.query_row(
        "SELECT COALESCE(SUM(returned_qty * unit_price), 0) FROM invoice_items WHERE invoice_id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    if let Err(e) = db.execute("UPDATE invoices SET returned_amount = ?1, updated_at = datetime('now') WHERE id = ?2", rusqlite::params![total_returned, id]) {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to update invoice returned_amount: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
    }

    // Insert customer ledger entry for return
    if total_returned > 0.0 {
        let customer_id: i64 = db.query_row("SELECT customer_id FROM invoices WHERE id = ?1", [id], |row| row.get(0)).unwrap_or(0);
        let last_balance: f64 = db.query_row(
            "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
            [customer_id],
            |row| row.get(0),
        ).unwrap_or(0.0);
        let new_balance = last_balance - total_returned;
        if let Err(e) = db.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (?1, datetime('now'), 'RETURN', ?2, 0, ?3, ?4)",
            rusqlite::params![customer_id, format!("RET-{}", invoice_no), total_returned, new_balance],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to insert return ledger entry: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }
        if let Err(e) = db.execute(
            "UPDATE customers SET current_balance = current_balance - ?1 WHERE id = ?2",
            rusqlite::params![total_returned, customer_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update customer balance on return: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record return (transaction rolled back)." })));
        }
    }

    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit return transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit return (transaction rolled back)." })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Return recorded.", "returned_amount": total_returned } })))
}

async fn invoice_payments(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.payment_no, p.customer_id, c.customer_name, p.invoice_id,
                p.payment_date, p.amount, p.payment_method, p.reference, p.notes,
                p.created_by, p.created_at
         FROM payments p LEFT JOIN customers c ON p.customer_id = c.id
         WHERE p.invoice_id = ?1 ORDER BY p.payment_date DESC"
    ).unwrap();
    let payments: Vec<Payment> = stmt.query_map([id], |row| {
        Ok(Payment {
            id: row.get(0)?, payment_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, invoice_id: row.get(4)?, payment_date: row.get(5)?,
            amount: row.get(6)?, payment_method: row.get(7)?, reference: row.get(8)?,
            notes: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": payments })))
}

async fn list_returns(State(_state): State<AppState>) -> impl IntoResponse {
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

/// One-time recalculation endpoint: fix existing invoices where total_amount
/// doesn't match SUM(line_item.amount). Returns list of corrected invoices.
async fn recalculate_invoice_totals(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // Find invoices where total_amount != SUM(line items)
    let mismatched: Vec<(i64, String, f64, f64)> = {
        let mut stmt = db.prepare(
            "SELECT i.id, i.invoice_no, i.total_amount,
                    COALESCE(SUM(ii.amount), 0) as line_sum
             FROM invoices i
             LEFT JOIN invoice_items ii ON ii.invoice_id = i.id
             WHERE i.status != 'Cancelled'
             GROUP BY i.id
             HAVING ABS(i.total_amount - line_sum) > 0.01"
        ).unwrap();
        stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).unwrap().filter_map(|r| r.ok()).collect()
    };

    let mut fixed_count = 0;
    for (inv_id, inv_no, old_total, correct_total) in &mismatched {
        let _ = db.execute(
            "UPDATE invoices SET total_amount = ?1, balance_amount = ?1 - paid_amount, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![correct_total, inv_id],
        );
        fixed_count += 1;
        tracing::info!("Fixed invoice {}: {} -> {}", inv_no, old_total, correct_total);
    }

    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit recalculation: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit recalculation." })));
    }

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "mismatched_count": mismatched.len(),
            "fixed_count": fixed_count,
            "details": mismatched.iter().map(|(id, no, old, new)| json!({
                "invoice_id": id, "invoice_no": no, "old_total": old, "new_total": new
            })).collect::<Vec<_>>()
        }
    })))
}
