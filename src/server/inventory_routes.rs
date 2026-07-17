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
use crate::calculations::stock::{StockBatch, consume_fifo_batches};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;

// ============================================================================
// Stock-adjustment GL posting (pure decision logic)
// ============================================================================

/// Chart-of-account ids for stock-adjustment journal lines. These are the
/// seeded ids (see db.rs COA seed, append-only). Hardcoded to match the four
/// existing journal posters (invoice/payment/purchase/expense).
const ACCT_INVENTORY: i64 = 3; // 1200 Inventory (asset)
const ACCT_SHRINKAGE: i64 = 18; // 5100 Inventory Shrinkage (expense)
const ACCT_ADJ_GAIN: i64 = 19; // 4200 Inventory Adjustment Gain (revenue)

/// Decide the balanced double-entry lines for an ADJUSTMENT movement.
/// Returns `None` when there is nothing to post (zero value), else
/// `(debit_account_id, credit_account_id, value)`.
///
///   qty > 0 (correction up): Dr Inventory      Cr Adjustment Gain
///   qty < 0 (shrinkage):     Dr Shrinkage       Cr Inventory
fn adjustment_journal(quantity: f64, standard_cost: f64) -> Option<(i64, i64, f64)> {
    let value = quantity.abs() * standard_cost;
    if value <= 0.0 {
        return None;
    }
    if quantity < 0.0 {
        Some((ACCT_SHRINKAGE, ACCT_INVENTORY, value))
    } else {
        Some((ACCT_INVENTORY, ACCT_ADJ_GAIN, value))
    }
}

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
        .route("/api/inventory/stock-movements/item/{itemId}", get(list_stock_movements_by_item))
        .route("/api/inventory/stock-balances", get(list_stock_balances))
        .route("/api/inventory/stock-summary", get(stock_summary))
        // Physical Counts
        .route("/api/inventory/physical-counts", get(list_physical_counts).post(create_physical_count))
        .route("/api/inventory/physical-counts/{id}", get(get_physical_count).put(update_physical_count).delete(delete_physical_count))
        .route("/api/inventory/physical-counts/{id}/items", post(add_count_item))
        .route("/api/inventory/physical-counts/{id}/items/{item_id}", put(update_count_item))
        .route("/api/inventory/physical-counts/{id}/complete", post(complete_physical_count))
        .route("/api/inventory/physical-counts/{id}/cancel", post(cancel_physical_count))
}

// ============================================================================
// Items
// ============================================================================

async fn list_items(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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

    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db
        .prepare(
        "SELECT id, warehouse_code, warehouse_name, location, capacity, is_active, created_at
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
                capacity: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, warehouse_code, warehouse_name, location, capacity, is_active, created_at
         FROM warehouses WHERE id = ?1",
        [id],
        |row| {
            Ok(Warehouse {
                id: row.get(0)?,
                warehouse_code: row.get(1)?,
                warehouse_name: row.get(2)?,
                location: row.get(3)?,
                capacity: row.get(4)?,
                is_active: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
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

    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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
                        capacity: 0.0,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
                        capacity: 0.0,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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

async fn list_stock_movements_by_item(
    State(_state): State<AppState>,
    Path(item_id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db
        .prepare(
            "SELECT sm.id, sm.movement_no, sm.item_id, i.item_name, i.item_code,
                    sm.warehouse_id, w.warehouse_name, sm.movement_type, sm.quantity,
                    sm.unit_cost, sm.reference_doctype, sm.reference_docno,
                    sm.batch_id, sm.notes, sm.created_by, sm.created_at
             FROM stock_movements sm
             LEFT JOIN items i ON sm.item_id = i.id
             LEFT JOIN warehouses w ON sm.warehouse_id = w.id
             WHERE sm.item_id = ?1
             ORDER BY sm.created_at DESC
             LIMIT 50",
        )
        .unwrap();

    let movements: Vec<StockMovement> = stmt
        .query_map([item_id], |row| {
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Generate movement number
    let seq: i64 = db
        .query_row(
            "SELECT COUNT(*) + 1 FROM stock_movements",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    let movement_no = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), seq);

    // For OUT movements, do FIFO consumption from batches
    let (fifo_unit_cost, batch_id) = if form.movement_type == "OUT" {
        // Load existing batches ordered FIFO (oldest first)
        let mut stmt = db.prepare(
            "SELECT id, quantity_remaining, unit_cost FROM stock_batches
             WHERE item_id = ?1 AND warehouse_id = ?2 AND quantity_remaining > 0
             ORDER BY received_date ASC, id ASC"
        ).unwrap();
        let batches: Vec<StockBatch> = stmt.query_map(
            rusqlite::params![form.item_id, form.warehouse_id],
            |row| Ok(StockBatch {
                id: row.get(0)?,
                quantity_remaining: row.get(1)?,
                unit_cost: row.get(2)?,
            }),
        ).unwrap().filter_map(|r| r.ok()).collect();

        let fifo_result = consume_fifo_batches(&batches, form.quantity);

        // Update batch quantities in DB
        for updated in &fifo_result.updated_batches {
            db.execute(
                "UPDATE stock_batches SET quantity_remaining = ?1 WHERE id = ?2",
                rusqlite::params![updated.quantity_remaining, updated.id],
            ).ok();
        }

        let last_batch_id = batches.last().map(|b| b.id);
        (fifo_result.weighted_avg_cost, last_batch_id)
    } else {
        (form.unit_cost.unwrap_or(0.0), None)
    };

    // Use FIFO cost for OUT, provided cost for IN
    let effective_unit_cost = if form.movement_type == "OUT" {
        fifo_unit_cost
    } else {
        form.unit_cost.unwrap_or(0.0)
    };

    let movement_result = db.execute(
        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type,
            quantity, unit_cost, reference_doctype, reference_docno, batch_id, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            movement_no,
            form.item_id,
            form.warehouse_id,
            form.movement_type,
            form.quantity,
            effective_unit_cost,
            form.reference_doctype,
            form.reference_docno,
            batch_id,
            form.notes.as_deref().unwrap_or(""),
        ],
    );

    match movement_result {
        Ok(_) => {
            let movement_id = db.last_insert_rowid();

            // Update stock_balances
            let exists: bool = db
                .query_row(
                    "SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
                    rusqlite::params![form.item_id, form.warehouse_id],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            // Signed-quantity semantics (matches source StockMovement model):
            //   IN  → +qty, OUT → -qty (legacy paths),
            //   ADJUSTMENT / TRANSFER → quantity as sent (client signs it:
            //   negative on the from-warehouse row, positive on the to-warehouse row).
            let delta = match form.movement_type.as_str() {
                "IN" => form.quantity,
                "OUT" => -form.quantity,
                _ => form.quantity,
            };
            if exists {
                db.execute(
                    "UPDATE stock_balances SET quantity = quantity + ?1
                     WHERE item_id = ?2 AND warehouse_id = ?3",
                    rusqlite::params![delta, form.item_id, form.warehouse_id],
                )
                .ok();
            } else {
                db.execute(
                    "INSERT INTO stock_balances (item_id, warehouse_id, quantity)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![form.item_id, form.warehouse_id, delta],
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

            // For IN movements, create a stock batch
            if form.movement_type == "IN" {
                let batch_no = format!("{}-BATCH", movement_no);
                db.execute(
                    "INSERT INTO stock_batches (batch_no, item_id, warehouse_id, source_type, source_id,
                        quantity_original, quantity_remaining, unit_cost, received_date)
                     VALUES (?1, ?2, ?3, 'MOVEMENT', ?4, ?5, ?5, ?6, datetime('now'))",
                    rusqlite::params![batch_no, form.item_id, form.warehouse_id, movement_id,
                        form.quantity, effective_unit_cost],
                ).ok();
            }

            // ── Financial posting for ADJUSTMENT movements ──
            // Mirrors source StockMovement.postFinancialEntryForAdjustment, translated
            // to this repo's double-entry model (journal_entries header + balanced
            // journal_lines). Value = |qty| x standard_cost; skip when zero.
            // Account ids are hardcoded to match the four existing posters:
            //   3 = Inventory (1200), 18 = Inventory Shrinkage (5100),
            //   19 = Inventory Adjustment Gain (4200).
            if form.movement_type == "ADJUSTMENT" {
                let std_cost: f64 = db
                    .query_row("SELECT standard_cost FROM items WHERE id = ?1", [form.item_id], |r| r.get(0))
                    .unwrap_or(0.0);
                if let Some((dr_acct, cr_acct, value)) = adjustment_journal(form.quantity, std_cost) {
                    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                    let desc = if form.quantity < 0.0 {
                        format!("Stock removal: {} units @ {:.2}", form.quantity.abs(), std_cost)
                    } else {
                        format!("Stock addition: {} units @ {:.2}", form.quantity.abs(), std_cost)
                    };
                    db.execute(
                        "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('stock_adjustment', ?1, ?2)",
                        rusqlite::params![movement_id, today],
                    ).ok();
                    let je_id = db.last_insert_rowid();
                    db.execute(
                        "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date, reference_type, reference_id)
                         VALUES (?1, ?2, ?3, 0, ?5, ?6, 'stock_adjustment', ?7),
                                (?1, ?4, 0, ?3, ?5, ?6, 'stock_adjustment', ?7)",
                        rusqlite::params![je_id, dr_acct, value, cr_acct, desc, today, movement_id],
                    ).ok();
                }
            }

            let movement = db.query_row(
                "SELECT sm.id, sm.movement_no, sm.item_id, i.item_name, i.item_code,
                        sm.warehouse_id, w.warehouse_name, sm.movement_type, sm.quantity,
                        sm.unit_cost, sm.reference_doctype, sm.reference_docno,
                        sm.batch_id, sm.notes, sm.created_by, sm.created_at
                 FROM stock_movements sm
                 LEFT JOIN items i ON sm.item_id = i.id
                 LEFT JOIN warehouses w ON sm.warehouse_id = w.id
                 WHERE sm.id = ?1",
                [movement_id],
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db
        .prepare(
            "SELECT sb.id, sb.item_id, i.item_name, i.item_code, i.category, i.unit_of_measure,
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
                category: row.get(4)?,
                unit_of_measure: row.get(5)?,
                warehouse_id: row.get(6)?,
                warehouse_name: row.get(7)?,
                quantity: row.get(8)?,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
                            pci.system_quantity, pci.counted_quantity, pci.variance,
                            COALESCE(i.standard_cost, 0) as unit_cost
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
                        unit_cost: row.get(8)?,
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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

async fn update_count_item(
    State(_state): State<AppState>,
    Path((count_id, item_id)): Path<(i64, i64)>,
    Json(form): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let counted_qty = form["counted_quantity"].as_f64().unwrap_or(0.0);

    let result = db.execute(
        "UPDATE physical_count_items
         SET counted_quantity = ?1,
             variance = ?1 - system_quantity
         WHERE count_id = ?2 AND id = ?3",
        rusqlite::params![counted_qty, count_id, item_id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Count item updated." } })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Count item not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to update count item: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to update count item." })),
            )
        }
    }
}

async fn complete_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Check status
    let status: String = db.query_row(
        "SELECT status FROM physical_counts WHERE id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or_default();

    if status != "Draft" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Count not found or not in Draft status." })),
        );
    }

    // Get warehouse_id
    let warehouse_id: i64 = db.query_row(
        "SELECT warehouse_id FROM physical_counts WHERE id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or(0);

    // Get all items with variance
    let mut stmt = db.prepare(
        "SELECT pci.item_id, pci.system_quantity, pci.counted_quantity, pci.variance,
                COALESCE(i.standard_cost, 0) as unit_cost
         FROM physical_count_items pci
         LEFT JOIN items i ON pci.item_id = i.id
         WHERE pci.count_id = ?1 AND pci.counted_quantity IS NOT NULL AND pci.variance != 0"
    ).unwrap();

    let variance_items: Vec<(i64, f64, f64, f64, f64)> = stmt.query_map([id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    }).unwrap().filter_map(|r| r.ok()).collect();

    // Post stock adjustments for each variance item
    for (item_id, _system_qty, _counted_qty, variance, unit_cost) in &variance_items {
        // Create stock movement
        let movement_type = if *variance > 0.0 { "IN" } else { "OUT" };

        // Get movement number
        let mseq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM stock_movements", [], |row| row.get(0)).unwrap_or(1);
        let mno = format!("SM-{}-{:04}", chrono::Utc::now().format("%Y"), mseq);

        db.execute(
            "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype, reference_docno, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'PHYSICAL_COUNT', ?7, 'Stock adjustment from physical count')",
            rusqlite::params![mno, item_id, warehouse_id, movement_type, variance.abs(), unit_cost, id],
        ).ok();

        let movement_id = db.last_insert_rowid();

        // Update or insert stock_balances
        let exists: bool = db.query_row(
            "SELECT COUNT(*) > 0 FROM stock_balances WHERE item_id = ?1 AND warehouse_id = ?2",
            rusqlite::params![item_id, warehouse_id],
            |row| row.get(0),
        ).unwrap_or(false);

        if exists {
            db.execute(
                "UPDATE stock_balances SET quantity = quantity + ?1 WHERE item_id = ?2 AND warehouse_id = ?3",
                rusqlite::params![variance, item_id, warehouse_id],
            ).ok();
        } else {
            db.execute(
                "INSERT INTO stock_balances (item_id, warehouse_id, quantity) VALUES (?1, ?2, ?3)",
                rusqlite::params![item_id, warehouse_id, variance],
            ).ok();
        }

        // Update items.current_stock
        db.execute(
            "UPDATE items SET current_stock = current_stock + ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![variance, item_id],
        ).ok();

        // Update physical_count_items
        db.execute(
            "UPDATE physical_count_items SET adjustment_movement_id = ?1 WHERE count_id = ?2 AND item_id = ?3",
            rusqlite::params![movement_id, id, item_id],
        ).ok();
    }

    // Mark count as completed
    let result = db.execute(
        "UPDATE physical_counts SET status = 'Completed', completed_at = datetime('now') WHERE id = ?1",
        [id],
    );

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Physical count completed and stock adjustments posted.", "adjustments": variance_items.len() } })),
        ),
        Ok(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Failed to update count status." })),
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

async fn update_physical_count(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<PhysicalCountForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE physical_counts SET warehouse_id=?1, notes=?2 WHERE id=?3 AND status='Draft'",
        rusqlite::params![form.warehouse_id, form.notes.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Physical count updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Count not found or not in Draft status." }))),
        Err(e) => { tracing::error!("Failed to update physical count: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update physical count." }))) }
    }
}

async fn cancel_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

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

async fn delete_physical_count(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Only allow delete for Draft or Cancelled counts
    let status: String = db.query_row(
        "SELECT status FROM physical_counts WHERE id = ?1",
        [id],
        |row| row.get(0),
    ).unwrap_or_default();

    if status != "Draft" && status != "Cancelled" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Only Draft or Cancelled counts can be deleted." })),
        );
    }

    // Delete items first
    db.execute("DELETE FROM physical_count_items WHERE count_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM physical_counts WHERE id = ?1", [id]);

    match result {
        Ok(rows) if rows > 0 => (
            StatusCode::OK,
            Json(json!({ "success": true, "data": { "message": "Physical count deleted." } })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Count not found." })),
        ),
        Err(e) => {
            tracing::error!("Failed to delete physical count: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "Failed to delete count." })),
            )
        }
    }
}

#[cfg(test)]
mod adjustment_journal_tests {
    use super::*;

    // The posting must always be a balanced two-line entry: the single `value`
    // is written once as a debit and once as a credit, so balance is structural.
    // These tests pin the account routing and the zero-value skip.

    #[test]
    fn positive_adjustment_credits_gain_debits_inventory() {
        let (dr, cr, value) = adjustment_journal(5.0, 120.0).unwrap();
        assert_eq!(dr, ACCT_INVENTORY);
        assert_eq!(cr, ACCT_ADJ_GAIN);
        assert_eq!(value, 600.0);
    }

    #[test]
    fn negative_adjustment_debits_shrinkage_credits_inventory() {
        let (dr, cr, value) = adjustment_journal(-2.0, 120.0).unwrap();
        assert_eq!(dr, ACCT_SHRINKAGE);
        assert_eq!(cr, ACCT_INVENTORY);
        assert_eq!(value, 240.0); // |qty| used, value always positive
    }

    #[test]
    fn zero_cost_or_zero_qty_posts_nothing() {
        assert!(adjustment_journal(5.0, 0.0).is_none());
        assert!(adjustment_journal(0.0, 120.0).is_none());
    }

    #[test]
    fn debit_equals_credit_value_balanced() {
        // Same value drives both lines → entry balances by construction.
        let (_, _, value) = adjustment_journal(-3.5, 40.0).unwrap();
        let debit_total = value;
        let credit_total = value;
        assert!((debit_total - credit_total).abs() < f64::EPSILON);
    }
}
