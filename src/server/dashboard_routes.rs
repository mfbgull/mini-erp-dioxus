use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/dashboard/summary", get(dashboard_summary))
        .route("/api/dashboard/top-customers", get(top_customers))
        .route("/api/dashboard/sales-summary", get(sales_summary))
        .route("/api/dashboard/expense-summary", get(expense_summary))
        .route("/api/dashboard/production-status", get(production_status))
        .route("/api/dashboard/stock-movement-summary", get(stock_movement_summary))
        .route("/api/dashboard/kpi", get(kpi))
        .route("/api/dashboard/ar-summary", get(ar_summary))
        .route("/api/dashboard/layout", get(list_layouts).post(create_layout))
        .route("/api/dashboard/layout/{id}", put(update_layout).delete(delete_layout))
}

async fn dashboard_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let total_items: i64 = db.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0);
    let total_customers: i64 = db.query_row("SELECT COUNT(*) FROM customers WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0);
    let total_suppliers: i64 = db.query_row("SELECT COUNT(*) FROM suppliers WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0);
    let total_invoices: i64 = db.query_row("SELECT COUNT(*) FROM invoices WHERE status != 'Cancelled'", [], |r| r.get(0)).unwrap_or(0);
    let total_revenue: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled'", [], |r| r.get(0)).unwrap_or(0.0);
    let total_expenses: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses", [], |r| r.get(0)).unwrap_or(0.0);
    let outstanding_ar: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE status IN ('Unpaid', 'Partially Paid')", [], |r| r.get(0)).unwrap_or(0.0);
    let outstanding_ap: f64 = db.query_row("SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM supplier_ledger", [], |r| r.get(0)).unwrap_or(0.0);
    let low_stock: i64 = db.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1 AND current_stock <= reorder_level", [], |r| r.get(0)).unwrap_or(0);
    let stock_value: f64 = db.query_row("SELECT COALESCE(SUM(current_stock * standard_cost), 0) FROM items WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "total_items": total_items, "total_customers": total_customers, "total_suppliers": total_suppliers,
            "total_invoices": total_invoices, "total_revenue": total_revenue, "total_expenses": total_expenses,
            "outstanding_ar": outstanding_ar, "outstanding_ap": outstanding_ap, "low_stock_count": low_stock, "stock_value": stock_value,
        }
    })))
}

async fn top_customers(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT c.id, c.customer_name, COALESCE(SUM(i.total_amount), 0) as revenue, COUNT(i.id) as cnt
         FROM customers c LEFT JOIN invoices i ON c.id = i.customer_id AND i.status != 'Cancelled'
         WHERE c.is_active = 1 GROUP BY c.id ORDER BY revenue DESC LIMIT 5"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({ "customer_id": row.get::<_, i64>(0)?, "customer_name": row.get::<_, String>(1)?,
            "total_revenue": row.get::<_, f64>(2)?, "invoice_count": row.get::<_, i64>(3)? }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn sales_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_sales: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE invoice_date = ?1 AND status != 'Cancelled'", [&today], |r| r.get(0)).unwrap_or(0.0);
    let today_count: i64 = db.query_row("SELECT COUNT(*) FROM invoices WHERE invoice_date = ?1 AND status != 'Cancelled'", [&today], |r| r.get(0)).unwrap_or(0);
    // Real weekly sales: Monday of current week to today
    let this_week: f64 = db.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE invoice_date >= date('now', 'weekday 0', '-6 days') AND status != 'Cancelled'",
        [], |r| r.get(0),
    ).unwrap_or(0.0);
    // Real monthly sales: first day of current month to today
    let this_month: f64 = db.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE invoice_date >= date('now', 'start of month') AND status != 'Cancelled'",
        [], |r| r.get(0),
    ).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "today": today_sales, "this_week": this_week, "this_month": this_month, "invoice_count_today": today_count } })))
}

async fn expense_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let this_month: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses WHERE expense_date >= date('now', 'start of month')", [], |r| r.get(0)).unwrap_or(0.0);
    let this_week: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses WHERE expense_date >= date('now', '-7 days')", [], |r| r.get(0)).unwrap_or(0.0);
    let mut stmt = db.prepare("SELECT category, SUM(amount) FROM expenses GROUP BY category ORDER BY SUM(amount) DESC").unwrap();
    let by_category: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({ "category": row.get::<_, String>(0)?, "amount": row.get::<_, f64>(1)? }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": { "this_week": this_week, "this_month": this_month, "by_category": by_category } })))
}

async fn production_status(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let in_progress: i64 = db.query_row("SELECT COUNT(*) FROM productions WHERE status = 'In Progress'", [], |r| r.get(0)).unwrap_or(0);
    let completed: i64 = db.query_row("SELECT COUNT(*) FROM productions WHERE status = 'Completed'", [], |r| r.get(0)).unwrap_or(0);
    let total_output: f64 = db.query_row("SELECT COALESCE(SUM(output_quantity), 0) FROM productions", [], |r| r.get(0)).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "in_progress": in_progress, "completed": completed, "total_output_quantity": total_output } })))
}

async fn stock_movement_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT DATE(created_at) as date,
                SUM(CASE WHEN movement_type = 'IN' THEN quantity ELSE 0 END) as inbound,
                SUM(CASE WHEN movement_type = 'OUT' THEN quantity ELSE 0 END) as outbound
         FROM stock_movements WHERE created_at >= date('now', '-7 days')
         GROUP BY DATE(created_at) ORDER BY date"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({ "date": row.get::<_, String>(0)?, "inbound": row.get::<_, f64>(1)?, "outbound": row.get::<_, f64>(2)? }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn kpi(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let total_items: i64 = db.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0);
    let low_stock: i64 = db.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1 AND current_stock <= reorder_level", [], |r| r.get(0)).unwrap_or(0);
    let health = if total_items > 0 { ((total_items - low_stock) as f64 / total_items as f64) * 100.0 } else { 100.0 };
    (StatusCode::OK, Json(json!({ "success": true, "data": { "stock_health": health, "total_items": total_items, "low_stock_count": low_stock } })))
}

async fn ar_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let current: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date >= ?1 AND status IN ('Unpaid','Partially Paid')", [&today], |r| r.get(0)).unwrap_or(0.0);
    let d1_30: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date < ?1 AND due_date >= date(?1, '-30 days') AND status IN ('Unpaid','Partially Paid')", [&today], |r| r.get(0)).unwrap_or(0.0);
    let d31_60: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date < date(?1, '-30 days') AND due_date >= date(?1, '-60 days') AND status IN ('Unpaid','Partially Paid')", [&today], |r| r.get(0)).unwrap_or(0.0);
    let d61_90: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date < date(?1, '-60 days') AND due_date >= date(?1, '-90 days') AND status IN ('Unpaid','Partially Paid')", [&today], |r| r.get(0)).unwrap_or(0.0);
    let d90_plus: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date < date(?1, '-90 days') AND status IN ('Unpaid','Partially Paid')", [&today], |r| r.get(0)).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "current": current, "days_1_30": d1_30, "days_31_60": d31_60, "days_61_90": d61_90, "days_90_plus": d90_plus } })))
}

// ============================================================================
// Dashboard Layouts
// ============================================================================

async fn list_layouts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, user_id, layout_name, blocks, is_active, created_at FROM dashboard_layouts ORDER BY layout_name").unwrap();
    let items: Vec<DashboardLayout> = stmt.query_map([], |row| {
        Ok(DashboardLayout { id: row.get(0)?, user_id: row.get(1)?, layout_name: row.get(2)?, blocks: row.get(3)?, is_active: row.get::<_, i64>(4)? != 0, created_at: row.get(5)? })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_layout(State(_state): State<AppState>, Json(form): Json<DashboardLayoutForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "INSERT INTO dashboard_layouts (user_id, layout_name, blocks) VALUES (1, ?1, ?2)",
        rusqlite::params![form.layout_name, form.blocks],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create layout: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create layout." }))) }
    }
}

async fn update_layout(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<DashboardLayoutForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE dashboard_layouts SET layout_name=?1, blocks=?2 WHERE id=?3", rusqlite::params![form.layout_name, form.blocks, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Layout updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Layout not found." }))),
        Err(e) => { tracing::error!("Failed to update layout: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update layout." }))) }
    }
}

async fn delete_layout(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("DELETE FROM dashboard_layouts WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Layout deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Layout not found." }))),
        Err(e) => { tracing::error!("Failed to delete layout: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete layout." }))) }
    }
}
