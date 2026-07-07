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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/customers", get(list_customers).post(create_customer))
        .route("/api/customers/{id}", get(get_customer).put(update_customer).delete(delete_customer))
        .route("/api/customers/{id}/ledger", get(customer_ledger))
        .route("/api/customers/{id}/statement", get(customer_statement))
        .route("/api/customers/{id}/balance", get(customer_balance))
        .route("/api/customers/{id}/payments", get(customer_payments))
        .route("/api/customers/recalculate-balances", post(recalculate_balances))
}

async fn list_customers(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, customer_code, customer_name, email, phone, billing_address, shipping_address,
                payment_terms, credit_limit, credit_balance, current_balance, opening_balance,
                is_active, customer_type, notes, total_invoiced, total_paid, last_invoice_date,
                created_at, updated_at
         FROM customers WHERE is_active = 1 ORDER BY customer_code"
    ).unwrap();
    let items: Vec<Customer> = stmt.query_map([], |row| {
        Ok(Customer {
            id: row.get(0)?,
            customer_code: row.get(1)?,
            customer_name: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            billing_address: row.get(5)?,
            shipping_address: row.get(6)?,
            payment_terms: row.get(7)?,
            credit_limit: row.get(8)?,
            credit_balance: row.get(9)?,
            current_balance: row.get(10)?,
            opening_balance: row.get(11)?,
            is_active: row.get::<_, i64>(12)? != 0,
            customer_type: row.get(13)?,
            notes: row.get(14)?,
            total_invoiced: row.get(15)?,
            total_paid: row.get(16)?,
            last_invoice_date: row.get(17)?,
            created_at: row.get(18)?,
            updated_at: row.get(19)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_customer(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, customer_code, customer_name, email, phone, billing_address, shipping_address,
                payment_terms, credit_limit, credit_balance, current_balance, opening_balance,
                is_active, customer_type, notes, total_invoiced, total_paid, last_invoice_date,
                created_at, updated_at
         FROM customers WHERE id = ?1",
        [id],
        |row| Ok(Customer {
            id: row.get(0)?,
            customer_code: row.get(1)?,
            customer_name: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            billing_address: row.get(5)?,
            shipping_address: row.get(6)?,
            payment_terms: row.get(7)?,
            credit_limit: row.get(8)?,
            credit_balance: row.get(9)?,
            current_balance: row.get(10)?,
            opening_balance: row.get(11)?,
            is_active: row.get::<_, i64>(12)? != 0,
            customer_type: row.get(13)?,
            notes: row.get(14)?,
            total_invoiced: row.get(15)?,
            total_paid: row.get(16)?,
            last_invoice_date: row.get(17)?,
            created_at: row.get(18)?,
            updated_at: row.get(19)?,
        }),
    );
    match result {
        Ok(c) => (StatusCode::OK, Json(json!({ "success": true, "data": c }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Customer not found." }))),
    }
}

async fn create_customer(State(_state): State<AppState>, Json(form): Json<CustomerForm>) -> impl IntoResponse {
    if form.customer_code.trim().is_empty() || form.customer_name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Customer code and name are required." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM customers WHERE customer_code = ?1", [&form.customer_code], |row| row.get(0)).unwrap_or(false);
    if exists {
        return (StatusCode::CONFLICT, Json(json!({ "success": false, "error": "Customer code already exists." })));
    }
    let ob = form.opening_balance.unwrap_or(0.0);
    let result = db.execute(
        "INSERT INTO customers (customer_code, customer_name, email, phone, billing_address, shipping_address, payment_terms, credit_limit, current_balance, opening_balance)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            form.customer_code, form.customer_name,
            form.email.as_deref().unwrap_or(""), form.phone.as_deref().unwrap_or(""),
            form.billing_address.as_deref().unwrap_or(""), form.shipping_address.as_deref().unwrap_or(""),
            form.payment_terms.as_deref().unwrap_or("Net 30"), form.credit_limit.unwrap_or(0.0),
            ob, ob,
        ],
    );
    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            let c = db.query_row(
                "SELECT id, customer_code, customer_name, email, phone, billing_address, shipping_address,
                        payment_terms, credit_limit, credit_balance, current_balance, opening_balance,
                        is_active, customer_type, notes, total_invoiced, total_paid, last_invoice_date,
                        created_at, updated_at FROM customers WHERE id = ?1",
                [id],
                |row| Ok(Customer {
                    id: row.get(0)?, customer_code: row.get(1)?, customer_name: row.get(2)?,
                    email: row.get(3)?, phone: row.get(4)?, billing_address: row.get(5)?,
                    shipping_address: row.get(6)?, payment_terms: row.get(7)?,
                    credit_limit: row.get(8)?, credit_balance: row.get(9)?,
                    current_balance: row.get(10)?, opening_balance: row.get(11)?,
                    is_active: row.get::<_, i64>(12)? != 0, customer_type: row.get(13)?,
                    notes: row.get(14)?, total_invoiced: row.get(15)?, total_paid: row.get(16)?,
                    last_invoice_date: row.get(17)?, created_at: row.get(18)?, updated_at: row.get(19)?,
                }),
            ).unwrap();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": c })))
        }
        Err(e) => {
            tracing::error!("Failed to create customer: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create customer." })))
        }
    }
}

async fn update_customer(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<CustomerForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE customers SET customer_code=?1, customer_name=?2, email=?3, phone=?4,
         billing_address=?5, shipping_address=?6, payment_terms=?7, credit_limit=?8, updated_at=datetime('now')
         WHERE id=?9",
        rusqlite::params![form.customer_code, form.customer_name, form.email.as_deref().unwrap_or(""),
            form.phone.as_deref().unwrap_or(""), form.billing_address.as_deref().unwrap_or(""),
            form.shipping_address.as_deref().unwrap_or(""), form.payment_terms.as_deref().unwrap_or("Net 30"),
            form.credit_limit.unwrap_or(0.0), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            let c = db.query_row(
                "SELECT id, customer_code, customer_name, email, phone, billing_address, shipping_address,
                        payment_terms, credit_limit, credit_balance, current_balance, opening_balance,
                        is_active, created_at, updated_at FROM customers WHERE id = ?1",
                [id],
                |row| Ok(Customer {
                    id: row.get(0)?, customer_code: row.get(1)?, customer_name: row.get(2)?,
                    email: row.get(3)?, phone: row.get(4)?, billing_address: row.get(5)?,
                    shipping_address: row.get(6)?, payment_terms: row.get(7)?,
                    credit_limit: row.get(8)?, credit_balance: row.get(9)?,
                    current_balance: row.get(10)?, opening_balance: row.get(11)?,
                    is_active: row.get::<_, i64>(12)? != 0, customer_type: row.get(13)?,
                    notes: row.get(14)?, total_invoiced: row.get(15)?, total_paid: row.get(16)?,
                    last_invoice_date: row.get(17)?, created_at: row.get(18)?, updated_at: row.get(19)?,
                }),
            ).unwrap();
            (StatusCode::OK, Json(json!({ "success": true, "data": c })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Customer not found." }))),
        Err(e) => { tracing::error!("Failed to update customer: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update customer." }))) }
    }
}

async fn delete_customer(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE customers SET is_active = 0, updated_at = datetime('now') WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Customer deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Customer not found." }))),
        Err(e) => { tracing::error!("Failed to delete customer: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete customer." }))) }
    }
}

async fn customer_ledger(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, customer_id, transaction_date, type, reference_no, debit, credit, balance
         FROM customer_ledger WHERE customer_id = ?1 ORDER BY id"
    ).unwrap();
    let entries: Vec<CustomerLedgerEntry> = stmt.query_map([id], |row| {
        Ok(CustomerLedgerEntry {
            id: row.get(0)?, customer_id: row.get(1)?, transaction_date: row.get(2)?,
            transaction_type: row.get(3)?, reference_no: row.get(4)?,
            debit: row.get(5)?, credit: row.get(6)?, balance: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": entries })))
}

async fn customer_statement(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let start = params.get("start_date").map(|s| s.as_str()).unwrap_or("2000-01-01");
    let end = params.get("end_date").map(|s| s.as_str()).unwrap_or("2099-12-31");
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, customer_id, transaction_date, type, reference_no, debit, credit, balance
         FROM customer_ledger WHERE customer_id = ?1 AND transaction_date BETWEEN ?2 AND ?3 ORDER BY id"
    ).unwrap();
    let entries: Vec<CustomerLedgerEntry> = stmt.query_map(rusqlite::params![id, start, end], |row| {
        Ok(CustomerLedgerEntry {
            id: row.get(0)?, customer_id: row.get(1)?, transaction_date: row.get(2)?,
            transaction_type: row.get(3)?, reference_no: row.get(4)?,
            debit: row.get(5)?, credit: row.get(6)?, balance: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": entries })))
}

async fn customer_balance(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row("SELECT current_balance FROM customers WHERE id = ?1", [id], |row| row.get::<_, f64>(0));
    match result {
        Ok(balance) => (StatusCode::OK, Json(json!({ "success": true, "data": { "balance": balance } }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Customer not found." }))),
    }
}

async fn customer_payments(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.payment_no, p.customer_id, c.customer_name, p.invoice_id,
                p.payment_date, p.amount, p.payment_method, p.reference, p.notes,
                p.created_by, p.created_at
         FROM payments p LEFT JOIN customers c ON p.customer_id = c.id
         WHERE p.customer_id = ?1 ORDER BY p.payment_date DESC"
    ).unwrap();
    let items: Vec<Payment> = stmt.query_map([id], |row| {
        Ok(Payment {
            id: row.get(0)?, payment_no: row.get(1)?, customer_id: row.get(2)?,
            customer_name: row.get(3)?, invoice_id: row.get(4)?, payment_date: row.get(5)?,
            amount: row.get(6)?, payment_method: row.get(7)?, reference: row.get(8)?,
            notes: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn recalculate_balances(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let customer_ids: Vec<i64> = {
        let mut stmt = db.prepare("SELECT id FROM customers WHERE is_active = 1").unwrap();
        stmt.query_map([], |row| row.get(0)).unwrap().filter_map(|r| r.ok()).collect()
    };
    for cid in &customer_ids {
        let balance: f64 = db.query_row(
            "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM customer_ledger WHERE customer_id = ?1",
            [cid],
            |row| row.get(0),
        ).unwrap_or(0.0);
        db.execute("UPDATE customers SET current_balance = ?1 WHERE id = ?2", rusqlite::params![balance, cid]).ok();
    }
    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": format!("Recalculated {} customers.", customer_ids.len()) } })))
}
