//! Inventory route handlers for the MiniERP API server.
//!
//! Endpoints:
//! - Items: GET/POST /api/inventory/items, GET/PUT/DELETE /api/inventory/items/:id
//! - Warehouses: GET/POST /api/inventory/warehouses, GET/PUT/DELETE /api/inventory/warehouses/:id
//! - Stock Movements: GET/POST /api/inventory/stock-movements
//! - Stock Balances: GET /api/inventory/stock-balances
//! - Physical Counts: GET/POST /api/inventory/physical-counts, etc.

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

// ============================================================================
// Router
// ============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // Items
        .route("/api/inventory/items", get(list_items).post(create_item))
        .route("/api/inventory/items/{id}", get(get_item).put(update_item).delete(delete_item))
        .route("/api/inventory/items-categories", get(list_categories))
        .route("/api/inventory/items-low-stock", get(list_low_stock))
        .route("/api/inventory/items-uom", get(list_uom))
        // Warehouses
        .route("/api/inventory/warehouses", get(list_warehouses).post(create_warehouse))
        .route("/api/inventory/warehouses/{id}", get(get_warehouse).put(update_warehouse).delete(delete_warehouse))
        // Stock Movements
        .route("/api/inventory/stock-movements", get(list_stock_movements).post(create_stock_movement))
        .route("/api/inventory/stock-balances", get(list_stock_balances))
        .route("/api/inventory/stock-summary", get(stock_summary))
        // Physical Counts
        .route("/api/inventory/physical-counts", get(list_physical_counts).post(create_physical_count))
        .route("/api/inventory/physical-counts/{id}", get(get_physical_count))
        .route("/api/inventory/physical-counts/{id}/items", post(add_count_item))
        .route("/api/inventory/physical-counts/{id}/complete", post(complete_physical_count))
        .route("/api/inventory/physical-counts/{id}/cancel", post(cancel_physical_count))
}

// ============================================================================
// Items
// ============================================================================

async fn list_items(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, item_code, item_name, description, category, unit_of_measure,
                    current_stock, reorder_level, standard_cost, selling_price,
                    is_raw_material, is_finished_good, is_purchased, is_manufactured,
                    is_active, created_at, updated_at
             FROM items WHERE is_active = 1 ORDER BY item_code",
        )
        .unwrap();

    let items: Vec<Item> = stmt
        .query_map([], |row| {
            Ok(Item {
                id: row.get(0)?,
                item_code: row.get(1)?,
                item_name: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                unit_of_measure: row.get(5)?,
                current_stock: row.get(6)?,
                reorder_level: row.get(7)?,
                standard_cost: row.get(8)?,
                selling_price: row.get(9)?,
                is_raw_material: row.get::<_, i64>(10)? != 0,
                is_finished_good: row.get::<_, i64>(11)? != 0,
                is_purchased: row.get::<_, i64>(12)? != 0,
                is_manufactured: row.get::<_, i64>(13)? != 0,
                is_active: row.get::<_, i64>(14)? != 0,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_item(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, item_code, item_name, description, category, unit_of_measure,
                current_stock, reorder_level, standard_cost, selling_price,
                is_raw_material, is_finished_good, is_purchased, is_manufactured,
                is_active, created_at, updated_at
         FROM items WHERE id = ?1",
        [id],
        |row| {
            Ok(Item {
                id: row.get(0)?,
                item_code: row.get(1)?,
                item_name: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                unit_of_measure: row.get(5)?,
                current_stock: row.get(6)?,
                reorder_level: row.get(7)?,
                standard_cost: row.get(8)?,
                selling_price: row.get(9)?,
                is_raw_material: row.get::<_, i64>(10)? != 0,
                is_finished_good: row.get::<_, i64>(11)? != 0,
                is_purchased: row.get::<_, i64>(12)? != 0,
                is_manufactured: row.get::<_, i64>(13)? != 0,
                is_active: row.get::<_, i64>(14)? != 0,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        },
    );

    match result {
        Ok(item) => (StatusCode::OK, Json(json!({ "success": true, "data": item }))),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Item not found." })),
        ),
    }
}

async fn create_item(
    State(_state): State<AppState>,
    Json(form): Json<ItemForm>,
) -> impl IntoResponse {
    if form.item_code.trim().is_empty() || form.item_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Item code and name are required." })),
        );
    }

    let db = db::get_db().lock().unwrap();

    // Check for duplicate code
    let exists: bool = db
        .query_row(
            "SELECT COUNT(*) > 0 FROM items WHERE item_code = ?1",
            [&form.item_code],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "success": false, "error": "Item code already exists." })),
        );
    }

    let result = db.execute(
        "INSERT INTO items (item_code, item_name, description, category, unit_of_measure,
            reorder_level, standard_cost, selling_price,
            is_raw_material, is_finished_good, is_purchased, is_manufactured)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            form.item_code,
            form.item_name,
            form.description.as_deref().unwrap_or(""),
            form.category.as_deref().unwrap_or(""),
            form.unit_of_measure.as_deref().unwrap_or("pcs"),
            form.reorder_level.unwrap_or(0.0),
            form.standard_cost.unwrap_or(0.0),
            form.selling_price.unwrap_or(0.0),
            form.is_raw_material.unwrap_or(false) as i64,
            form.is_finished_good.unwrap_or(false) as i64,
            form.is_purchased.unwrap_or(true) as i64,
            form.is_manufactured.unwrap_or(false) as i64,
        ],
    );

    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            let item = db.query_row(
                "SELECT id, item_code, item_name, description, category, unit_of_measure,
                        current_stock, reorder_level, standard_cost, selling_price,
                        is_raw_material, is_finished_good, is_purchased, is_manufactured,
                        is_active, created_at, updated_at
                 FROM items WHERE id = ?1",
                [id],
                |row| {
                    Ok(Item {
                        id: row.get(0)?,
                        item_code: row.get(1)?,
                        item_name: row.get(2)?,
                        description: row.get(3)?,
                        category: row.get(4)?,
                        unit_of_measure: row.get(5)?,
                        current_stock: row.get(6)?,
                        reorder_level: row.get(7)?,
                        standard_cost: row.get(8)?,
                        selling_price: row.get(9)?,
                        is_raw_material: row.get::<_, i64>(10)? != 0,
                        is_finished_good: row.get::<_, i64>(11)? != 0,
                        is_purchased: row.get::<_, i64>(12)? != 0,
                        is_manufactured: row.get::<_, i64>(13)? != 0,
                        is_active: row.get::<_, i64>(14)? != 0,
                        created_at: row.get(15)?,
                        updated_at: row.get(16)?,
                    })
                },
            ).unwrap();

            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "data": item })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create item: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to create item." })),
            )
        }
    }
}

async fn update_item(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
    Json(form): Json<ItemForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let result = db.execute(
        "UPDATE items SET
            item_code = ?1, item_name = ?2, description = ?3, category = ?4,
            unit_of_measure = ?5, reorder_level = ?6, standard_cost = ?7,
            selling_price = ?8, is_raw_material = ?9, is_finished_good = ?10,
            is_purchased = ?11, is_manufactured = ?12, updated_at = datetime('now')
         WHERE id = ?13",
        rusqlite::params![
            form.item_code,
            form.item_name,
            form.description.as_deref().unwrap_or(""),
            form.category.as_deref().unwrap_or(""),
            form.unit_of_measure.as_deref().unwrap_or("pcs"),
            form.reorder_level.unwrap_or(0.0),
            form.standard_cost.unwrap_or(0.0),
            form.selling_price.unwrap_or(0.0),
            form.is_raw_material.unwrap_or(false) as i64,
            form.is_finished_good.unwrap_or(false) as i64,
            form.is_purchased.unwrap_or(true) as i64,
            form.is_manufactured.unwrap_or(false) as i64,
            id,
        ],
    );

    match result {
        Ok(rows) if rows > 0 => {
            let item = db.query_row(
                "SELECT id, item_code, item_name, description, category, unit_of_measure,
                        current_stock, reorder_level, standard_cost, selling_price,
                        is_raw_material, is_finished_good, is_purchased, is_manufactured,
                        is_active, created_at, updated_at
                 FROM items WHERE id = ?1",
                [id],
                |row| {
                    Ok(Item {
                        id: row.get(0)?,
                        item_code: row.get(1)?,
                        item_name: row.get(2)?,
                        description: row.get(3)?,
                        category: row.get(4)?,
                        unit_of_measure: row.get(5)?,
                        current_stock: row.get(6)?,
                        reorder_level: row.get(7)?,
                        standard_cost: row.get(8)?,
                        selling_price: row.get(9)?,
                        is_raw_material: row.get::<_, i64>(10)? != 0,
                        is_finished_good: row.get::<_, i64>(11)? != 0,
                        is_purchased: row.get::<_, i64>(12)? != 0,
                        is_manufactured: row.get::<_, i64>(13)? != 0,
                        is_active: row.get::<_, i64>(14)? != 0,
                        created_at: row.get(15)?,
                        updated_at: row.get(16)?,
                    })
                },
            ).unwrap();

            (
                StatusCode::OK,
                Json(json!({ "success": true, "data": item })),
            )
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Item not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to update item: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to update item." })),
            )
        }
    }
}

async fn delete_item(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "UPDATE items SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
        [id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Item deleted." } })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Item not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to delete item: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to delete item." })),
            )
        }
    }
}

async fn list_categories(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare("SELECT DISTINCT category FROM items WHERE is_active = 1 AND category != '' ORDER BY category")
        .unwrap();

    let categories: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": categories })))
}

async fn list_low_stock(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, item_code, item_name, description, category, unit_of_measure,
                    current_stock, reorder_level, standard_cost, selling_price,
                    is_raw_material, is_finished_good, is_purchased, is_manufactured,
                    is_active, created_at, updated_at
             FROM items WHERE is_active = 1 AND current_stock <= reorder_level
             ORDER BY (current_stock - reorder_level) ASC",
        )
        .unwrap();

    let items: Vec<Item> = stmt
        .query_map([], |row| {
            Ok(Item {
                id: row.get(0)?,
                item_code: row.get(1)?,
                item_name: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                unit_of_measure: row.get(5)?,
                current_stock: row.get(6)?,
                reorder_level: row.get(7)?,
                standard_cost: row.get(8)?,
                selling_price: row.get(9)?,
                is_raw_material: row.get::<_, i64>(10)? != 0,
                is_finished_good: row.get::<_, i64>(11)? != 0,
                is_purchased: row.get::<_, i64>(12)? != 0,
                is_manufactured: row.get::<_, i64>(13)? != 0,
                is_active: row.get::<_, i64>(14)? != 0,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_uom(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare("SELECT DISTINCT unit_of_measure FROM items WHERE is_active = 1 ORDER BY unit_of_measure")
        .unwrap();

    let uom: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": uom })))
}

// ============================================================================
// Warehouses
// ============================================================================

async fn list_warehouses(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT id, warehouse_code, warehouse_name, location, is_active, created_at
             FROM warehouses WHERE is_active = 1 ORDER BY warehouse_code",
        )
        .unwrap();

    let warehouses: Vec<Warehouse> = stmt
        .query_map([], |row| {
            Ok(Warehouse {
                id: row.get(0)?,
                warehouse_code: row.get(1)?,
                warehouse_name: row.get(2)?,
                location: row.get(3)?,
                is_active: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": warehouses })))
}

async fn get_warehouse(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, warehouse_code, warehouse_name, location, is_active, created_at
         FROM warehouses WHERE id = ?1",
        [id],
        |row| {
            Ok(Warehouse {
                id: row.get(0)?,
                warehouse_code: row.get(1)?,
                warehouse_name: row.get(2)?,
                location: row.get(3)?,
                is_active: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        },
    );

    match result {
        Ok(wh) => (StatusCode::OK, Json(json!({ "success": true, "data": wh }))),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Warehouse not found." })),
        ),
    }
}

async fn create_warehouse(
    State(_state): State<AppState>,
    Json(form): Json<WarehouseForm>,
) -> impl IntoResponse {
    if form.warehouse_code.trim().is_empty() || form.warehouse_name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Warehouse code and name are required." })),
        );
    }

    let db = db::get_db().lock().unwrap();

    let exists: bool = db
        .query_row(
            "SELECT COUNT(*) > 0 FROM warehouses WHERE warehouse_code = ?1",
            [&form.warehouse_code],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "success": false, "error": "Warehouse code already exists." })),
        );
    }

    let result = db.execute(
        "INSERT INTO warehouses (warehouse_code, warehouse_name, location)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![
            form.warehouse_code,
            form.warehouse_name,
            form.location.as_deref().unwrap_or(""),
        ],
    );

    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            let wh = db.query_row(
                "SELECT id, warehouse_code, warehouse_name, location, is_active, created_at
                 FROM warehouses WHERE id = ?1",
                [id],
                |row| {
                    Ok(Warehouse {
                        id: row.get(0)?,
                        warehouse_code: row.get(1)?,
                        warehouse_name: row.get(2)?,
                        location: row.get(3)?,
                        is_active: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                },
            ).unwrap();

            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "data": wh })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create warehouse: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to create warehouse." })),
            )
        }
    }
}

async fn update_warehouse(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
    Json(form): Json<WarehouseForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "UPDATE warehouses SET warehouse_code = ?1, warehouse_name = ?2, location = ?3
         WHERE id = ?4",
        rusqlite::params![form.warehouse_code, form.warehouse_name, form.location.as_deref().unwrap_or(""), id],
    );

    match result {
        Ok(rows) if rows > 0 => {
            let wh = db.query_row(
                "SELECT id, warehouse_code, warehouse_name, location, is_active, created_at
                 FROM warehouses WHERE id = ?1",
                [id],
                |row| {
                    Ok(Warehouse {
                        id: row.get(0)?,
                        warehouse_code: row.get(1)?,
                        warehouse_name: row.get(2)?,
                        location: row.get(3)?,
                        is_active: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                },
            ).unwrap();

            (StatusCode::OK, Json(json!({ "success": true, "data": wh })))
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Warehouse not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to update warehouse: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to update warehouse." })),
            )
        }
    }
}

async fn delete_warehouse(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "UPDATE warehouses SET is_active = 0 WHERE id = ?1",
        [id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Warehouse deleted." } })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Warehouse not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to delete warehouse: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to delete warehouse." })),
            )
        }
    }
}

// ============================================================================
// Stock Movements
// ============================================================================

async fn list_stock_movements(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT sm.id, sm.movement_no, sm.item_id, i.item_name, i.item_code,
                    sm.warehouse_id, w.warehouse_name, sm.movement_type, sm.quantity,
                    sm.unit_cost, sm.reference_doctype, sm.reference_docno,
                    sm.batch_id, sm.notes, sm.created_by, sm.created_at
             FROM stock_movements sm
             LEFT JOIN items i ON sm.item_id = i.id
             LEFT JOIN warehouses w ON sm.warehouse_id = w.id
             ORDER BY sm.created_at DESC
             LIMIT 100",
        )
        .unwrap();

    let movements: Vec<StockMovement> = stmt
        .query_map([], |row| {
            Ok(StockMovement {
                id: row.get(0)?,
                movement_no: row.get(1)?,
                item_id: row.get(2)?,
                item_name: row.get(3)?,
                item_code: row.get(4)?,
                warehouse_id: row.get(5)?,
                warehouse_name: row.get(6)?,
                movement_type: row.get(7)?,
                quantity: row.get(8)?,
                unit_cost: row.get(9)?,
                reference_doctype: row.get(10)?,
                reference_docno: row.get(11)?,
                batch_id: row.get(12)?,
                notes: row.get(13)?,
                created_by: row.get(14)?,
                created_at: row.get(15)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": movements })))
}

async fn create_stock_movement(
    State(_state): State<AppState>,
    Json(form): Json<StockMovementForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    // Generate movement number
    let seq: i64 = db
        .query_row(
            "SELECT COUNT(*) + 1 FROM stock_movements",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    let movement_no = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    let result = db.execute(
        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type,
            quantity, unit_cost, reference_doctype, reference_docno, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            movement_no,
            form.item_id,
            form.warehouse_id,
            form.movement_type,
            form.quantity,
            form.unit_cost.unwrap_or(0.0),
            form.reference_doctype,
            form.reference_docno,
            form.notes.as_deref().unwrap_or(""),
        ],
    );

    match result {
        Ok(_) => {
            // Update stock_balances
            let exists: bool = db
                .query_row(
                    "SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                    rusqlite::params![form.item_id, form.warehouse_id],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if exists {
                let delta = if form.movement_type == "IN" {
                    form.quantity
                } else {
                    -form.quantity
                };
                db.execute(
                    "UPDATE stock_balances SET quantity = quantity + ?1
                     WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![delta, form.item_id, form.warehouse_id],
                )
                .ok();
            } else if form.movement_type == "IN" {
                db.execute(
                    "INSERT INTO stock_balances (item_id, warehouse_id, quantity)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![form.item_id, form.warehouse_id, form.quantity],
                )
                .ok();
            }

            // Update item current_stock
            let total_balance: f64 = db
                .query_row(
                    "SELECT COALESCE(SUM(quantity), 0) FROM stock_balances WHERE item_id = ?1",
                    [form.item_id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            db.execute(
                "UPDATE items SET current_stock = ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![total_balance, form.item_id],
            )
            .ok();

            let id = db.last_insert_rowid();
            let movement = db.query_row(
                "SELECT sm.id, sm.movement_no, sm.item_id, i.item_name, i.item_code,
                        sm.warehouse_id, w.warehouse_name, sm.movement_type, sm.quantity,
                        sm.unit_cost, sm.reference_doctype, sm.reference_docno,
                        sm.batch_id, sm.notes, sm.created_by, sm.created_at
                 FROM stock_movements sm
                 LEFT JOIN items i ON sm.item_id = i.id
                 LEFT JOIN warehouses w ON sm.warehouse_id = w.id
                 WHERE sm.id = ?1",
                [id],
                |row| {
                    Ok(StockMovement {
                        id: row.get(0)?,
                        movement_no: row.get(1)?,
                        item_id: row.get(2)?,
                        item_name: row.get(3)?,
                        item_code: row.get(4)?,
                        warehouse_id: row.get(5)?,
                        warehouse_name: row.get(6)?,
                        movement_type: row.get(7)?,
                        quantity: row.get(8)?,
                        unit_cost: row.get(9)?,
                        reference_doctype: row.get(10)?,
                        reference_docno: row.get(11)?,
                        batch_id: row.get(12)?,
                        notes: row.get(13)?,
                        created_by: row.get(14)?,
                        created_at: row.get(15)?,
                    })
                },
            )
            .unwrap();

            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "data": movement })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create stock movement: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to create stock movement." })),
            )
        }
    }
}

async fn list_stock_balances(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT sb.id, sb.item_id, i.item_name, i.item_code,
                    sb.warehouse_id, w.warehouse_name, sb.quantity
             FROM stock_balances sb
             LEFT JOIN items i ON sb.item_id = i.id
             LEFT JOIN warehouses w ON sb.warehouse_id = w.id
             WHERE sb.quantity > 0
             ORDER BY i.item_code, w.warehouse_code",
        )
        .unwrap();

    let balances: Vec<StockBalance> = stmt
        .query_map([], |row| {
            Ok(StockBalance {
                id: row.get(0)?,
                item_id: row.get(1)?,
                item_name: row.get(2)?,
                item_code: row.get(3)?,
                warehouse_id: row.get(4)?,
                warehouse_name: row.get(5)?,
                quantity: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": balances })))
}

async fn stock_summary(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let total_items: i64 = db
        .query_row("SELECT COUNT(*) FROM items WHERE is_active = 1", [], |row| row.get(0))
        .unwrap_or(0);

    let total_stock_value: f64 = db
        .query_row(
            "SELECT COALESCE(SUM(current_stock * standard_cost), 0) FROM items WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let low_stock_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM items WHERE is_active = 1 AND current_stock <= reorder_level",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let warehouse_count: i64 = db
        .query_row("SELECT COUNT(*) FROM warehouses WHERE is_active = 1", [], |row| row.get(0))
        .unwrap_or(0);

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "total_items": total_items,
            "total_stock_value": total_stock_value,
            "low_stock_count": low_stock_count,
            "warehouse_count": warehouse_count,
        }
    })))
}

// ============================================================================
// Physical Counts
// ============================================================================

async fn list_physical_counts(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db
        .prepare(
            "SELECT pc.id, pc.count_no, pc.count_date, pc.warehouse_id,
                    w.warehouse_name, pc.status, pc.notes, pc.created_by,
                    pc.created_at, pc.completed_at
             FROM physical_counts pc
             LEFT JOIN warehouses w ON pc.warehouse_id = w.id
             ORDER BY pc.created_at DESC",
        )
        .unwrap();

    let counts: Vec<PhysicalCount> = stmt
        .query_map([], |row| {
            Ok(PhysicalCount {
                id: row.get(0)?,
                count_no: row.get(1)?,
                count_date: row.get(2)?,
                warehouse_id: row.get(3)?,
                warehouse_name: row.get(4)?,
                status: row.get(5)?,
                notes: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": counts })))
}

async fn get_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT pc.id, pc.count_no, pc.count_date, pc.warehouse_id,
                w.warehouse_name, pc.status, pc.notes, pc.created_by,
                pc.created_at, pc.completed_at
         FROM physical_counts pc
         LEFT JOIN warehouses w ON pc.warehouse_id = w.id
         WHERE pc.id = ?1",
        [id],
        |row| {
            Ok(PhysicalCount {
                id: row.get(0)?,
                count_no: row.get(1)?,
                count_date: row.get(2)?,
                warehouse_id: row.get(3)?,
                warehouse_name: row.get(4)?,
                status: row.get(5)?,
                notes: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        },
    );

    match result {
        Ok(count) => {
            // Get count items
            let mut stmt = db
                .prepare(
                    "SELECT pci.id, pci.count_id, pci.item_id, i.item_name, i.item_code,
                            pci.system_quantity, pci.counted_quantity, pci.variance
                     FROM physical_count_items pci
                     LEFT JOIN items i ON pci.item_id = i.id
                     WHERE pci.count_id = ?1
                     ORDER BY i.item_code",
                )
                .unwrap();

            let items: Vec<PhysicalCountItem> = stmt
                .query_map([id], |row| {
                    Ok(PhysicalCountItem {
                        id: row.get(0)?,
                        count_id: row.get(1)?,
                        item_id: row.get(2)?,
                        item_name: row.get(3)?,
                        item_code: row.get(4)?,
                        system_quantity: row.get(5)?,
                        counted_quantity: row.get(6)?,
                        variance: row.get(7)?,
                    })
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            (
                StatusCode::OK,
                Json(json!({ "success": true, "data": { "count": count, "items": items } })),
            )
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Physical count not found." })),
        ),
    }
}

async fn create_physical_count(
    State(_state): State<AppState>,
    Json(form): Json<PhysicalCountForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let seq: i64 = db
        .query_row("SELECT COUNT(*) + 1 FROM physical_counts", [], |row| row.get(0))
        .unwrap_or(1);
    let count_no = format!("PC-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    let result = db.execute(
        "INSERT INTO physical_counts (count_no, warehouse_id, notes, status)
         VALUES (?1, ?2, ?3, 'Draft')",
        rusqlite::params![count_no, form.warehouse_id, form.notes.as_deref().unwrap_or("")],
    );

    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();

            // Pre-populate count items from stock_balances
            let mut stmt = db
                .prepare(
                    "INSERT INTO physical_count_items (count_id, item_id, system_quantity)
                     SELECT ?1, sb.item_id, sb.quantity
                     FROM stock_balances sb
                     WHERE sb.warehouse_id = ?2 AND sb.quantity > 0",
                )
                .unwrap();
            stmt.execute(rusqlite::params![id, form.warehouse_id]).ok();

            let count = db.query_row(
                "SELECT pc.id, pc.count_no, pc.count_date, pc.warehouse_id,
                        w.warehouse_name, pc.status, pc.notes, pc.created_by,
                        pc.created_at, pc.completed_at
                 FROM physical_counts pc
                 LEFT JOIN warehouses w ON pc.warehouse_id = w.id
                 WHERE pc.id = ?1",
                [id],
                |row| {
                    Ok(PhysicalCount {
                        id: row.get(0)?,
                        count_no: row.get(1)?,
                        count_date: row.get(2)?,
                        warehouse_id: row.get(3)?,
                        warehouse_name: row.get(4)?,
                        status: row.get(5)?,
                        notes: row.get(6)?,
                        created_by: row.get(7)?,
                        created_at: row.get(8)?,
                        completed_at: row.get(9)?,
                    })
                },
            )
            .unwrap();

            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "data": count })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create physical count: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to create physical count." })),
            )
        }
    }
}

async fn add_count_item(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
    Json(form): Json<PhysicalCountItemForm>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let result = db.execute(
        "UPDATE physical_count_items
         SET counted_quantity = ?1, variance = ?1 - system_quantity
         WHERE count_id = ?2 AND item_id = ?3",
        rusqlite::params![form.counted_quantity, id, form.item_id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Count item recorded." } })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Count item not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to add count item: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to record count item." })),
            )
        }
    }
}

async fn complete_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let result = db.execute(
        "UPDATE physical_counts SET status = 'Completed', completed_at = datetime('now')
         WHERE id = ?1 AND status = 'Draft'",
        [id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Physical count completed." } })),
        ),
        Ok(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Count not found or not in Draft status." })),
        ),
        Err(e) => {
            tracing::error!("Failed to complete physical count: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to complete count." })),
            )
        }
    }
}

async fn cancel_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();

    let result = db.execute(
        "UPDATE physical_counts SET status = 'Cancelled' WHERE id = ?1 AND status = 'Draft'",
        [id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Physical count cancelled." } })),
        ),
        Ok(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Count not found or not in Draft status." })),
        ),
        Err(e) => {
            tracing::error!("Failed to cancel physical count: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to cancel count." })),
            )
        }
    }
}
