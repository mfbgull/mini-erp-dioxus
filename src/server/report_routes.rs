use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/reports/ar-aging", get(report_ar_aging))
        .route("/api/reports/ar-summary", get(report_ar_summary))
        .route("/api/reports/customer-statements", get(report_customer_statements))
        .route("/api/reports/top-debtors", get(report_top_debtors))
        .route("/api/reports/dso", get(report_dso))
        .route("/api/reports/sales-summary", get(report_sales_summary))
        .route("/api/reports/sales-by-customer", get(report_sales_by_customer))
        .route("/api/reports/sales-by-item", get(report_sales_by_item))
        .route("/api/reports/stock-level", get(report_stock_level))
        .route("/api/reports/low-stock", get(report_low_stock))
        .route("/api/reports/stock-valuation", get(report_stock_valuation))
        .route("/api/reports/profit-loss", get(report_profit_loss))
        .route("/api/reports/cash-flow", get(report_cash_flow))
        .route("/api/reports/trial-balance", get(report_trial_balance))
        .route("/api/reports/general-ledger", get(report_general_ledger))
        .route("/api/reports/balance-sheet", get(report_balance_sheet))
        .route("/api/reports/income-statement", get(report_income_statement))
        .route("/api/reports/tax-summary", get(report_tax_summary))
        .route("/api/reports/expenses", get(report_expenses))
        .route("/api/reports/purchase-summary", get(report_purchase_summary))
        .route("/api/reports/supplier-analysis", get(report_supplier_analysis))
        .route("/api/reports/production-summary", get(report_production_summary))
        .route("/api/reports/inventory-movement", get(report_inventory_movement))
        .route("/api/reports/bom-usage", get(report_bom_usage))
        .route("/api/reports/batch-traceability/{itemId}", get(report_batch_traceability))
        // Custom Reports
        .route("/api/reports/custom", get(list_custom_reports).post(create_custom_report))
        .route("/api/reports/custom/{id}", put(update_custom_report).delete(delete_custom_report))
}

macro_rules! query_report {
    ($db:expr, $sql:expr) => {{
        let mut stmt = $db.prepare($sql).unwrap();
        let cols: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let rows: Vec<serde_json::Value> = stmt.query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, col) in cols.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                map.insert(col.clone(), match val {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => json!(n),
                    rusqlite::types::Value::Real(f) => json!(f),
                    rusqlite::types::Value::Text(s) => json!(s),
                    rusqlite::types::Value::Blob(_) => json!("(blob)"),
                });
            }
            Ok(serde_json::Value::Object(map))
        }).unwrap().filter_map(|r| r.ok()).collect();
        rows
    }};
}

async fn report_ar_aging(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let items = query_report!(db,
        "SELECT c.id, c.customer_name, c.current_balance,
            CASE WHEN i.due_date >= ?1 THEN i.balance_amount ELSE 0 END as current,
            CASE WHEN i.due_date < ?1 AND i.due_date >= date(?1, '-30 days') THEN i.balance_amount ELSE 0 END as days_1_30,
            CASE WHEN i.due_date < date(?1, '-30 days') AND i.due_date >= date(?1, '-60 days') THEN i.balance_amount ELSE 0 END as days_31_60,
            CASE WHEN i.due_date < date(?1, '-60 days') AND i.due_date >= date(?1, '-90 days') THEN i.balance_amount ELSE 0 END as days_61_90,
            CASE WHEN i.due_date < date(?1, '-90 days') THEN i.balance_amount ELSE 0 END as days_90_plus
         FROM customers c LEFT JOIN invoices i ON c.id = i.customer_id AND i.status IN ('Unpaid','Partially Paid')
         WHERE c.is_active = 1 GROUP BY c.id HAVING current_balance > 0"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_ar_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let total_ar: f64 = db.query_row(
        "SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE status IN ('Unpaid','Partially Paid')",
        [], |r| r.get(0),
    ).unwrap_or(0.0);
    let current: f64 = db.query_row(
        "SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date >= ?1 AND status IN ('Unpaid','Partially Paid')",
        [&today], |r| r.get(0),
    ).unwrap_or(0.0);
    let overdue: f64 = db.query_row(
        "SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE due_date < ?1 AND status IN ('Unpaid','Partially Paid')",
        [&today], |r| r.get(0),
    ).unwrap_or(0.0);
    let customer_count: i64 = db.query_row(
        "SELECT COUNT(DISTINCT customer_id) FROM invoices WHERE status IN ('Unpaid','Partially Paid')",
        [], |r| r.get(0),
    ).unwrap_or(0);
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": { "total_ar": total_ar, "current": current, "overdue": overdue, "customer_count": customer_count }
    })))
}

async fn report_customer_statements(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT cl.id, cl.customer_id, c.customer_name, cl.transaction_date, cl.type,
                cl.reference_no, cl.debit, cl.credit, cl.balance
         FROM customer_ledger cl LEFT JOIN customers c ON cl.customer_id = c.id
         ORDER BY cl.customer_id, cl.id"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_top_debtors(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT id, customer_name, current_balance FROM customers
         WHERE is_active = 1 AND current_balance > 0 ORDER BY current_balance DESC LIMIT 20"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_dso(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let ar: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE status IN ('Unpaid','Partially Paid')", [], |r| r.get(0)).unwrap_or(0.0);
    let credit_sales: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled'", [], |r| r.get(0)).unwrap_or(0.0);
    let dso = if credit_sales > 0.0 { (ar / credit_sales) * 30.0 } else { 0.0 };
    (StatusCode::OK, Json(json!({ "success": true, "data": { "dso": dso, "accounts_receivable": ar, "credit_sales": credit_sales } })))
}

async fn report_sales_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT invoice_date, COUNT(*) as invoice_count, SUM(total_amount) as total,
                SUM(paid_amount) as paid, SUM(balance_amount) as outstanding
         FROM invoices WHERE status != 'Cancelled' GROUP BY invoice_date ORDER BY invoice_date DESC LIMIT 30"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_sales_by_customer(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT c.id, c.customer_name, COUNT(i.id) as invoice_count, SUM(i.total_amount) as total_amount
         FROM customers c LEFT JOIN invoices i ON c.id = i.customer_id AND i.status != 'Cancelled'
         WHERE c.is_active = 1 GROUP BY c.id ORDER BY total_amount DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_sales_by_item(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, SUM(ii.quantity) as qty_sold, SUM(ii.amount) as total_amount
         FROM items i LEFT JOIN invoice_items ii ON i.id = ii.item_id
         LEFT JOIN invoices inv ON ii.invoice_id = inv.id AND inv.status != 'Cancelled'
         WHERE i.is_active = 1 GROUP BY i.id ORDER BY total_amount DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_stock_level(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, i.category, i.current_stock, i.reorder_level,
                i.standard_cost, (i.current_stock * i.standard_cost) as stock_value
         FROM items i WHERE i.is_active = 1 ORDER BY i.category, i.item_code"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_low_stock(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT id, item_code, item_name, category, current_stock, reorder_level
         FROM items WHERE is_active = 1 AND current_stock <= reorder_level
         ORDER BY (current_stock - reorder_level) ASC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_stock_valuation(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, i.category, i.current_stock, i.standard_cost,
                (i.current_stock * i.standard_cost) as total_value
         FROM items i WHERE i.is_active = 1 ORDER BY total_value DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_profit_loss(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let revenue: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled'", [], |r| r.get(0)).unwrap_or(0.0);
    let cogs: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM invoice_items", [], |r| r.get(0)).unwrap_or(0.0) * 0.6;
    let expenses: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses", [], |r| r.get(0)).unwrap_or(0.0);
    let gross_profit = revenue - cogs;
    let net_profit = gross_profit - expenses;
    (StatusCode::OK, Json(json!({ "success": true, "data": { "revenue": revenue, "cogs": cogs, "gross_profit": gross_profit, "expenses": expenses, "net_profit": net_profit } })))
}

async fn report_cash_flow(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let cash_in: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM payments", [], |r| r.get(0)).unwrap_or(0.0);
    let cash_out: f64 = db.query_row("SELECT COALESCE(SUM(total_cost), 0) FROM purchases", [], |r| r.get(0)).unwrap_or(0.0);
    let expense_out: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses", [], |r| r.get(0)).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "operating_in": cash_in, "operating_out": cash_out + expense_out, "net_cash_flow": cash_in - cash_out - expense_out } })))
}

async fn report_trial_balance(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT a.code, a.name, a.type, a.normal_balance,
                COALESCE(SUM(jl.debit), 0) as total_debit,
                COALESCE(SUM(jl.credit), 0) as total_credit
         FROM chart_of_accounts a LEFT JOIN journal_lines jl ON a.id = jl.account_id AND jl.voided = 0
         GROUP BY a.id ORDER BY a.code"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_general_ledger(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT jl.id, jl.journal_entry_id, a.code, a.name, jl.debit, jl.credit,
                jl.description, jl.line_date, jl.reference_type, jl.reference_id
         FROM journal_lines jl JOIN chart_of_accounts a ON jl.account_id = a.id
         WHERE jl.voided = 0 ORDER BY jl.line_date DESC LIMIT 200"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_balance_sheet(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT a.code, a.name, a.type,
                COALESCE(SUM(jl.debit) - SUM(jl.credit), 0) as balance
         FROM chart_of_accounts a LEFT JOIN journal_lines jl ON a.id = jl.account_id AND jl.voided = 0
         WHERE a.type IN ('Asset', 'Liability', 'Equity')
         GROUP BY a.id ORDER BY a.type, a.code"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_income_statement(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT a.code, a.name, a.type,
                COALESCE(SUM(jl.credit) - SUM(jl.debit), 0) as balance
         FROM chart_of_accounts a LEFT JOIN journal_lines jl ON a.id = jl.account_id AND jl.voided = 0
         WHERE a.type IN ('Revenue', 'Expense')
         GROUP BY a.id ORDER BY a.type DESC, a.code"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_tax_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT ii.tax_rate, COUNT(*) as item_count, SUM(ii.amount) as total_amount,
                SUM(ii.amount * ii.tax_rate / 100) as tax_amount
         FROM invoice_items ii JOIN invoices i ON ii.invoice_id = i.id
         WHERE i.status != 'Cancelled' AND ii.tax_rate > 0
         GROUP BY ii.tax_rate ORDER BY ii.tax_rate"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_expenses(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT category, COUNT(*) as count, SUM(amount) as total
         FROM expenses GROUP BY category ORDER BY total DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_purchase_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT purchase_date, COUNT(*) as count, SUM(total_cost) as total
         FROM purchases GROUP BY purchase_date ORDER BY purchase_date DESC LIMIT 30"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_supplier_analysis(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT supplier_name, COUNT(*) as count, SUM(total_cost) as total
         FROM purchases GROUP BY supplier_name ORDER BY total DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_production_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT i.item_name, COUNT(*) as production_count, SUM(p.output_quantity) as total_output
         FROM productions p JOIN items i ON p.output_item_id = i.id
         GROUP BY p.output_item_id ORDER BY total_output DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_inventory_movement(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT sm.id, sm.movement_no, i.item_name, w.warehouse_name, sm.movement_type,
                sm.quantity, sm.unit_cost, sm.created_at
         FROM stock_movements sm
         LEFT JOIN items i ON sm.item_id = i.id
         LEFT JOIN warehouses w ON sm.warehouse_id = w.id
         ORDER BY sm.created_at DESC LIMIT 200"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_batch_traceability(State(_state): State<AppState>, Path(_item_id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT sb.id, sb.batch_no, i.item_name, w.warehouse_name, sb.quantity_original,
                sb.quantity_remaining, sb.unit_cost, sb.received_date
         FROM stock_batches sb
         LEFT JOIN items i ON sb.item_id = i.id
         LEFT JOIN warehouses w ON sb.warehouse_id = w.id
         ORDER BY sb.received_date"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_bom_usage(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let items = query_report!(db,
        "SELECT b.bom_no, b.bom_name, i.item_name as finished_item,
                COUNT(DISTINCT wo.id) as work_order_count,
                COALESCE(SUM(wo.produced_quantity), 0) as total_produced
         FROM boms b
         LEFT JOIN items i ON b.finished_item_id = i.id
         LEFT JOIN work_orders wo ON b.id = wo.bom_id
         GROUP BY b.id ORDER BY total_produced DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

// ============================================================================
// Custom Reports
// ============================================================================

async fn list_custom_reports(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, user_id, name, config, is_active, last_run_at, created_at FROM custom_reports ORDER BY name").unwrap();
    let items: Vec<CustomReport> = stmt.query_map([], |row| {
        Ok(CustomReport {
            id: row.get(0)?, user_id: row.get(1)?, name: row.get(2)?, config: row.get(3)?,
            is_active: row.get::<_, i64>(4)? != 0, last_run_at: row.get(5)?, created_at: row.get(6)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_custom_report(State(_state): State<AppState>, Json(form): Json<CustomReportForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "INSERT INTO custom_reports (user_id, name, config) VALUES (1, ?1, ?2)",
        rusqlite::params![form.name, form.config],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create report: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create report." }))) }
    }
}

async fn update_custom_report(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<CustomReportForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE custom_reports SET name=?1, config=?2 WHERE id=?3", rusqlite::params![form.name, form.config, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Report updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Report not found." }))),
        Err(e) => { tracing::error!("Failed to update report: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update report." }))) }
    }
}

async fn delete_custom_report(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("DELETE FROM custom_reports WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Report deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Report not found." }))),
        Err(e) => { tracing::error!("Failed to delete report: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete report." }))) }
    }
}
