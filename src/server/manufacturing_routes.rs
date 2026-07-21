use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/bom", get(list_boms).post(create_bom))
        .route("/api/bom/{id}", get(get_bom).put(update_bom).delete(delete_bom))
        .route("/api/bom/{id}/toggle-active", patch(toggle_bom_active))
        .route("/api/bom/by-item/{itemId}", get(get_bom_by_item))
        .route("/api/production/productions", get(list_productions).post(create_production))
        .route("/api/production/productions/{id}", get(get_production).put(update_production).delete(delete_production))
        .route("/api/production/productions/summary/item/{id}", get(production_summary_by_item))
}

// ============================================================================
// BOM
// ============================================================================

async fn list_boms(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT b.id, b.bom_no, b.bom_name, b.finished_item_id, i.item_name, i.item_code,
                b.quantity, b.version, b.is_active, b.created_at, b.updated_at,
                (SELECT COALESCE(SUM(bi.quantity * bi.unit_cost), 0) FROM bom_items bi WHERE bi.bom_id = b.id) AS total_cost
         FROM boms b LEFT JOIN items i ON b.finished_item_id = i.id
         ORDER BY b.created_at DESC"
    ).unwrap();
    let items: Vec<Bom> = stmt.query_map([], |row| {
        Ok(Bom {
            id: row.get(0)?, bom_no: row.get(1)?, bom_name: row.get(2)?,
            finished_item_id: row.get(3)?, finished_item_name: row.get(4)?,
            finished_item_code: row.get(5)?, quantity: row.get(6)?,
            version: row.get(7)?, is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?, updated_at: row.get(10)?,
            total_cost: row.get(11)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_bom(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT b.id, b.bom_no, b.bom_name, b.finished_item_id, i.item_name, i.item_code,
                b.quantity, b.version, b.is_active, b.created_at, b.updated_at,
                (SELECT COALESCE(SUM(bi.quantity * bi.unit_cost), 0) FROM bom_items bi WHERE bi.bom_id = b.id) AS total_cost
         FROM boms b LEFT JOIN items i ON b.finished_item_id = i.id WHERE b.id = ?1",
        [id],
        |row| Ok(Bom {
            id: row.get(0)?, bom_no: row.get(1)?, bom_name: row.get(2)?,
            finished_item_id: row.get(3)?, finished_item_name: row.get(4)?,
            finished_item_code: row.get(5)?, quantity: row.get(6)?,
            version: row.get(7)?, is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?, updated_at: row.get(10)?,
            total_cost: row.get(11)?,
        }),
    );
    match result {
        Ok(bom) => {
            let mut stmt = db.prepare(
                "SELECT bi.id, bi.bom_id, bi.item_id, i.item_name, i.item_code, bi.quantity, bi.unit_cost
                 FROM bom_items bi LEFT JOIN items i ON bi.item_id = i.id WHERE bi.bom_id = ?1"
            ).unwrap();
            let items: Vec<BomItem> = stmt.query_map([id], |row| {
                Ok(BomItem {
                    id: row.get(0)?, bom_id: row.get(1)?, item_id: row.get(2)?,
                    item_name: row.get(3)?, item_code: row.get(4)?,
                    quantity: row.get(5)?, unit_cost: row.get(6)?,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "bom": bom, "items": items } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
    }
}

async fn create_bom(State(_state): State<AppState>, Json(form): Json<BomForm>) -> impl IntoResponse {
    if form.items.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one raw material is required." })));
    }
    if form.bom_name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "BOM name is required." })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    // ponytail: simple sequence via MAX, works for single-server; use a sequence table if multi-node
    let year = chrono::Utc::now().format("%Y");
    let max_seq: i64 = db.query_row(
        "SELECT COALESCE(MAX(CAST(SUBSTR(bom_no, 9) AS INTEGER)), 0) FROM boms WHERE bom_no LIKE ?1",
        [&format!("BOM-{}%-%", year)],
        |row| row.get(0),
    ).unwrap_or(0);
    let bom_no = format!("BOM-{}-{:04}", year, max_seq + 1);
    // ponytail: description stored but not displayed on detail page yet
    let description = form.description.as_deref().unwrap_or("");
    let result = db.execute(
        "INSERT INTO boms (bom_no, bom_name, finished_item_id, quantity, description) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![bom_no, form.bom_name, form.finished_item_id, form.quantity, description],
    );
    match result {
        Ok(_) => {
            let bom_id = db.last_insert_rowid();
            for item in &form.items {
                db.execute(
                    "INSERT INTO bom_items (bom_id, item_id, quantity, unit_cost) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![bom_id, item.item_id, item.quantity, item.unit_cost.unwrap_or(0.0)],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": bom_id, "bom_no": bom_no } })))
        }
        Err(e) => { tracing::error!("Failed to create BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create BOM." }))) }
    }
}

async fn update_bom(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<BomForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE boms SET bom_name=?1, finished_item_id=?2, quantity=?3, updated_at=datetime('now') WHERE id=?4",
        rusqlite::params![form.bom_name, form.finished_item_id, form.quantity, id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            db.execute("DELETE FROM bom_items WHERE bom_id = ?1", [id]).ok();
            for item in &form.items {
                db.execute("INSERT INTO bom_items (bom_id, item_id, quantity, unit_cost) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![id, item.item_id, item.quantity, item.unit_cost.unwrap_or(0.0)]).ok();
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "BOM updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
        Err(e) => { tracing::error!("Failed to update BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update BOM." }))) }
    }
}

async fn delete_bom(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    db.execute("DELETE FROM bom_items WHERE bom_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM boms WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "BOM deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
        Err(e) => { tracing::error!("Failed to delete BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete BOM." }))) }
    }
}

async fn toggle_bom_active(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE boms SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END, updated_at = datetime('now') WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "BOM status toggled." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
        Err(e) => { tracing::error!("Failed to toggle BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to toggle BOM." }))) }
    }
}

async fn get_bom_by_item(State(_state): State<AppState>, Path(item_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, bom_no, bom_name, finished_item_id, quantity, is_active, created_at, updated_at
         FROM boms WHERE finished_item_id = ?1 AND is_active = 1 LIMIT 1",
        [item_id],
        |row| Ok(json!({
            "id": row.get::<_, i64>(0)?, "bom_no": row.get::<_, String>(1)?,
            "bom_name": row.get::<_, String>(2)?, "finished_item_id": row.get::<_, i64>(3)?,
            "quantity": row.get::<_, f64>(4)?, "is_active": row.get::<_, i64>(5)? != 0,
            "created_at": row.get::<_, String>(6)?, "updated_at": row.get::<_, String>(7)?,
        })),
    );
    match result {
        Ok(bom) => (StatusCode::OK, Json(json!({ "success": true, "data": bom }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "No active BOM found for this item." }))),
    }
}

// ============================================================================
// Productions
// ============================================================================

async fn list_productions(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT p.id, p.production_no, p.output_item_id, i.item_name, i.item_code,
                p.output_quantity, p.completed_qty, p.end_date,
                p.warehouse_id, w.warehouse_name, p.bom_id,
                b.bom_name, p.overhead_cost, p.batch_id, p.unit_cost, p.total_material_cost,
                p.status, p.notes, p.created_by, p.created_at
         FROM productions p
         LEFT JOIN items i ON p.output_item_id = i.id
         LEFT JOIN warehouses w ON p.warehouse_id = w.id
         LEFT JOIN boms b ON p.bom_id = b.id
         ORDER BY p.created_at DESC"
    ).unwrap();
    let items: Vec<Production> = stmt.query_map([], |row| {
        Ok(Production {
            id: row.get(0)?, production_no: row.get(1)?, output_item_id: row.get(2)?,
            output_item_name: row.get(3)?, output_item_code: row.get(4)?,
            output_quantity: row.get(5)?, completed_qty: row.get(6)?,
            end_date: row.get(7)?, warehouse_id: row.get(8)?, warehouse_name: row.get(9)?,
            bom_id: row.get(10)?, bom_name: row.get(11)?, overhead_cost: row.get(12)?,
            batch_id: row.get(13)?, unit_cost: row.get(14)?, total_material_cost: row.get(15)?,
            status: row.get(16)?, notes: row.get(17)?, created_by: row.get(18)?,
            created_at: row.get(19)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_production(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT p.id, p.production_no, p.output_item_id, i.item_name, i.item_code,
                p.output_quantity, p.completed_qty, p.end_date,
                p.warehouse_id, w.warehouse_name, p.bom_id,
                b.bom_name, p.overhead_cost, p.batch_id, p.unit_cost, p.total_material_cost,
                p.status, p.notes, p.created_by, p.created_at
         FROM productions p
         LEFT JOIN items i ON p.output_item_id = i.id
         LEFT JOIN warehouses w ON p.warehouse_id = w.id
         LEFT JOIN boms b ON p.bom_id = b.id
         WHERE p.id = ?1",
        [id],
        |row| Ok(Production {
            id: row.get(0)?, production_no: row.get(1)?, output_item_id: row.get(2)?,
            output_item_name: row.get(3)?, output_item_code: row.get(4)?,
            output_quantity: row.get(5)?, completed_qty: row.get(6)?,
            end_date: row.get(7)?, warehouse_id: row.get(8)?, warehouse_name: row.get(9)?,
            bom_id: row.get(10)?, bom_name: row.get(11)?, overhead_cost: row.get(12)?,
            batch_id: row.get(13)?, unit_cost: row.get(14)?, total_material_cost: row.get(15)?,
            status: row.get(16)?, notes: row.get(17)?, created_by: row.get(18)?,
            created_at: row.get(19)?,
        }),
    );
    match result {
        Ok(prod) => {
            let mut stmt = db.prepare(
                "SELECT pi.id, pi.production_id, pi.item_id, i.item_name, i.item_code,
                        pi.quantity, pi.warehouse_id
                 FROM production_inputs pi LEFT JOIN items i ON pi.item_id = i.id
                 WHERE pi.production_id = ?1"
            ).unwrap();
            let inputs: Vec<ProductionInput> = stmt.query_map([id], |row| {
                Ok(ProductionInput {
                    id: row.get(0)?, production_id: row.get(1)?, item_id: row.get(2)?,
                    item_name: row.get(3)?, item_code: row.get(4)?,
                    quantity: row.get(5)?, warehouse_id: row.get(6)?, unit_cost: None,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "production": prod, "inputs": inputs } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Production not found." }))),
    }
}

async fn create_production(State(_state): State<AppState>, Json(form): Json<ProductionForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM productions", [], |row| row.get(0)).unwrap_or(1);
    let pno = format!("PRD-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let result = db.execute(
        "INSERT INTO productions (production_no, output_item_id, output_quantity, warehouse_id, bom_id, overhead_cost, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![pno, form.output_item_id, form.output_quantity, form.warehouse_id,
            form.bom_id, form.overhead_cost.unwrap_or(0.0), form.notes.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => {
            let prod_id = db.last_insert_rowid();

            // Get output item unit cost
            let output_cost: f64 = db.query_row(
                "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                [form.output_item_id],
                |row| row.get(0),
            ).unwrap_or(0.0);

            // Create stock movement for output (IN)
            let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
            let mno_out = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
            if let Err(e) = db.execute(
                "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                 VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'PRODUCTION', ?6, ?7)",
                rusqlite::params![mno_out, form.output_item_id, form.warehouse_id, form.output_quantity, output_cost, pno, format!("Production Output {}", pno)],
            ) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to create output stock movement: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create output stock movement." })));
            }

            // Add output to stock
            let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                rusqlite::params![form.output_item_id, form.warehouse_id], |row| row.get(0)).unwrap_or(false);
            if exists {
                if let Err(e) = db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![form.output_quantity, form.output_item_id, form.warehouse_id]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update output stock balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update output stock balance." })));
                }
            } else {
                if let Err(e) = db.execute("INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                    rusqlite::params![form.output_item_id, form.warehouse_id, form.output_quantity]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to insert output stock balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create output stock balance." })));
                }
            }
            if let Err(e) = db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![form.output_quantity, form.output_item_id]) {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to update output item stock: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update output item stock." })));
            }

            // Record inputs and create stock movements
            for input in &form.inputs {
                if let Err(e) = db.execute(
                    "INSERT INTO production_inputs (production_id, item_id, quantity, warehouse_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![prod_id, input.item_id, input.quantity, input.warehouse_id],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create production input: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create production input." })));
                }

                // Get input item unit cost
                let input_cost: f64 = db.query_row(
                    "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1",
                    [input.item_id],
                    |row| row.get(0),
                ).unwrap_or(0.0);

                // Create stock movement for input (OUT)
                let mseq_in: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
                let mno_in = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq_in);
                if let Err(e) = db.execute(
                    "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
                     VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'PRODUCTION', ?6, ?7)",
                    rusqlite::params![mno_in, input.item_id, input.warehouse_id, input.quantity, input_cost, pno, format!("Production Input {}", pno)],
                ) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to create input stock movement: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create input stock movement." })));
                }

                // Deduct input stock
                if let Err(e) = db.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![input.quantity, input.item_id, input.warehouse_id]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update input stock balance: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update input stock balance." })));
                }
                if let Err(e) = db.execute("UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![input.quantity, input.item_id]) {
                    let _ = db.execute_batch("ROLLBACK");
                    tracing::error!("Failed to update input item stock: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update input item stock." })));
                }
            }
            if let Err(e) = db.execute_batch("COMMIT") {
                let _ = db.execute_batch("ROLLBACK");
                tracing::error!("Failed to commit production: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to commit transaction." })));
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": prod_id, "production_no": pno } })))
        }
        Err(e) => { let _ = db.execute_batch("ROLLBACK"); tracing::error!("Failed to create production: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create production." }))) }
    }
}

async fn update_production(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<ProductionForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE productions SET output_item_id=?1, output_quantity=?2, warehouse_id=?3, bom_id=?4,
         overhead_cost=?5, notes=?6 WHERE id=?7 AND status != 'Completed'",
        rusqlite::params![form.output_item_id, form.output_quantity, form.warehouse_id,
            form.bom_id, form.overhead_cost.unwrap_or(0.0), form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => {
            // Re-insert production inputs (delete old, insert new)
            db.execute("DELETE FROM production_inputs WHERE production_id = ?1", [id]).ok();
            for input in &form.inputs {
                db.execute(
                    "INSERT INTO production_inputs (production_id, item_id, quantity, warehouse_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![id, input.item_id, input.quantity, input.warehouse_id],
                ).ok();
            }
            (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Production updated." } })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Production not found or already completed." }))),
        Err(e) => { tracing::error!("Failed to update production: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update production." }))) }
    }
}

async fn delete_production(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    if let Err(e) = db.execute_batch("BEGIN IMMEDIATE") {
        tracing::error!("Failed to begin transaction: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to start transaction." })));
    }

    // 1. Get production record
    let (output_item_id, output_quantity, warehouse_id, production_no): (i64, f64, i64, String) = match db.query_row(
        "SELECT output_item_id, output_quantity, warehouse_id, production_no FROM productions WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ) {
        Ok(v) => v,
        Err(_) => { let _ = db.execute_batch("ROLLBACK"); return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Production not found." }))); }
    };

    // 2. Get production inputs
    let inputs: Vec<(i64, f64, i64)> = {
        let mut stmt = db.prepare("SELECT item_id, quantity, warehouse_id FROM production_inputs WHERE production_id = ?1").unwrap();
        stmt.query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap().filter_map(|r| r.ok()).collect()
    };

    // 3. Reverse output: create stock movement OUT (remove produced items)
    {
        let unit_cost: f64 = db.query_row(
            "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1", [output_item_id],
            |row| row.get(0),
        ).unwrap_or(0.0);
        let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
        let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
        if let Err(e) = db.execute(
            "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
             VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, 'PRODUCTION_DELETE', ?6, ?7)",
            rusqlite::params![mno, output_item_id, warehouse_id, output_quantity, unit_cost, production_no, format!("Production deleted {}", production_no)],
        ) {
            let _ = db.execute_batch("ROLLBACK");
            tracing::error!("Failed to create output reversal movement: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete production (transaction rolled back)." })));
        }
        let _ = db.execute(
            "UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
            rusqlite::params![output_quantity, output_item_id, warehouse_id],
        );
        let _ = db.execute(
            "UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![output_quantity, output_item_id],
        );
    }

    // 4. Reverse inputs: create stock movement IN for each (restore consumed stock)
    for (item_id, quantity, wh_id) in &inputs {
        let unit_cost: f64 = db.query_row(
            "SELECT COALESCE(standard_cost, 0) FROM items WHERE id = ?1", [*item_id],
            |row| row.get(0),
        ).unwrap_or(0.0);
        let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
        let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);
        let _ = db.execute(
            "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
             VALUES (?1, ?2, ?3, 'IN', ?4, ?5, 'PRODUCTION_DELETE', ?6, ?7)",
            rusqlite::params![mno, item_id, wh_id, quantity, unit_cost, production_no, format!("Input restored {}", production_no)],
        );
        let _ = db.execute(
            "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
            rusqlite::params![quantity, item_id, wh_id],
        );
        let _ = db.execute(
            "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![quantity, item_id],
        );
    }

    // 5. Delete records
    let _ = db.execute("DELETE FROM production_inputs WHERE production_id = ?1", [id]);
    let _ = db.execute("DELETE FROM productions WHERE id = ?1", [id]);

    if let Err(e) = db.execute_batch("COMMIT") {
        let _ = db.execute_batch("ROLLBACK");
        tracing::error!("Failed to commit production deletion: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete production (transaction rolled back)." })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Production deleted and stock reversed.", "inputs_restored": inputs.len() } })))
}

async fn production_summary_by_item(State(_state): State<AppState>, Path(item_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let total: f64 = db.query_row("SELECT COALESCE(SUM(output_quantity), 0) FROM productions WHERE output_item_id = ?1", [item_id], |row| row.get(0)).unwrap_or(0.0);
    let count: i64 = db.query_row("SELECT COUNT(*) FROM productions WHERE output_item_id = ?1", [item_id], |row| row.get(0)).unwrap_or(0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "total_produced": total, "production_count": count } })))
}
