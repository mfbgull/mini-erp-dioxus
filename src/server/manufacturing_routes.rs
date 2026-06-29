use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put, patch},
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
        .route("/api/production/productions/{id}", get(get_production).delete(delete_production))
        .route("/api/production/productions/summary/item/{id}", get(production_summary_by_item))
}

// ============================================================================
// BOM
// ============================================================================

async fn list_boms(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT b.id, b.bom_no, b.bom_name, b.finished_item_id, i.item_name, i.item_code,
                b.quantity, b.is_active, b.created_at, b.updated_at
         FROM boms b LEFT JOIN items i ON b.finished_item_id = i.id
         ORDER BY b.created_at DESC"
    ).unwrap();
    let items: Vec<Bom> = stmt.query_map([], |row| {
        Ok(Bom {
            id: row.get(0)?, bom_no: row.get(1)?, bom_name: row.get(2)?,
            finished_item_id: row.get(3)?, finished_item_name: row.get(4)?,
            finished_item_code: row.get(5)?, quantity: row.get(6)?,
            is_active: row.get::<_, i64>(7)? != 0, created_at: row.get(8)?, updated_at: row.get(9)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_bom(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT b.id, b.bom_no, b.bom_name, b.finished_item_id, i.item_name, i.item_code,
                b.quantity, b.is_active, b.created_at, b.updated_at
         FROM boms b LEFT JOIN items i ON b.finished_item_id = i.id WHERE b.id = ?1",
        [id],
        |row| Ok(Bom {
            id: row.get(0)?, bom_no: row.get(1)?, bom_name: row.get(2)?,
            finished_item_id: row.get(3)?, finished_item_name: row.get(4)?,
            finished_item_code: row.get(5)?, quantity: row.get(6)?,
            is_active: row.get::<_, i64>(7)? != 0, created_at: row.get(8)?, updated_at: row.get(9)?,
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
    let db = db::get_db().lock().unwrap();
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM boms", [], |row| row.get(0)).unwrap_or(1);
    let bom_no = format!("BOM-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let result = db.execute(
        "INSERT INTO boms (bom_no, bom_name, finished_item_id, quantity) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![bom_no, form.bom_name, form.finished_item_id, form.quantity],
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    db.execute("DELETE FROM bom_items WHERE bom_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM boms WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "BOM deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
        Err(e) => { tracing::error!("Failed to delete BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete BOM." }))) }
    }
}

async fn toggle_bom_active(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE boms SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END, updated_at = datetime('now') WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "BOM status toggled." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "BOM not found." }))),
        Err(e) => { tracing::error!("Failed to toggle BOM: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to toggle BOM." }))) }
    }
}

async fn get_bom_by_item(State(_state): State<AppState>, Path(item_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT p.id, p.production_no, p.output_item_id, i.item_name, i.item_code,
                p.output_quantity, p.warehouse_id, w.warehouse_name, p.bom_id,
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
            output_quantity: row.get(5)?, warehouse_id: row.get(6)?, warehouse_name: row.get(7)?,
            bom_id: row.get(8)?, bom_name: row.get(9)?, overhead_cost: row.get(10)?,
            batch_id: row.get(11)?, unit_cost: row.get(12)?, total_material_cost: row.get(13)?,
            status: row.get(14)?, notes: row.get(15)?, created_by: row.get(16)?,
            created_at: row.get(17)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_production(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT p.id, p.production_no, p.output_item_id, i.item_name, i.item_code,
                p.output_quantity, p.warehouse_id, w.warehouse_name, p.bom_id,
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
            output_quantity: row.get(5)?, warehouse_id: row.get(6)?, warehouse_name: row.get(7)?,
            bom_id: row.get(8)?, bom_name: row.get(9)?, overhead_cost: row.get(10)?,
            batch_id: row.get(11)?, unit_cost: row.get(12)?, total_material_cost: row.get(13)?,
            status: row.get(14)?, notes: row.get(15)?, created_by: row.get(16)?,
            created_at: row.get(17)?,
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
    let db = db::get_db().lock().unwrap();
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
            // Add output to stock
            let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                rusqlite::params![form.output_item_id, form.warehouse_id], |row| row.get(0)).unwrap_or(false);
            if exists {
                db.execute("UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![form.output_quantity, form.output_item_id, form.warehouse_id]).ok();
            } else {
                db.execute("INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                    rusqlite::params![form.output_item_id, form.warehouse_id, form.output_quantity]).ok();
            }
            db.execute("UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![form.output_quantity, form.output_item_id]).ok();
            // Record inputs
            for input in &form.inputs {
                db.execute(
                    "INSERT INTO production_inputs (production_id, item_id, quantity, warehouse_id) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![prod_id, input.item_id, input.quantity, input.warehouse_id],
                ).ok();
                // Deduct input stock
                db.execute("UPDATE stock_balances SET quantity = quantity - ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![input.quantity, input.item_id, input.warehouse_id]).ok();
                db.execute("UPDATE items SET current_stock = current_stock - ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![input.quantity, input.item_id]).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": prod_id, "production_no": pno } })))
        }
        Err(e) => { tracing::error!("Failed to create production: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create production." }))) }
    }
}

async fn delete_production(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    db.execute("DELETE FROM production_inputs WHERE production_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM productions WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Production deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Production not found." }))),
        Err(e) => { tracing::error!("Failed to delete production: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete production." }))) }
    }
}

async fn production_summary_by_item(State(_state): State<AppState>, Path(item_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let total: f64 = db.query_row("SELECT COALESCE(SUM(output_quantity), 0) FROM productions WHERE output_item_id = ?1", [item_id], |row| row.get(0)).unwrap_or(0.0);
    let count: i64 = db.query_row("SELECT COUNT(*) FROM productions WHERE output_item_id = ?1", [item_id], |row| row.get(0)).unwrap_or(0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "total_produced": total, "production_count": count } })))
}
