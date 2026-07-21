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
        // Suppliers
        .route("/api/suppliers", get(list_suppliers).post(create_supplier))
        .route("/api/suppliers/{id}", get(get_supplier).put(update_supplier).delete(delete_supplier))
        .route("/api/suppliers/next-code", get(next_supplier_code))
        // Purchase Orders
        .route("/api/purchase-orders", get(list_purchase_orders).post(create_purchase_order))
        .route("/api/purchase-orders/{id}", get(get_purchase_order).put(update_purchase_order).delete(delete_purchase_order))
        .route("/api/purchase-orders/{id}/status", post(update_po_status))
        .route("/api/purchase-orders/{id}/receipts", get(list_po_receipts).post(create_goods_receipt))
        .route("/api/purchase-orders/{id}/return-receipt", post(return_receipt))
        .route("/api/purchase-orders/pending", get(list_pending_pos))
        .route("/api/purchase-orders/summary/supplier/{id}", get(po_summary_by_supplier))
        .route("/api/purchase-orders/suppliers/{id}/balance", get(supplier_po_balance))
        .route("/api/purchase-orders/suppliers/{id}/transactions", get(supplier_po_transactions))
        .route("/api/suppliers/{id}/ledger", get(supplier_ledger))
        .route("/api/suppliers/{id}/payments", post(create_supplier_payment))
        .route("/api/purchase-orders/{id}/items", post(add_po_item))
        .route("/api/purchase-orders/{id}/items/{item_id}", put(update_po_item).delete(delete_po_item))
        // Direct Purchases
        .route("/api/purchases", get(list_direct_purchases).post(create_direct_purchase))
        .route("/api/purchases/{id}", get(get_direct_purchase).put(update_direct_purchase).delete(delete_direct_purchase))
        .route("/api/purchases/{id}/return", post(return_direct_purchase))
        // Global lists
        .route("/api/receipts", get(list_receipts))
        .route("/api/purchase-returns", get(list_purchase_returns))
}

// ============================================================================
// Suppliers
// ============================================================================

async fn list_suppliers(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, supplier_code, supplier_name, email, phone, address, is_active, created_at
         FROM suppliers WHERE is_active = 1 ORDER BY supplier_code"
    ).unwrap();
    let items: Vec<Supplier> = stmt.query_map([], |row| {
        Ok(Supplier {
            id: row.get(0)?, supplier_code: row.get(1)?, supplier_name: row.get(2)?,
            email: row.get(3)?, phone: row.get(4)?, address: row.get(5)?,
            is_active: row.get::<_, i64>(6)? != 0, created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_supplier(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, supplier_code, supplier_name, email, phone, address, is_active, created_at
         FROM suppliers WHERE id = ?1",
        [id],
        |row| Ok(Supplier {
            id: row.get(0)?, supplier_code: row.get(1)?, supplier_name: row.get(2)?,
            email: row.get(3)?, phone: row.get(4)?, address: row.get(5)?,
            is_active: row.get::<_, i64>(6)? != 0, created_at: row.get(7)?,
        }),
    );
    match result {
        Ok(s) => (StatusCode::OK, Json(json!({ "success": true, "data": s }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Supplier not found." }))),
    }
}

async fn create_supplier(State(_state): State<AppState>, Json(form): Json<SupplierForm>) -> impl IntoResponse {
    if form.supplier_code.trim().is_empty() || form.supplier_name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Supplier code and name are required." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM suppliers WHERE supplier_code = ?1", [&form.supplier_code], |row| row.get(0)).unwrap_or(false);
    if exists { return (StatusCode::CONFLICT, Json(json!({ "success": false, "error": "Supplier code already exists." }))); }
    let result = db.execute(
        "INSERT INTO suppliers (supplier_code, supplier_name, email, phone, address) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![form.supplier_code, form.supplier_name, form.email.as_deref().unwrap_or(""),
            form.phone.as_deref().unwrap_or(""), form.address.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            let s = db.query_row("SELECT id, supplier_code, supplier_name, email, phone, address, is_active, created_at FROM suppliers WHERE id = ?1", [id],
                |row| Ok(Supplier { id: row.get(0)?, supplier_code: row.get(1)?, supplier_name: row.get(2)?, email: row.get(3)?, phone: row.get(4)?, address: row.get(5)?, is_active: row.get::<_, i64>(6)? != 0, created_at: row.get(7)? })).unwrap();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": s })))
        }
        Err(e) => { tracing::error!("Failed to create supplier: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create supplier." }))) }
    }
}

async fn update_supplier(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SupplierForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE suppliers SET supplier_code=?1, supplier_name=?2, email=?3, phone=?4, address=?5 WHERE id=?6",
        rusqlite::params![form.supplier_code, form.supplier_name, form.email.as_deref().unwrap_or(""), form.phone.as_deref().unwrap_or(""), form.address.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Supplier updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Supplier not found." }))),
        Err(e) => { tracing::error!("Failed to update supplier: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update supplier." }))) }
    }
}

async fn delete_supplier(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE suppliers SET is_active = 0 WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Supplier deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Supplier not found." }))),
        Err(e) => { tracing::error!("Failed to delete supplier: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete supplier." }))) }
    }
}

async fn next_supplier_code(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let count: i64 = db.query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0)).unwrap_or(0);
    let code = format!("SUP-{:04}", count + 1);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "next_code": code } })))
}

// ============================================================================
// Purchase Orders
// ============================================================================

async fn list_purchase_orders(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.prepare(
        "SELECT po.id, po.po_no, po.supplier_id, po.po_date, po.status,
                po.total_amount, po.warehouse_id, po.notes, po.created_at, po.updated_at
         FROM purchase_orders po ORDER BY po.created_at DESC"
    ).and_then(|mut stmt| {
        stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let po_no: String = row.get(1)?;
            let supplier_id: i64 = row.get(2)?;
            let po_date: String = row.get(3)?;
            let status: String = row.get(4)?;
            let total_amount: f64 = row.get(5)?;
            let warehouse_id: Option<i64> = row.get(6)?;
            let notes: Option<String> = row.get(7)?;
            let created_at: String = row.get(8)?;
            let updated_at: String = row.get(9)?;
            let (supplier_name, supplier_code): (Option<String>, Option<String>) = db.query_row(
                "SELECT supplier_name, supplier_code FROM suppliers WHERE id = ?1",
                [supplier_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ).unwrap_or((None, None));
            let item_count: i64 = db.query_row(
                "SELECT COUNT(*) FROM purchase_order_items WHERE po_id = ?1",
                [id],
                |r| r.get(0),
            ).unwrap_or(0);
            Ok(PurchaseOrder {
                id, po_no, supplier_id, supplier_name, supplier_code,
                po_date, status, total_amount,
                expected_date: None, warehouse_id, notes,
                created_by: None, created_at, updated_at, item_count,
            })
        }).map(|iter| iter.filter_map(|r| r.ok()).collect::<Vec<PurchaseOrder>>())
    });
    match result {
        Ok(items) => (StatusCode::OK, Json(json!({ "success": true, "data": items }))),
        Err(e) => {
            tracing::error!("Failed to list POs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to list purchase orders." })))
        }
    }
}

async fn get_purchase_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // First check if PO exists
    let exists: bool = db.query_row(
        "SELECT COUNT(*) > 0 FROM purchase_orders WHERE id = ?1", [id],
        |row| row.get(0),
    ).unwrap_or(false);
    if !exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." })));
    }

    // Build PO from simple columns (avoid query issues with missing columns)
    let po = db.query_row(
        "SELECT po.id, po.po_no, po.supplier_id, po.po_date, po.status,
                po.total_amount, po.warehouse_id, po.notes, po.created_at, po.updated_at
         FROM purchase_orders po WHERE po.id = ?1",
        [id],
        |row| {
            let id: i64 = row.get(0)?;
            let po_no: String = row.get(1)?;
            let supplier_id: i64 = row.get(2)?;
            let po_date: String = row.get(3)?;
            let status: String = row.get(4)?;
            let total_amount: f64 = row.get(5)?;
            let warehouse_id: Option<i64> = row.get(6)?;
            let notes: Option<String> = row.get(7)?;
            let created_at: String = row.get(8)?;
            let updated_at: String = row.get(9)?;
            // Get supplier info separately
            let (supplier_name, supplier_code): (Option<String>, Option<String>) = db.query_row(
                "SELECT supplier_name, supplier_code FROM suppliers WHERE id = ?1",
                [supplier_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ).unwrap_or((None, None));
            let item_count: i64 = db.query_row(
                "SELECT COUNT(*) FROM purchase_order_items WHERE po_id = ?1",
                [id],
                |r| r.get(0),
            ).unwrap_or(0);
            Ok(PurchaseOrder {
                id, po_no, supplier_id, supplier_name, supplier_code,
                po_date, status, total_amount,
                expected_date: None, warehouse_id, notes,
                created_by: None, created_at, updated_at, item_count,
            })
        },
    );

    match po {
        Ok(po) => {
            let items: Vec<PurchaseOrderItem> = db.prepare(
                "SELECT poi.id, poi.po_id, poi.item_id, i.item_name, i.item_code,
                        poi.description, poi.quantity, poi.received_quantity, poi.returned_quantity,
                        poi.unit_price, poi.amount
                 FROM purchase_order_items poi LEFT JOIN items i ON poi.item_id = i.id
                 WHERE poi.po_id = ?1"
            ).and_then(|mut stmt| {
                stmt.query_map([id], |row| {
                    Ok(PurchaseOrderItem {
                        id: row.get(0)?, po_id: row.get(1)?, item_id: row.get(2)?,
                        item_name: row.get(3)?, item_code: row.get(4)?, description: row.get(5)?,
                        quantity: row.get(6)?, received_quantity: row.get(7)?,
                        returned_quantity: row.get(8)?, unit_price: row.get(9)?, amount: row.get(10)?,
                    })
                }).map(|iter| iter.filter_map(|r| r.ok()).collect::<Vec<PurchaseOrderItem>>())
            }).unwrap_or_default();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "purchase_order": po, "items": items } })))
        }
        Err(e) => {
            tracing::error!("Failed to get PO {}: {}", id, e);
            (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." })))
        }
    }
}

async fn create_purchase_order(State(_state): State<AppState>, Json(form): Json<PurchaseOrderForm>) -> impl IntoResponse {
    if form.items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one item is required." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM purchase_orders", [], |row| row.get(0)).unwrap_or(1);
    let po_no = format!("PO-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let total: f64 = form.items.iter().map(|i| i.quantity * i.unit_price).sum();

    let result = db.execute(
        "INSERT INTO purchase_orders (po_no, supplier_id, po_date, status, total_amount, warehouse_id, notes)
         VALUES (?1, ?2, ?3, 'Draft', ?4, ?5, ?6)",
        rusqlite::params![po_no, form.supplier_id, form.po_date, total, form.warehouse_id, form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let po_id = db.last_insert_rowid();
            for item in &form.items {
                let amount = item.quantity * item.unit_price;
                if let Err(e) = db.execute(
                    "INSERT INTO purchase_order_items (po_id, item_id, description, quantity, unit_price, amount)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![po_id, item.item_id, item.description.as_deref().unwrap_or(""), item.quantity, item.unit_price, amount],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create PO item: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create purchase order item." })));
                }
            }
            let total_items = form.items.len() as i64;
            // Insert supplier ledger entry for PO
            {
                let last_balance: f64 = db.query_row(
                    "SELECT COALESCE(balance, 0) FROM supplier_ledger WHERE supplier_id = ?1 ORDER BY id DESC LIMIT 1",
                    [form.supplier_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);
                let new_balance = last_balance + total;
                if let Err(e) = db.execute(
                    "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
                     VALUES (?1, ?2, 'PURCHASE', ?3, ?4, 0, ?5)",
                    rusqlite::params![form.supplier_id, form.po_date, po_no, total, new_balance],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create supplier ledger entry: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create supplier ledger entry." })));
                }
            }
            // Auto-journal: debit Inventory (account_id=3), credit AP (account_id=6)
            {
                if let Err(e) = db.execute(
                    "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('purchase_order', ?1, ?2)",
                    rusqlite::params![po_id, form.po_date],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create PO journal entry: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create purchase order journal entry." })));
                }
                let je_id = db.last_insert_rowid();
                if let Err(e) = db.execute(
                    "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                     VALUES (?1, 3, ?2, 0, ?3, ?4),
                            (?1, 6, 0, ?2, ?5, ?4)",
                    rusqlite::params![je_id, total, format!("PO {} - Inventory", po_no), form.po_date, format!("AP - PO {}", po_no)],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create PO journal lines: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create purchase order journal lines." })));
                }
            }
            let item_count_result = db.query_row(
                "SELECT COUNT(*) FROM purchase_order_items WHERE po_id = ?1",
                [po_id],
                |row| row.get::<_, i64>(0),
            ).unwrap_or(total_items);
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit PO: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            let po = json!({
                "id": po_id,
                "po_no": po_no,
                "supplier_id": form.supplier_id,
                "po_date": form.po_date,
                "status": "Draft",
                "total_amount": total,
                "warehouse_id": form.warehouse_id,
                "notes": form.notes,
                "item_count": item_count_result,
            });
            (StatusCode::CREATED, Json(json!({ "success": true, "data": po })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create PO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create purchase order." }))) }
    }
}

async fn update_purchase_order(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<PurchaseOrderForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let total: f64 = form.items.iter().map(|i| i.quantity * i.unit_price).sum();
    let result = db.execute(
        "UPDATE purchase_orders SET supplier_id=?1, po_date=?2, total_amount=?3, warehouse_id=?4, notes=?5, updated_at=datetime('now') WHERE id=?6",
        rusqlite::params![form.supplier_id, form.po_date, total, form.warehouse_id, form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            db.execute("DELETE FROM purchase_order_items WHERE po_id = ?1", [id]).ok();
            for item in &form.items {
                let amount = item.quantity * item.unit_price;
                db.execute(
                    "INSERT INTO purchase_order_items (po_id, item_id, description, quantity, unit_price, amount) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![id, item.item_id, item.description.as_deref().unwrap_or(""), item.quantity, item.unit_price, amount],
                ).ok();
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Purchase order updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." }))),
        Err(e) => { tracing::error!("Failed to update PO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update purchase order." }))) }
    }
}

async fn delete_purchase_order(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // 1. Get PO data
    let (supplier_id, total_amount, po_no): (i64, f64, String) = match db.query_row(
        "SELECT supplier_id, total_amount, po_no FROM purchase_orders WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ) {
        Ok(v) => v,
        Err(_) => { let _ = db.execute_batch("ROLLBACK"); return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." }))); }
    };

    // 2. Reverse supplier ledger
    if supplier_id > 0 && total_amount > 0.0 {
        let last_balance: f64 = db.query_row(
            "SELECT COALESCE(balance, 0) FROM supplier_ledger WHERE supplier_id = ?1 ORDER BY id DESC LIMIT 1",
            [supplier_id], |row| row.get(0),
        ).unwrap_or(0.0);
        let _ = db.execute(
            "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (?1, datetime('now'), 'PO_CANCELLATION', ?2, 0, ?3, ?4)",
            rusqlite::params![supplier_id, format!("DEL-{}", po_no), total_amount, last_balance - total_amount],
        );
    }

    // 3. Reverse GL entry
    let je_id: Option<i64> = db.query_row(
        "SELECT id FROM journal_entries WHERE reference_type = 'purchase_order' AND reference_id = ?1",
        [id],
        |row| row.get(0),
    ).ok();
    if let Some(je) = je_id {
        let _ = db.execute("DELETE FROM journal_lines WHERE journal_entry_id = ?1", [je]);
        let _ = db.execute("DELETE FROM journal_entries WHERE id = ?1", [je]);
    }

    // 4. Delete records
    let _ = db.execute("DELETE FROM purchase_order_items WHERE po_id = ?1", [id]);
    let result = db.execute("DELETE FROM purchase_orders WHERE id = ?1", [id]);

    match result {
        Ok(rows) if rows > 0 => {
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit PO deletion: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete PO (transaction rolled back)." })));
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Purchase order deleted and ledger reversed." } })))
        }
        Ok(_) => { let _ = db.execute_batch("ROLLBACK"); (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." }))) }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to delete PO: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete purchase order." }))) }
    }
}

async fn update_po_status(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<PurchaseOrderStatusUpdate>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE purchase_orders SET status=?1, updated_at=datetime('now') WHERE id=?2",
        rusqlite::params![form.status, id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Status updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase order not found." }))),
        Err(e) => { tracing::error!("Failed to update PO status: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update status." }))) }
    }
}

async fn list_pending_pos(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT po.id, po.po_no, po.supplier_id, s.supplier_name, s.supplier_code, po.po_date, po.status,
                po.total_amount, po.expected_date, po.warehouse_id, po.notes, po.created_by, po.created_at, po.updated_at,
                (SELECT COUNT(*) FROM purchase_order_items poi WHERE poi.po_id = po.id) AS item_count
         FROM purchase_orders po LEFT JOIN suppliers s ON po.supplier_id = s.id
         WHERE po.status IN ('Draft', 'Approved') ORDER BY po.created_at DESC"
    ).unwrap();
    let items: Vec<PurchaseOrder> = stmt.query_map([], |row| {
        Ok(PurchaseOrder {
            id: row.get(0)?, po_no: row.get(1)?, supplier_id: row.get(2)?,
            supplier_name: row.get(3)?, supplier_code: row.get(4)?, po_date: row.get(5)?, status: row.get(6)?,
            total_amount: row.get(7)?, expected_date: row.get(8)?,
            warehouse_id: row.get(9)?, notes: row.get(10)?,
            created_by: row.get(11)?, created_at: row.get(12)?, updated_at: row.get(13)?,
            item_count: row.get(14)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_po_receipts(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT gr.id, gr.receipt_no, gr.po_id, gr.receipt_date, gr.warehouse_id, gr.notes,
                gr.created_by, gr.created_at
         FROM goods_receipts gr WHERE gr.po_id = ?1 ORDER BY gr.created_at DESC"
    ).unwrap();
    let items: Vec<GoodsReceipt> = stmt.query_map([id], |row| {
        Ok(GoodsReceipt {
            id: row.get(0)?, receipt_no: row.get(1)?, po_id: row.get(2)?,
            receipt_date: row.get(3)?, warehouse_id: row.get(4)?, notes: row.get(5)?,
            created_by: row.get(6)?, created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_goods_receipt(State(_state): State<AppState>, Path(po_id): Path<i64>, Json(form): Json<GoodsReceiptForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM goods_receipts", [], |row| row.get(0)).unwrap_or(1);
    let rn = format!("GR-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    let result = db.execute(
        "INSERT INTO goods_receipts (receipt_no, po_id, receipt_date, warehouse_id, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![rn, po_id, form.receipt_date, form.warehouse_id, form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let gr_id = db.last_insert_rowid();
            for item in &form.items {
                if let Err(e) = db.execute(
                    "INSERT INTO goods_receipt_items (receipt_id, po_item_id, item_id, received_quantity) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![gr_id, item.po_item_id, item.item_id, item.received_quantity],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create GRN item: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create goods receipt item." })));
                }
                if let Err(e) = db.execute(
                    "UPDATE purchase_order_items SET received_quantity = received_quantity + ?1 WHERE id = ?2",
                    rusqlite::params![item.received_quantity, item.po_item_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update PO item received qty: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update purchase order item." })));
                }

                // Look up unit cost from PO item
                let unit_cost: f64 = db.query_row(
                    "SELECT unit_price FROM purchase_order_items WHERE id = ?1",
                    [item.po_item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);

                // Add stock
                if let Some(wh_id) = form.warehouse_id {
                    let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                        rusqlite::params![item.item_id, wh_id], |row| row.get(0)).unwrap_or(false);
                    if exists {
                        if let Err(e) = db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                            rusqlite::params![item.received_quantity, item.item_id, wh_id]) {
                            let _ = db.execute_batch("ROLLBACK");
                            tracing::error!("Failed to update stock balance: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update stock balance." })));
                        }
                    } else {
                        if let Err(e) = db.execute("INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                            rusqlite::params![item.item_id, wh_id, item.received_quantity]) {
                            let _ = db.execute_batch("ROLLBACK");
                            tracing::error!("Failed to insert stock balance: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock balance." })));
                        }
                    }
                    if let Err(e) = db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                        rusqlite::params![item.received_quantity, item.item_id]) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to update item stock: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update item stock." })));
                    }

                    // Create stock batch for FIFO tracking
                    let batch_no = format!("{}-BATCH-{}", rn, item.po_item_id);
                    if let Err(e) = db.execute(
                        "INSERT INTO stock_batches (batch_no, item_id, warehouse_id, source_type, source_id,
                            quantity_original, quantity_remaining, unit_cost, received_date)
                         VALUES (?1, ?2, ?3, 'GRN', ?4, ?5, ?5, ?6, ?7)",
                        rusqlite::params![batch_no, item.item_id, wh_id, gr_id,
                            item.received_quantity, unit_cost, form.receipt_date],
                    ) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to create stock batch: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock batch." })));
                    }

                    // Create stock movement (IN)
                    let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                    let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
                    if let Err(e) = db.execute(
                        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                         VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'GRN', ?6, ?7)",
                        rusqlite::params![mno, item.item_id, wh_id, item.received_quantity, unit_cost, rn, format!("Goods Receipt {}", rn)],
                    ) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to create stock movement: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock movement." })));
                    }
                }
            }
            // Insert supplier ledger entry for goods receipt
            {
                let supplier_id: i64 = db.query_row("SELECT supplier_id FROM purchase_orders WHERE id = ?1", [po_id], |row| row.get(0)).unwrap_or(0);
                let receipt_total: f64 = db.query_row(
                    "SELECT COALESCE(SUM(gri.received_quantity * poi.unit_price), 0)
                     FROM goods_receipt_items gri
                     JOIN purchase_order_items poi ON gri.po_item_id = poi.id
                     WHERE gri.receipt_id = ?1",
                    [gr_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);
                if supplier_id > 0 && receipt_total > 0.0 {
                    let last_balance: f64 = db.query_row(
                        "SELECT COALESCE(balance, 0) FROM supplier_ledger WHERE supplier_id = ?1 ORDER BY id DESC LIMIT 1",
                        [supplier_id],
                        |row| row.get(0),
                    ).unwrap_or(0.0);
                    let new_balance = last_balance + receipt_total;
                    if let Err(e) = db.execute(
                        "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
                         VALUES (?1, ?2, 'RECEIPT', ?3, ?4, 0, ?5)",
                        rusqlite::params![supplier_id, form.receipt_date, rn, receipt_total, new_balance],
                    ) {
                        let _ = db.execute_batch("ROLLBACK");
                        tracing::error!("Failed to create supplier ledger entry: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create supplier ledger entry." })));
                    }
                }
            }
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit goods receipt: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": gr_id, "receipt_no": rn } })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create goods receipt: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create goods receipt." }))) }
    }
}

async fn return_receipt(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<serde_json::Value>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Get receipt details
    let receipt: Option<(i64, String, i64)> = db.query_row(
        "SELECT id, receipt_no, warehouse_id FROM goods_receipts WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok();

    let (receipt_id, receipt_no, warehouse_id) = match receipt {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Receipt not found." }))),
    };

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // Get items to return
    let mut stmt = db.prepare(
        "SELECT item_id, received_quantity FROM goods_receipt_items WHERE receipt_id = ?1"
    ).unwrap();
    let items_to_return: Vec<(i64, f64)> = stmt.query_map([receipt_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).unwrap().filter_map(|r| r.ok()).collect();

    for (item_id, quantity) in &items_to_return {
        // Get unit cost
        let unit_cost: f64 = db.query_row(
            "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
            [item_id],
            |row| row.get(0),
        ).unwrap_or(0.0);

        // Create stock movement (OUT - goods returning to supplier)
        let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
        let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
        if let Err(e) = db.execute(
            "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
             VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'RECEIPT_RETURN', ?6, ?7)",
            rusqlite::params![mno, item_id, warehouse_id, quantity, unit_cost, receipt_no, format!("Receipt Return {}", receipt_no)],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to create return stock movement: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock movement." })));
        }

        // Update stock_balances
        if let Err(e) = db.execute(
            "UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
            rusqlite::params![quantity, item_id, warehouse_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update stock balance: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update stock balance." })));
        }

        // Update items.current_stock
        if let Err(e) = db.execute(
            "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![quantity, item_id],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to update item stock: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update item stock." })));
        }
    }

    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit receipt return: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Receipt return recorded.", "items_returned": items_to_return.len() } })))
}

// ============================================================================
// Direct Purchases
// ============================================================================

async fn list_direct_purchases(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.purchase_no, p.item_id, i.item_name, i.item_code, p.warehouse_id,
                w.warehouse_name, p.batch_id, p.quantity, p.unit_cost, p.total_cost,
                p.supplier_name, p.purchase_date, p.status, p.notes, p.created_by, p.created_at
         FROM purchases p
         LEFT JOIN items i ON p.item_id = i.id
         LEFT JOIN warehouses w ON p.warehouse_id = w.id
         ORDER BY p.created_at DESC"
    ).unwrap();
    let items: Vec<DirectPurchase> = stmt.query_map([], |row| {
        Ok(DirectPurchase {
            id: row.get(0)?, purchase_no: row.get(1)?, item_id: row.get(2)?,
            item_name: row.get(3)?, item_code: row.get(4)?, warehouse_id: row.get(5)?,
            warehouse_name: row.get(6)?, batch_id: row.get(7)?, quantity: row.get(8)?,
            unit_cost: row.get(9)?, total_cost: row.get(10)?, supplier_name: row.get(11)?,
            purchase_date: row.get(12)?, status: row.get(13)?,
            notes: row.get(14)?, created_by: row.get(15)?,
            created_at: row.get(16)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_direct_purchase(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT p.id, p.purchase_no, p.item_id, i.item_name, i.item_code, p.warehouse_id,
                w.warehouse_name, p.batch_id, p.quantity, p.unit_cost, p.total_cost,
                p.supplier_name, p.purchase_date, p.status, p.notes, p.created_by, p.created_at
         FROM purchases p LEFT JOIN items i ON p.item_id = i.id LEFT JOIN warehouses w ON p.warehouse_id = w.id
         WHERE p.id = ?1",
        [id],
        |row| Ok(DirectPurchase {
            id: row.get(0)?, purchase_no: row.get(1)?, item_id: row.get(2)?,
            item_name: row.get(3)?, item_code: row.get(4)?, warehouse_id: row.get(5)?,
            warehouse_name: row.get(6)?, batch_id: row.get(7)?, quantity: row.get(8)?,
            unit_cost: row.get(9)?, total_cost: row.get(10)?, supplier_name: row.get(11)?,
            purchase_date: row.get(12)?, status: row.get(13)?,
            notes: row.get(14)?, created_by: row.get(15)?,
            created_at: row.get(16)?,
        }),
    );
    match result {
        Ok(p) => (StatusCode::OK, Json(json!({ "success": true, "data": p }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase not found." }))),
    }
}

async fn create_direct_purchase(State(_state): State<AppState>, Json(form): Json<DirectPurchaseForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM purchases", [], |row| row.get(0)).unwrap_or(1);
    let pno = format!("PUR-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let total = form.quantity * form.unit_cost;

    let result = db.execute(
        "INSERT INTO purchases (purchase_no, item_id, warehouse_id, quantity, unit_cost, total_cost, supplier_name, purchase_date, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![pno, form.item_id, form.warehouse_id, form.quantity, form.unit_cost, total,
            form.supplier_name, form.purchase_date, form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let purchase_id = db.last_insert_rowid();

            // Update stock balances
            let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                rusqlite::params![form.item_id, form.warehouse_id], |row| row.get(0)).unwrap_or(false);
            if exists {
                if let Err(e) = db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![form.quantity, form.item_id, form.warehouse_id]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update stock balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update stock balance." })));
                }
            } else {
                if let Err(e) = db.execute("INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                    rusqlite::params![form.item_id, form.warehouse_id, form.quantity]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert stock balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock balance." })));
                }
            }
            if let Err(e) = db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![form.quantity, form.item_id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update item stock: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update item stock." })));
            }

            // Create stock batch for FIFO tracking
            let batch_no = format!("{}-BATCH", pno);
            if let Err(e) = db.execute(
                "INSERT INTO stock_batches (batch_no, item_id, warehouse_id, source_type, source_id,
                    quantity_original, quantity_remaining, unit_cost, received_date)
                 VALUES (?1, ?2, ?3, 'PURCHASE', ?4, ?5, ?5, ?6, ?7)",
                rusqlite::params![batch_no, form.item_id, form.warehouse_id, purchase_id,
                    form.quantity, form.unit_cost, form.purchase_date],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to create stock batch: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock batch." })));
            }
            let batch_id = db.last_insert_rowid();
            if let Err(e) = db.execute("UPDATE purchases SET batch_id = ?1 WHERE id = ?2",
                rusqlite::params![batch_id, purchase_id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update purchase batch_id: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update purchase record." })));
            }

            // Create stock movement (IN)
            let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
            let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
            if let Err(e) = db.execute(
                "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                 VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'PURCHASE', ?6, ?7)",
                rusqlite::params![mno, form.item_id, form.warehouse_id, form.quantity, form.unit_cost, pno, format!("Direct Purchase {}", pno)],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to create stock movement: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create stock movement." })));
            }

            // Note: Direct purchases use supplier_name (not supplier_id), so ledger entry
            // is only created if a supplier_id can be resolved. For now, skip ledger entry
            // for direct purchases without a linked supplier record.

            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit direct purchase: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": purchase_id, "purchase_no": pno } })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create direct purchase: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create purchase." }))) }
    }
}

async fn delete_direct_purchase(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Reverse stock before deleting
    let old: Option<(f64, i64, i64)> = db.query_row(
        "SELECT quantity, item_id, warehouse_id FROM purchases WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok();
    if let Some((qty, item_id, warehouse_id)) = old {
        db.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
            rusqlite::params![qty, item_id, warehouse_id]).ok();
        db.execute("UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![qty, item_id]).ok();
    }

    // Delete the stock batch
    db.execute("DELETE FROM stock_batches WHERE source_type = 'PURCHASE' AND source_id = ?1", [id]).ok();

    let result = db.execute("DELETE FROM purchases WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Purchase deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase not found." }))),
        Err(e) => { tracing::error!("Failed to delete purchase: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete purchase." }))) }
    }
}

async fn update_direct_purchase(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<DirectPurchaseForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let total = form.quantity * form.unit_cost;

    // Get the old quantities for stock adjustment
    let old: Option<(f64, i64, i64)> = db.query_row(
        "SELECT quantity, item_id, warehouse_id FROM purchases WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok();
    let (old_qty, old_item_id, old_warehouse_id) = match old {
        Some(v) => v,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase not found." }))),
    };

    let result = db.execute(
        "UPDATE purchases SET item_id=?1, warehouse_id=?2, quantity=?3, unit_cost=?4, total_cost=?5,
         supplier_name=?6, purchase_date=?7, notes=?8 WHERE id=?9",
        rusqlite::params![form.item_id, form.warehouse_id, form.quantity, form.unit_cost, total,
            form.supplier_name, form.purchase_date, form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            // Adjust stock balances: reverse old, apply new
            if old_item_id == form.item_id && old_warehouse_id == form.warehouse_id {
                let delta = form.quantity - old_qty;
                db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![delta, form.item_id, form.warehouse_id]).ok();
                db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![delta, form.item_id]).ok();
            } else {
                // Different item or warehouse: reverse old, add new
                db.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![old_qty, old_item_id, old_warehouse_id]).ok();
                db.execute("UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![old_qty, old_item_id]).ok();
                let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                    rusqlite::params![form.item_id, form.warehouse_id], |row| row.get(0)).unwrap_or(false);
                if exists {
                    db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                        rusqlite::params![form.quantity, form.item_id, form.warehouse_id]).ok();
                } else {
                    db.execute("INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                        rusqlite::params![form.item_id, form.warehouse_id, form.quantity]).ok();
                }
                db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![form.quantity, form.item_id]).ok();
            }

            // Update the stock batch
            db.execute(
                "UPDATE stock_batches SET item_id=?1, warehouse_id=?2, quantity_original=?3,
                    quantity_remaining=quantity_remaining + (?3 - quantity_original), unit_cost=?4, received_date=?5
                 WHERE source_type='PURCHASE' AND source_id=?6",
                rusqlite::params![form.item_id, form.warehouse_id, form.quantity, form.unit_cost, form.purchase_date, id],
            ).ok();

            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Purchase updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase not found." }))),
        Err(e) => { tracing::error!("Failed to update purchase: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update purchase." }))) }
    }
}

async fn return_direct_purchase(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Get the purchase details
    let purchase: Option<(i64, f64, i64, i64, String)> = db.query_row(
        "SELECT id, quantity, item_id, warehouse_id, purchase_no FROM purchases WHERE id = ?1 AND status != 'Returned'",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).ok();

    let (purchase_id, quantity, item_id, warehouse_id, purchase_no) = match purchase {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Purchase not found or already returned." }))),
    };

    // Get unit cost
    let unit_cost: f64 = db.query_row(
        "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
        [item_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Create stock movement (OUT - goods leaving back to supplier)
    let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
    let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
    db.execute(
        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
         VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'PURCHASE_RETURN', ?6, ?7)",
        rusqlite::params![mno, item_id, warehouse_id, quantity, unit_cost, purchase_no, format!("Purchase Return {}", purchase_no)],
    ).ok();

    // Update stock_balances
    db.execute(
        "UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
        rusqlite::params![quantity, item_id, warehouse_id],
    ).ok();

    // Update items.current_stock
    db.execute(
        "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![quantity, item_id],
    ).ok();

    // Mark purchase as returned
    db.execute(
        "UPDATE purchases SET status = 'Returned', updated_at = datetime('now') WHERE id = ?1",
        [purchase_id],
    ).ok();

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Purchase return recorded.", "movement_no": mno } })))
}

async fn list_receipts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT gr.id, gr.receipt_no, gr.po_id, gr.receipt_date, gr.warehouse_id,
                w.warehouse_name, gr.notes, gr.created_by, gr.created_at
         FROM goods_receipts gr
         LEFT JOIN warehouses w ON gr.warehouse_id = w.id
         ORDER BY gr.created_at DESC"
    ).unwrap();
    let items: Vec<GoodsReceipt> = stmt.query_map([], |row| {
        Ok(GoodsReceipt {
            id: row.get(0)?, receipt_no: row.get(1)?, po_id: row.get(2)?,
            receipt_date: row.get(3)?, warehouse_id: row.get(4)?, notes: row.get(5)?,
            created_by: row.get(6)?, created_at: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_purchase_returns(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.purchase_no, p.item_id, i.item_name, i.item_code,
                p.quantity, p.unit_cost, p.total_cost, p.supplier_name, p.purchase_date, p.status, p.notes,
                p.created_by, p.created_at
         FROM purchases p LEFT JOIN items i ON p.item_id = i.id
         ORDER BY p.created_at DESC"
    ).unwrap();
    let items: Vec<DirectPurchase> = stmt.query_map([], |row| {
        Ok(DirectPurchase {
            id: row.get(0)?, purchase_no: row.get(1)?, item_id: row.get(2)?,
            item_name: row.get(3)?, item_code: row.get(4)?, warehouse_id: 0,
            warehouse_name: None, batch_id: None, quantity: row.get(5)?,
            unit_cost: row.get(6)?, total_cost: row.get(7)?, supplier_name: row.get(8)?,
            purchase_date: row.get(9)?, status: row.get(10)?,
            notes: row.get(11)?, created_by: row.get(12)?,
            created_at: row.get(13)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn po_summary_by_supplier(State(_state): State<AppState>, Path(supplier_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT COUNT(*) as count, COALESCE(SUM(total_amount), 0) as total
         FROM purchase_orders WHERE supplier_id = ?1",
        [supplier_id],
        |row| Ok(json!({ "count": row.get::<_, i64>(0)?, "total_amount": row.get::<_, f64>(1)? })),
    );
    match result {
        Ok(data) => (StatusCode::OK, Json(json!({ "success": true, "data": data }))),
        Err(e) => { tracing::error!("Failed to get PO summary: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to get summary." }))) }
    }
}

async fn supplier_po_balance(State(_state): State<AppState>, Path(supplier_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    // Use ledger balance directly: SUM(debit) - SUM(credit) gives outstanding amount
    let balance: f64 = db.query_row(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM supplier_ledger WHERE supplier_id = ?1",
        [supplier_id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "balance": balance } })))
}

async fn supplier_po_transactions(State(_state): State<AppState>, Path(supplier_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, po_no, po_date, status, total_amount FROM purchase_orders
         WHERE supplier_id = ?1 ORDER BY created_at DESC LIMIT 50"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([supplier_id], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "po_no": row.get::<_, String>(1)?,
            "po_date": row.get::<_, String>(2)?, "status": row.get::<_, String>(3)?,
            "total_amount": row.get::<_, f64>(4)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn add_po_item(
    State(_state): State<AppState>,
    Path(po_id): Path<i64>,
    Json(form): Json<PurchaseOrderItemForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let amount = form.quantity * form.unit_price;
    let result = db.execute(
        "INSERT INTO purchase_order_items (po_id, item_id, description, quantity, unit_price, amount) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![po_id, form.item_id, form.description.as_deref().unwrap_or(""), form.quantity, form.unit_price, amount],
    );
    match result {
        Ok(_) => {
            let item_id = db.last_insert_rowid();
            db.execute(
                "UPDATE purchase_orders SET total_amount = total_amount + ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![amount, po_id],
            ).ok();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": item_id, "amount": amount } })))
        }
        Err(e) => {
            tracing::error!("Failed to add PO item: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to add item." })))
        }
    }
}

async fn update_po_item(
    State(_state): State<AppState>,
    Path((po_id, item_id)): Path<(i64, i64)>,
    Json(form): Json<PurchaseOrderItemForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let old_amount: f64 = db.query_row(
        "SELECT amount FROM purchase_order_items WHERE id = ?1 AND po_id = ?2",
        rusqlite::params![item_id, po_id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    let new_amount = form.quantity * form.unit_price;
    let result = db.execute(
        "UPDATE purchase_order_items SET item_id = ?1, description = ?2, quantity = ?3, unit_price = ?4, amount = ?5 WHERE id = ?6 AND po_id = ?7",
        rusqlite::params![form.item_id, form.description.as_deref().unwrap_or(""), form.quantity, form.unit_price, new_amount, item_id, po_id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            db.execute(
                "UPDATE purchase_orders SET total_amount = total_amount - ?1 + ?2, updated_at = datetime('now') WHERE id = ?3",
                rusqlite::params![old_amount, new_amount, po_id],
            ).ok();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Item updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Item not found." }))),
        Err(e) => {
            tracing::error!("Failed to update PO item: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update item." })))
        }
    }
}

async fn delete_po_item(
    State(_state): State<AppState>,
    Path((po_id, item_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let amount: f64 = db.query_row(
        "SELECT amount FROM purchase_order_items WHERE id = ?1 AND po_id = ?2",
        rusqlite::params![item_id, po_id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    let result = db.execute("DELETE FROM purchase_order_items WHERE id = ?1 AND po_id = ?2", rusqlite::params![item_id, po_id]);
    match result {
        Ok(rows) if rows > 0 => {
            db.execute(
                "UPDATE purchase_orders SET total_amount = total_amount - ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![amount, po_id],
            ).ok();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Item removed." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Item not found." }))),
        Err(e) => {
            tracing::error!("Failed to delete PO item: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete item." })))
        }
    }
}

async fn supplier_ledger(
    State(_state): State<AppState>,
    Path(supplier_id): Path<i64>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let from_date = params.get("from_date").map(|s| s.as_str()).unwrap_or("2000-01-01");
    let to_date = params.get("to_date").map(|s| s.as_str()).unwrap_or("2099-12-31");
    let mut stmt = db.prepare(
        "SELECT id, supplier_id, transaction_date, type, reference_no, debit, credit, balance
         FROM supplier_ledger WHERE supplier_id = ?1 AND transaction_date BETWEEN ?2 AND ?3 ORDER BY id"
    ).unwrap();
    let entries: Vec<SupplierLedgerEntry> = stmt.query_map(rusqlite::params![supplier_id, from_date, to_date], |row| {
        Ok(SupplierLedgerEntry {
            id: row.get(0)?, supplier_id: row.get(1)?, transaction_date: row.get(2)?,
            transaction_type: row.get(3)?, reference_no: row.get(4)?,
            debit: row.get(5)?, credit: row.get(6)?, balance: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": entries })))
}

async fn create_supplier_payment(
    State(_state): State<AppState>,
    Path(supplier_id): Path<i64>,
    Json(form): Json<SupplierPaymentForm>,
) -> impl IntoResponse {
    if form.amount <= 0.0 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Payment amount must be positive." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let last_balance: f64 = db.query_row(
        "SELECT COALESCE(balance, 0) FROM supplier_ledger WHERE supplier_id = ?1 ORDER BY id DESC LIMIT 1",
        [supplier_id],
        |row| row.get(0),
    ).unwrap_or(0.0);
    let new_balance = last_balance - form.amount;
    let ref_no = format!("SPAY-{}-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"), supplier_id);
    let result = db.execute(
        "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
         VALUES (?1, ?2, 'PAYMENT', ?3, 0, ?4, ?5)",
        rusqlite::params![supplier_id, form.payment_date, ref_no, form.amount, new_balance],
    );
    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id, "reference_no": ref_no } })))
        }
        Err(e) => {
            tracing::error!("Failed to create supplier payment: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record payment." })))
        }
    }
}
