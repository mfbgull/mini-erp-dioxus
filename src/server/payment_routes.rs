use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/payments", get(list_payments).post(create_payment))
        .route("/api/payments/{id}", get(get_payment).put(update_payment).delete(delete_payment))
}

async fn list_payments(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.payment_no, p.customer_id, c.customer_name, p.invoice_id,
                p.payment_date, p.amount, p.payment_method, p.reference, p.notes,
                p.created_by, p.created_at
         FROM payments p LEFT JOIN customers c ON p.customer_id = c.id
         ORDER BY p.created_at DESC"
    ).unwrap();
    let items: Vec<Payment> = stmt.query_map([], |row| {
        Ok(Payment {
            id: row.get(0)?, payment_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, invoice_id: row.get(4)?, payment_date: row.get(5)?,
            amount: row.get(6)?, payment_method: row.get(7)?, reference: row.get(8)?,
            notes: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_payment(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT p.id, p.payment_no, p.customer_id, c.customer_name, p.invoice_id,
                p.payment_date, p.amount, p.payment_method, p.reference, p.notes,
                p.created_by, p.created_at
         FROM payments p LEFT JOIN customers c ON p.customer_id = c.id WHERE p.id = ?1",
        [id],
        |row| Ok(Payment {
            id: row.get(0)?, payment_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, invoice_id: row.get(4)?, payment_date: row.get(5)?,
            amount: row.get(6)?, payment_method: row.get(7)?, reference: row.get(8)?,
            notes: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        }),
    );
    match result {
        Ok(p) => (StatusCode::OK, Json(json!({ "success": true, "data": p }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Payment not found." }))),
    }
}

async fn create_payment(State(_state): State<AppState>, Json(form): Json<PaymentForm>) -> impl IntoResponse {
    if form.amount <= 0.0 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Payment amount must be positive." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM payments", [], |row| row.get(0)).unwrap_or(1);
    let pno = format!("PAY-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    let result = db.execute(
        "INSERT INTO payments (payment_no, customer_id, invoice_id, payment_date, amount, payment_method, reference, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![pno, form.customer_id, form.invoice_id, form.payment_date, form.amount,
            form.payment_method, form.reference.as_deref().unwrap_or(""), form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let pay_id = db.last_insert_rowid();
            // Update invoice paid_amount if allocated
            if let Some(allocs) = &form.allocations {
                for alloc in allocs {
                    if let Err(e) = db.execute(
                        "INSERT INTO payment_allocations (payment_id, invoice_id, amount) VALUES (?1, ?2, ?3)",
                        rusqlite::params![pay_id, alloc.invoice_id, alloc.amount],
                    ) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to insert payment allocation: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                    }
                    if let Err(e) = db.execute(
                        "UPDATE invoices SET paid_amount = paid_amount + ?1, balance_amount = balance_amount - ?1,
                         status = CASE WHEN balance_amount - ?1 <= 0 THEN 'Paid' ELSE status END
                         WHERE id = ?2",
                        rusqlite::params![alloc.amount, alloc.invoice_id],
                    ) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to update invoice for allocation: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                    }
                }
            } else if let Some(inv_id) = form.invoice_id {
                if let Err(e) = db.execute("INSERT INTO payment_allocations (payment_id, invoice_id, amount) VALUES (?1, ?2, ?3)",
                    rusqlite::params![pay_id, inv_id, form.amount]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert payment allocation: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE invoices SET paid_amount = paid_amount + ?1, balance_amount = balance_amount - ?1,
                     status = CASE WHEN balance_amount - ?1 <= 0 THEN 'Paid'
                                  WHEN paid_amount + ?1 > 0 THEN 'Partially Paid'
                                  ELSE status END
                     WHERE id = ?2",
                    rusqlite::params![form.amount, inv_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update invoice for payment: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                }
            }
            // Update customer balance
            if let Err(e) = db.execute(
                "UPDATE customers SET current_balance = current_balance - ?1, credit_balance = credit_balance + ?1 WHERE id = ?2",
                rusqlite::params![form.amount, form.amount, form.customer_id],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update customer balance: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
            }
            // Insert customer ledger entry for payment
            {
                let last_balance: f64 = db.query_row(
                    "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
                    [form.customer_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);
                let new_balance = last_balance - form.amount;
                if let Err(e) = db.execute(
                    "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
                     VALUES (?1, ?2, 'PAYMENT', ?3, 0, ?4, ?5)",
                    rusqlite::params![form.customer_id, form.payment_date, pno, form.amount, new_balance],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert payment ledger entry: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                }
            }
            // Auto-journal: debit Cash (account_id=1), credit AR (account_id=2)
            {
                if let Err(e) = db.execute(
                    "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('payment', ?1, ?2)",
                    rusqlite::params![pay_id, form.payment_date],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert journal entry: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                }
                let je_id = db.last_insert_rowid();
                if let Err(e) = db.execute(
                    "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                     VALUES (?1, 1, ?2, 0, ?3, ?4),
                            (?1, 2, 0, ?2, ?5, ?4)",
                    rusqlite::params![je_id, form.amount, format!("Payment {}", pno), form.payment_date, format!("AR - Payment {}", pno)],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert journal lines: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment (transaction rolled back)." })));
                }
            }

            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit payment transaction: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit payment (transaction rolled back)." })));
            }

            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": pay_id, "payment_no": pno } })))
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to create payment: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create payment." })))
        }
    }
}

async fn update_payment(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<PaymentForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE payments SET payment_date=?1, amount=?2, payment_method=?3, reference=?4, notes=?5 WHERE id=?6",
        rusqlite::params![form.payment_date, form.amount, form.payment_method,
            form.reference.as_deref().unwrap_or(""), form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Payment updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Payment not found." }))),
        Err(e) => { tracing::error!("Failed to update payment: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update payment." }))) }
    }
}

async fn delete_payment(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // 1. Retrieve payment data before deletion
    let (pay_amount, customer_id, payment_no, payment_date): (f64, i64, String, String) = match db.query_row(
        "SELECT amount, customer_id, payment_no, payment_date FROM payments WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ) {
        Ok(v) => v,
        Err(_) => {
            let _ = db.execute_batch("ROLLBACK");
            return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Payment not found." })));
        }
    };

    // 2. Get payment allocations
    let allocations: Vec<(i64, f64)> = {
        let mut stmt = db.prepare(
            "SELECT invoice_id, amount FROM payment_allocations WHERE payment_id = ?1"
        ).unwrap();
        stmt.query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap().filter_map(|r| r.ok()).collect()
    };

    // 3. Reverse invoice balances for each allocation
    for (inv_id, alloc_amount) in &allocations {
        if let Err(e) = db.execute(
            "UPDATE invoices SET paid_amount = paid_amount - ?1, balance_amount = balance_amount + ?1 WHERE id = ?2",
            rusqlite::params![alloc_amount, inv_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to reverse invoice paid_amount: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
        }
        // Recalculate invoice status
        let (new_paid, total): (f64, f64) = db.query_row(
            "SELECT COALESCE(paid_amount, 0), COALESCE(total_amount, 0) FROM invoices WHERE id = ?1",
            [*inv_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0.0, 0.0));
        let new_status = if new_paid <= 0.0 { "Unpaid" }
            else if new_paid >= total { "Paid" }
            else { "Partially Paid" };
        if let Err(e) = db.execute(
            "UPDATE invoices SET status = ?1 WHERE id = ?2",
            rusqlite::params![new_status, inv_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update invoice status: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
        }
    }

    // 4. Reverse customer ledger
    {
        let last_balance: f64 = db.query_row(
            "SELECT COALESCE(balance, 0) FROM customer_ledger WHERE customer_id = ?1 ORDER BY id DESC LIMIT 1",
            [customer_id],
            |row| row.get(0),
        ).unwrap_or(0.0);
        let new_balance = last_balance + pay_amount;
        if let Err(e) = db.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (?1, ?2, 'PAYMENT_REVERSAL', ?3, ?4, 0, ?5)",
            rusqlite::params![customer_id, &payment_date, format!("DEL-{}", payment_no), pay_amount, new_balance],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to insert reversal ledger entry: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
        }
    }

    // 5. Restore customer balance
    if let Err(e) = db.execute(
        "UPDATE customers SET current_balance = current_balance + ?1, credit_balance = credit_balance - ?1 WHERE id = ?2",
        rusqlite::params![pay_amount, pay_amount, customer_id],
    ) {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to restore customer balance: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
    }

    // 6. Reverse GL: debit AR (2), credit Cash (1)
    {
        if let Err(e) = db.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('payment_deletion', ?1, ?2)",
            rusqlite::params![id, &payment_date],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to insert reversal journal entry: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
        }
        let je_id = db.last_insert_rowid();
        if let Err(e) = db.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 2, ?2, 0, ?3, ?4),
                    (?1, 1, 0, ?2, ?5, ?4)",
            rusqlite::params![je_id, pay_amount, format!("Payment reversal {}", payment_no), &payment_date, format!("Cash reversal {}", payment_no)],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to insert reversal journal lines: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
        }
    }

    // 7. Delete payment allocations, then payment
    if let Err(e) = db.execute("DELETE FROM payment_allocations WHERE payment_id = ?1", [id]) {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to delete payment allocations: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
    }
    if let Err(e) = db.execute("DELETE FROM payments WHERE id = ?1", [id]) {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to delete payment: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete payment (transaction rolled back)." })));
    }

    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit payment deletion: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit payment deletion (transaction rolled back)." })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Payment deleted and all effects reversed.", "reversed_amount": pay_amount } })))
}
