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
        .route("/api/reports/stock-valuation/fifo", get(report_stock_valuation_fifo))
        .route("/api/reports/stock-history/{itemId}", get(report_stock_history))
        .route("/api/reports/profit-loss", get(report_profit_loss))
        .route("/api/reports/profit-loss/by-item", get(report_profit_loss_by_item))
        .route("/api/reports/cash-flow", get(report_cash_flow))
        .route("/api/reports/trial-balance", get(report_trial_balance))
        .route("/api/reports/general-ledger", get(report_general_ledger))
        .route("/api/reports/balance-sheet", get(report_balance_sheet))
        .route("/api/reports/income-statement", get(report_income_statement))
        .route("/api/reports/tax-summary", get(report_tax_summary))
        .route("/api/reports/trend-decomposition", get(report_trend_decomposition))
        .route("/api/reports/expenses", get(report_expenses))
        .route("/api/reports/purchase-summary", get(report_purchase_summary))
        .route("/api/reports/supplier-analysis", get(report_supplier_analysis))
        .route("/api/reports/production-summary", get(report_production_summary))
        .route("/api/reports/inventory-movement", get(report_inventory_movement))
        .route("/api/reports/bom-usage", get(report_bom_usage))
        .route("/api/reports/batch-traceability/{itemId}", get(report_batch_traceability))
        .route("/api/reports/test-fifo", post(test_fifo_scenario))
        // Custom Reports
        .route("/api/reports/custom", get(list_custom_reports).post(create_custom_report))
        .route("/api/reports/custom/{id}", put(update_custom_report).delete(delete_custom_report))
}

#[macro_export]
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT cl.id, cl.customer_id, c.customer_name, cl.transaction_date, cl.type,
                cl.reference_no, cl.debit, cl.credit, cl.balance
         FROM customer_ledger cl LEFT JOIN customers c ON cl.customer_id = c.id
         ORDER BY cl.customer_id, cl.id"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_top_debtors(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT id, customer_name, current_balance FROM customers
         WHERE is_active = 1 AND current_balance > 0 ORDER BY current_balance DESC LIMIT 20"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_dso(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let ar: f64 = db.query_row("SELECT COALESCE(SUM(balance_amount), 0) FROM invoices WHERE status IN ('Unpaid','Partially Paid')", [], |r| r.get(0)).unwrap_or(0.0);
    let credit_sales: f64 = db.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled'", [], |r| r.get(0)).unwrap_or(0.0);
    let dso = if credit_sales > 0.0 { (ar / credit_sales) * 30.0 } else { 0.0 };
    (StatusCode::OK, Json(json!({ "success": true, "data": { "dso": dso, "accounts_receivable": ar, "credit_sales": credit_sales } })))
}

async fn report_sales_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT invoice_date, COUNT(*) as invoice_count, SUM(total_amount) as total,
                SUM(paid_amount) as paid, SUM(balance_amount) as outstanding
         FROM invoices WHERE status != 'Cancelled' GROUP BY invoice_date ORDER BY invoice_date DESC LIMIT 30"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_sales_by_customer(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT c.id, c.customer_name, COUNT(i.id) as invoice_count, SUM(i.total_amount) as total_amount
         FROM customers c LEFT JOIN invoices i ON c.id = i.customer_id AND i.status != 'Cancelled'
         WHERE c.is_active = 1 GROUP BY c.id ORDER BY total_amount DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_sales_by_item(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, SUM(ii.quantity) as qty_sold, SUM(ii.amount) as total_amount
         FROM items i LEFT JOIN invoice_items ii ON i.id = ii.item_id
         LEFT JOIN invoices inv ON ii.invoice_id = inv.id AND inv.status != 'Cancelled'
         WHERE i.is_active = 1 GROUP BY i.id ORDER BY total_amount DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_stock_level(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, i.category, i.current_stock, i.reorder_level,
                i.standard_cost, (i.current_stock * i.standard_cost) as stock_value
         FROM items i WHERE i.is_active = 1 ORDER BY i.category, i.item_code"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_low_stock(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT id, item_code, item_name, category, current_stock, reorder_level
         FROM items WHERE is_active = 1 AND current_stock <= reorder_level
         ORDER BY (current_stock - reorder_level) ASC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_stock_valuation(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, i.category, i.current_stock, i.standard_cost,
                (i.current_stock * i.standard_cost) as total_value
         FROM items i WHERE i.is_active = 1 ORDER BY total_value DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_stock_valuation_fifo(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // FIFO value per item = SUM(remaining_qty * unit_cost) across all batches
    let items = query_report!(db,
        "SELECT i.id, i.item_code, i.item_name, i.category, i.unit_of_measure,
                COALESCE(sb.stock_qty, 0) as stock_qty,
                i.current_stock,
                COALESCE(sb.fifo_value, 0) as fifo_value,
                CASE WHEN COALESCE(sb.stock_qty, 0) > 0
                     THEN COALESCE(sb.fifo_value, 0) / sb.stock_qty
                     ELSE 0 END as avg_fifo_cost,
                sb.batch_count
         FROM items i
         LEFT JOIN (
             SELECT item_id,
                    SUM(quantity_remaining) as stock_qty,
                    SUM(quantity_remaining * unit_cost) as fifo_value,
                    COUNT(*) as batch_count
             FROM stock_batches
             WHERE quantity_remaining > 0
             GROUP BY item_id
         ) sb ON i.id = sb.item_id
         WHERE i.is_active = 1
         ORDER BY fifo_value DESC"
    );

    // Per-batch detail
    let batches = query_report!(db,
        "SELECT sb.id, sb.batch_no, i.item_code, i.item_name,
                w.warehouse_name, sb.quantity_original, sb.quantity_remaining,
                sb.unit_cost, sb.received_date, sb.source_type,
                sb.quantity_remaining * sb.unit_cost as batch_value
         FROM stock_batches sb
         JOIN items i ON sb.item_id = i.id
         LEFT JOIN warehouses w ON sb.warehouse_id = w.id
         WHERE sb.quantity_remaining > 0
         ORDER BY sb.received_date, sb.id"
    );

    let total_value: f64 = items.iter()
        .map(|i| i["fifo_value"].as_f64().unwrap_or(0.0))
        .sum();
    let total_qty: f64 = items.iter()
        .map(|i| i["stock_qty"].as_f64().unwrap_or(0.0))
        .sum();
    let total_batches: i64 = batches.len() as i64;

    (StatusCode::OK, Json(json!({ "success": true, "data": {
        "items": items,
        "batches": batches,
        "summary": {
            "total_value": total_value,
            "total_quantity": total_qty,
            "total_batches": total_batches,
            "avg_cost": if total_qty > 0.0 { total_value / total_qty } else { 0.0 },
        }
    } })))
}

async fn report_stock_history(
    State(_state): State<AppState>,
    Path(item_id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Item info
    let item = db.query_row(
        "SELECT id, item_code, item_name, category, current_stock, standard_cost
         FROM items WHERE id = ?1",
        [item_id],
        |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "item_code": row.get::<_, String>(1)?,
                "item_name": row.get::<_, String>(2)?,
                "category": row.get::<_, String>(3)?,
                "current_stock": row.get::<_, f64>(4)?,
                "standard_cost": row.get::<_, f64>(5)?,
            }))
        },
    );

    let item_info = match item {
        Ok(i) => i,
        Err(_) => return (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Item not found." }))),
    };

    // 1. Stock movements (receipts, issues, adjustments, transfers)
    let movements = query_report!(db,
        &format!(
            "SELECT sm.movement_no, sm.movement_type, sm.quantity, sm.unit_cost,
                    sm.quantity * sm.unit_cost as total_cost,
                    w.warehouse_name, sm.reference_doctype, sm.reference_docno,
                    sm.batch_id, sm.notes, sm.created_at,
                    sb.batch_no
             FROM stock_movements sm
             LEFT JOIN warehouses w ON sm.warehouse_id = w.id
             LEFT JOIN stock_batches sb ON sm.batch_id = sb.id
             WHERE sm.item_id = {}
             ORDER BY sm.created_at ASC, sm.id ASC",
            item_id
        )
    );

    // 2. Purchases (direct purchases)
    let purchases = query_report!(db,
        &format!(
            "SELECT p.purchase_no, p.quantity, p.unit_cost, p.total_cost,
                    p.supplier_name, p.purchase_date, p.status,
                    w.warehouse_name, sb.batch_no
             FROM purchases p
             LEFT JOIN warehouses w ON p.warehouse_id = w.id
             LEFT JOIN stock_batches sb ON p.batch_id = sb.id
             WHERE p.item_id = {}
             ORDER BY p.purchase_date ASC, p.id ASC",
            item_id
        )
    );

    // 3. Invoice items (sales)
    let sales = query_report!(db,
        &format!(
            "SELECT inv.invoice_no, inv.invoice_date, inv.status,
                    ii.quantity, ii.unit_price, ii.amount,
                    c.customer_name
             FROM invoice_items ii
             JOIN invoices inv ON ii.invoice_id = inv.id
             LEFT JOIN customers c ON inv.customer_id = c.id
             WHERE ii.item_id = {}
             ORDER BY inv.invoice_date ASC, inv.id ASC",
            item_id
        )
    );

    // 4. Batch history
    let batches = query_report!(db,
        &format!(
            "SELECT sb.batch_no, sb.quantity_original, sb.quantity_remaining,
                    sb.unit_cost, sb.quantity_original * sb.unit_cost as original_value,
                    sb.quantity_remaining * sb.unit_cost as remaining_value,
                    sb.received_date, sb.source_type, w.warehouse_name
             FROM stock_batches sb
             LEFT JOIN warehouses w ON sb.warehouse_id = w.id
             WHERE sb.item_id = {}
             ORDER BY sb.received_date ASC, sb.id ASC",
            item_id
        )
    );

    // Compute running balance from movements
    let mut running_balance: f64 = 0.0;
    let movements_with_balance: Vec<serde_json::Value> = movements.into_iter().map(|m| {
        let qty = m["quantity"].as_f64().unwrap_or(0.0);
        let movement_type = m["movement_type"].as_str().unwrap_or("");
        match movement_type {
            "IN" | "TRANSFER_IN" => running_balance += qty,
            "OUT" | "TRANSFER_OUT" => running_balance -= qty,
            "ADJUSTMENT" => running_balance += qty, // adjustments can be positive or negative
            _ => {}
        }
        let mut enriched = m.clone();
        enriched.as_object_mut().map(|obj| {
            obj.insert("running_balance".to_string(), json!(running_balance));
        });
        enriched
    }).collect();

    // Summary stats
    let total_in: f64 = movements_with_balance.iter()
        .filter(|m| matches!(m["movement_type"].as_str(), Some("IN") | Some("TRANSFER_IN")))
        .map(|m| m["quantity"].as_f64().unwrap_or(0.0))
        .sum();
    let total_out: f64 = movements_with_balance.iter()
        .filter(|m| matches!(m["movement_type"].as_str(), Some("OUT") | Some("TRANSFER_OUT")))
        .map(|m| m["quantity"].as_f64().unwrap_or(0.0))
        .sum();
    let total_adjusted: f64 = movements_with_balance.iter()
        .filter(|m| m["movement_type"].as_str() == Some("ADJUSTMENT"))
        .map(|m| m["quantity"].as_f64().unwrap_or(0.0))
        .sum();

    (StatusCode::OK, Json(json!({ "success": true, "data": {
        "item": item_info,
        "movements": movements_with_balance,
        "purchases": purchases,
        "sales": sales,
        "batches": batches,
        "summary": {
            "total_received": total_in,
            "total_issued": total_out,
            "total_adjusted": total_adjusted,
            "current_stock": running_balance,
        }
    } })))
}

async fn report_profit_loss(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let from_date = params.get("from_date").map(|s| s.as_str()).unwrap_or("2000-01-01");
    let to_date = params.get("to_date").map(|s| s.as_str()).unwrap_or("2099-12-31");

    let revenue: f64 = db.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM invoices WHERE status != 'Cancelled' AND invoice_date BETWEEN ?1 AND ?2",
        rusqlite::params![from_date, to_date],
        |r| r.get(0)
    ).unwrap_or(0.0);

    let cogs: f64 = db.query_row(
        "SELECT COALESCE(SUM((quantity_original - quantity_remaining) * unit_cost), 0)
         FROM stock_batches WHERE quantity_remaining < quantity_original AND received_date BETWEEN ?1 AND ?2",
        rusqlite::params![from_date, to_date],
        |r| r.get(0)
    ).unwrap_or(0.0);

    let expenses: f64 = db.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM expenses WHERE expense_date BETWEEN ?1 AND ?2",
        rusqlite::params![from_date, to_date],
        |r| r.get(0)
    ).unwrap_or(0.0);

    let gross_profit = revenue - cogs;
    let net_profit = gross_profit - expenses;

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "revenue": revenue,
            "cogs": cogs,
            "gross_profit": gross_profit,
            "expenses": expenses,
            "net_profit": net_profit,
            "from_date": from_date,
            "to_date": to_date
        }
    })))
}

async fn report_profit_loss_by_item(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // Revenue per item from invoiced items
    let revenue_by_item = query_report!(db,
        "SELECT ii.item_id, i.item_code, i.item_name,
                SUM(ii.quantity) as qty_sold,
                SUM(ii.amount) as revenue
         FROM invoice_items ii
         JOIN items i ON ii.item_id = i.id
         JOIN invoices inv ON ii.invoice_id = inv.id AND inv.status != 'Cancelled'
         GROUP BY ii.item_id"
    );

    // FIFO COGS per item from consumed batches
    let cogs_by_item = query_report!(db,
        "SELECT item_id,
                SUM((quantity_original - quantity_remaining) * unit_cost) as cogs
         FROM stock_batches
         WHERE quantity_remaining < quantity_original
         GROUP BY item_id"
    );

    // Purchase cost per item (total purchases made)
    let purchase_by_item = query_report!(db,
        "SELECT item_id, SUM(total_cost) as total_purchased
         FROM purchases
         GROUP BY item_id"
    );

    // Build a map of COGS by item_id
    let mut cogs_map: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for row in &cogs_by_item {
        if let (Some(id), Some(cost)) = (row["item_id"].as_i64(), row["cogs"].as_f64()) {
            cogs_map.insert(id, cost);
        }
    }

    // Build a map of purchases by item_id
    let mut purchase_map: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for row in &purchase_by_item {
        if let (Some(id), Some(cost)) = (row["item_id"].as_i64(), row["total_purchased"].as_f64()) {
            purchase_map.insert(id, cost);
        }
    }

    // Merge into per-item P&L
    let mut items: Vec<serde_json::Value> = revenue_by_item.into_iter().map(|row| {
        let item_id = row["item_id"].as_i64().unwrap_or(0);
        let revenue = row["revenue"].as_f64().unwrap_or(0.0);
        let cogs = cogs_map.get(&item_id).copied().unwrap_or(0.0);
        let gross_profit = revenue - cogs;
        let margin_pct = if revenue > 0.0 { gross_profit / revenue * 100.0 } else { 0.0 };
        json!({
            "item_id": item_id,
            "item_code": row["item_code"],
            "item_name": row["item_name"],
            "qty_sold": row["qty_sold"],
            "revenue": revenue,
            "cogs": cogs,
            "gross_profit": gross_profit,
            "margin_pct": (margin_pct * 100.0).round() / 100.0,
            "total_purchased": purchase_map.get(&item_id).copied().unwrap_or(0.0),
        })
    }).collect();

    // Sort by gross profit descending
    items.sort_by(|a, b| {
        b["gross_profit"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["gross_profit"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_revenue: f64 = items.iter().map(|i| i["revenue"].as_f64().unwrap_or(0.0)).sum();
    let total_cogs: f64 = items.iter().map(|i| i["cogs"].as_f64().unwrap_or(0.0)).sum();
    let total_gross_profit = total_revenue - total_cogs;
    let expenses: f64 = db.query_row(
        "SELECT COALESCE(SUM(amount), 0) FROM expenses", [], |r| r.get(0)
    ).unwrap_or(0.0);
    let net_profit = total_gross_profit - expenses;

    (StatusCode::OK, Json(json!({ "success": true, "data": {
        "items": items,
        "summary": {
            "total_revenue": total_revenue,
            "total_cogs": total_cogs,
            "total_gross_profit": total_gross_profit,
            "expenses": expenses,
            "net_profit": net_profit,
        }
    } })))
}

async fn report_cash_flow(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let cash_in: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM payments", [], |r| r.get(0)).unwrap_or(0.0);
    let cash_out: f64 = db.query_row("SELECT COALESCE(SUM(total_cost), 0) FROM purchases", [], |r| r.get(0)).unwrap_or(0.0);
    let expense_out: f64 = db.query_row("SELECT COALESCE(SUM(amount), 0) FROM expenses", [], |r| r.get(0)).unwrap_or(0.0);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "operating_in": cash_in, "operating_out": cash_out + expense_out, "net_cash_flow": cash_in - cash_out - expense_out } })))
}

async fn report_trial_balance(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT jl.id, jl.journal_entry_id, a.code, a.name, jl.debit, jl.credit,
                jl.description, jl.line_date, jl.reference_type, jl.reference_id
         FROM journal_lines jl JOIN chart_of_accounts a ON jl.account_id = a.id
         WHERE jl.voided = 0 ORDER BY jl.line_date DESC LIMIT 200"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_balance_sheet(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let from_date = params.get("from_date").map(|s| s.as_str()).unwrap_or("2000-01-01");
    let to_date = params.get("to_date").map(|s| s.as_str()).unwrap_or("2099-12-31");

    let mut stmt = db.prepare(
        "SELECT a.code, a.name, a.type,
                COALESCE(SUM(jl.debit) - SUM(jl.credit), 0) as balance
         FROM chart_of_accounts a LEFT JOIN journal_lines jl ON a.id = jl.account_id AND jl.voided = 0
              AND jl.line_date BETWEEN ?1 AND ?2
         WHERE a.type IN ('Asset', 'Liability', 'Equity')
         GROUP BY a.id ORDER BY a.type, a.code"
    ).unwrap();

    let cols: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let items: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        let mut map = serde_json::Map::new();
        for (i, col) in cols.iter().enumerate() {
            let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
            map.insert(col.clone(), match val {
                rusqlite::types::Value::Null => serde_json::Value::Null,
                rusqlite::types::Value::Integer(n) => json!(n),
                rusqlite::types::Value::Real(f) => json!(f),
                rusqlite::types::Value::Text(s) => json!(s),
                rusqlite::types::Value::Blob(b) => json!(format!("<{} bytes>", b.len())),
            });
        }
        Ok(serde_json::Value::Object(map))
    }).unwrap().filter_map(|r| r.ok()).collect();

    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_income_statement(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    // returns sales_tax, income_tax, withholding_tax arrays grouped by month
    let sales_tax = query_report!(db,
        "SELECT strftime('%Y-%m', i.invoice_date) as period,
                SUM(i.total_amount) as tax_base,
                COALESCE(AVG(i.tax_rate), 0) as rate,
                SUM(i.total_amount * i.tax_rate / 100) as tax_amount,
                SUM(i.paid_amount * i.tax_rate / 100) as paid_amount
         FROM invoices i
         WHERE i.status != 'Cancelled' AND i.tax_rate > 0
         GROUP BY strftime('%Y-%m', i.invoice_date)
         ORDER BY period"
    );
    // Add balance = tax_amount - paid_amount to each row
    let sales_tax: Vec<serde_json::Value> = sales_tax.into_iter().map(|mut row| {
        let tax = row["tax_amount"].as_f64().unwrap_or(0.0);
        let paid = row["paid_amount"].as_f64().unwrap_or(0.0);
        row.as_object_mut().map(|obj| {
            obj.insert("balance".to_string(), json!(tax - paid));
        });
        row
    }).collect();
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "sales_tax": sales_tax,
            "income_tax": [],
            "withholding_tax": []
        }
    })))
}

async fn report_trend_decomposition(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    // Monthly invoice totals for the last 12 months
    let month_names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let raw = query_report!(db,
        "SELECT strftime('%Y-%m', invoice_date) as ym, SUM(total_amount) as actual
         FROM invoices WHERE status != 'Cancelled'
         GROUP BY ym ORDER BY ym DESC LIMIT 12"
    );
    let mut rows: Vec<serde_json::Value> = raw.into_iter().rev().collect();
    // Compute 3-month moving average as trend
    for i in 0..rows.len() {
        let actual = rows[i]["actual"].as_f64().unwrap_or(0.0);
        let trend = if i >= 2 {
            (rows[i-2]["actual"].as_f64().unwrap_or(0.0)
             + rows[i-1]["actual"].as_f64().unwrap_or(0.0)
             + actual) / 3.0
        } else if i == 1 {
            (rows[0]["actual"].as_f64().unwrap_or(0.0) + actual) / 2.0
        } else {
            actual
        };
        let ym = rows[i]["ym"].as_str().unwrap_or("").to_string();
        let month_num = ym[5..7].parse::<usize>().unwrap_or(1);
        let period = month_names.get(month_num - 1).unwrap_or(&"?").to_string();
        rows[i].as_object_mut().map(|obj| {
            obj.insert("period".to_string(), json!(period));
            obj.insert("trend".to_string(), json!(trend));
            obj.insert("seasonal".to_string(), json!(actual - trend));
            obj.insert("residual".to_string(), json!(0.0));
        });
    }
    (StatusCode::OK, Json(json!({ "success": true, "data": rows })))
}

async fn report_expenses(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT category, COUNT(*) as count, SUM(amount) as total
         FROM expenses GROUP BY category ORDER BY total DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_purchase_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT purchase_date, COUNT(*) as count, SUM(total_cost) as total
         FROM purchases GROUP BY purchase_date ORDER BY purchase_date DESC LIMIT 30"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_supplier_analysis(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT supplier_name, COUNT(*) as count, SUM(total_cost) as total
         FROM purchases GROUP BY supplier_name ORDER BY total DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_production_summary(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let items = query_report!(db,
        "SELECT i.item_name, COUNT(*) as production_count, SUM(p.output_quantity) as total_output
         FROM productions p JOIN items i ON p.output_item_id = i.id
         GROUP BY p.output_item_id ORDER BY total_output DESC"
    );
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn report_inventory_movement(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
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
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE custom_reports SET name=?1, config=?2 WHERE id=?3", rusqlite::params![form.name, form.config, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Report updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Report not found." }))),
        Err(e) => { tracing::error!("Failed to update report: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update report." }))) }
    }
}

async fn delete_custom_report(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("DELETE FROM custom_reports WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Report deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Report not found." }))),
        Err(e) => { tracing::error!("Failed to delete report: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete report." }))) }
    }
}

/// Test endpoint that runs through the user's FIFO scenario end-to-end:
///   1. Buy 1 unit at 100  → batch B1
///   2. Sell 1 unit        → FIFO consumes B1 (cost 100)
///   3. Buy 1 unit at 105  → batch B2
///   4. Sell 1 unit        → FIFO consumes B2 (cost 105)
///   5. Returns full trace showing FIFO worked correctly.
async fn test_fifo_scenario(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());

    // --- Ensure a test warehouse exists ---
    let wh_id: i64 = db.query_row(
        "SELECT id FROM warehouses WHERE warehouse_code = 'WH-FIFO-TEST' LIMIT 1",
        [], |row| row.get(0),
    ).unwrap_or_else(|_| {
        db.execute(
            "INSERT INTO warehouses (warehouse_code, warehouse_name, location) VALUES ('WH-FIFO-TEST', 'FIFO Test Warehouse', 'Test')",
            [],
        ).ok();
        db.last_insert_rowid()
    });

    // --- Ensure a test item exists ---
    let item_id: i64 = db.query_row(
        "SELECT id FROM items WHERE item_code = 'FIFO-TEST' LIMIT 1",
        [], |row| row.get(0),
    ).unwrap_or_else(|_| {
        db.execute(
            "INSERT INTO items (item_code, item_name, category, current_stock, reorder_level, standard_cost, selling_price, is_active)
             VALUES ('FIFO-TEST', 'FIFO Test Item', 'Test', 0, 0, 0, 0, 1)",
            [],
        ).ok();
        db.last_insert_rowid()
    });

    // Reset stock to zero for clean test
    db.execute("UPDATE items SET current_stock = 0 WHERE id = ?1", [item_id]).ok();
    db.execute("DELETE FROM stock_balances WHERE item_id = ?1", [item_id]).ok();
    db.execute("DELETE FROM stock_batches WHERE item_id = ?1", [item_id]).ok();
    db.execute("DELETE FROM purchases WHERE item_id = ?1 AND warehouse_id = ?2", rusqlite::params![item_id, wh_id]).ok();
    db.execute(
        "DELETE FROM stock_movements WHERE item_id = ?1 AND warehouse_id = ?2",
        rusqlite::params![item_id, wh_id],
    ).ok();

    let mut trace: Vec<serde_json::Value> = Vec::new();

    // ===== STEP 1: Buy 1 unit at 100 =====
    let p1_no = format!("PUR-FIFO-{:04}", 1);
    let p1_total = 1.0 * 100.0;
    db.execute(
        "INSERT INTO purchases (purchase_no, item_id, warehouse_id, quantity, unit_cost, total_cost, supplier_name, purchase_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'FIFO Test Supplier', date('now'))",
        rusqlite::params![p1_no, item_id, wh_id, 1.0, 100.0, p1_total],
    ).ok();
    let p1_id = db.last_insert_rowid();

    // Create stock batch B1
    let b1_no = format!("{}-BATCH", p1_no);
    db.execute(
        "INSERT INTO stock_batches (batch_no, item_id, warehouse_id, source_type, source_id,
            quantity_original, quantity_remaining, unit_cost, received_date)
         VALUES (?1, ?2, ?3, 'PURCHASE', ?4, ?5, ?5, ?6, datetime('now'))",
        rusqlite::params![b1_no, item_id, wh_id, p1_id, 1.0, 100.0],
    ).ok();
    let b1_id = db.last_insert_rowid();
    db.execute("UPDATE purchases SET batch_id = ?1 WHERE id = ?2", rusqlite::params![b1_id, p1_id]).ok();

    // Update stock
    db.execute("UPDATE stock_balances SET quantity = quantity + 1 WHERE item_id = ?1 AND warehouse_id = ?2",
        rusqlite::params![item_id, wh_id]).ok();
    db.execute("UPDATE items SET current_stock = current_stock + 1 WHERE id = ?1", [item_id]).ok();

    trace.push(json!({
        "step": 1, "action": "BUY", "quantity": 1.0, "unit_cost": 100.0,
        "batch_created": b1_no, "batch_id": b1_id,
    }));

    // ===== STEP 2: Sell 1 unit (OUT) =====
    // Load batches FIFO
    let mut stmt = db.prepare(
        "SELECT id, quantity_remaining, unit_cost FROM stock_batches
         WHERE item_id = ?1 AND warehouse_id = ?2 AND quantity_remaining > 0
         ORDER BY received_date ASC, id ASC"
    ).unwrap();
    let batches: Vec<(i64, f64, f64)> = stmt.query_map(
        rusqlite::params![item_id, wh_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap().filter_map(|r| r.ok()).collect();

    let batch_refs: Vec<crate::calculations::stock::StockBatch> = batches.iter()
        .map(|(id, qty, cost)| crate::calculations::stock::StockBatch { id: *id, quantity_remaining: *qty, unit_cost: *cost })
        .collect();
    let fifo_result = crate::calculations::stock::consume_fifo_batches(&batch_refs, 1.0);

    // Update batch quantities
    for updated in &fifo_result.updated_batches {
        db.execute("UPDATE stock_batches SET quantity_remaining = ?1 WHERE id = ?2",
            rusqlite::params![updated.quantity_remaining, updated.id]).ok();
    }

    // Create stock movement
    let sm2_no = format!("SM-FIFO-{:04}", 2);
    let last_batch_id = batches.last().map(|b| b.0);
    db.execute(
        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type,
            quantity, unit_cost, batch_id, notes)
         VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, ?6, 'FIFO test - sell 1')",
        rusqlite::params![sm2_no, item_id, wh_id, 1.0, fifo_result.weighted_avg_cost, last_batch_id],
    ).ok();

    // Update stock
    db.execute("UPDATE stock_balances SET quantity = quantity - 1 WHERE item_id = ?1 AND warehouse_id = ?2",
        rusqlite::params![item_id, wh_id]).ok();
    db.execute("UPDATE items SET current_stock = current_stock - 1 WHERE id = ?1", [item_id]).ok();

    trace.push(json!({
        "step": 2, "action": "SELL", "quantity": 1.0,
        "fifo_unit_cost": fifo_result.weighted_avg_cost,
        "consumed_from_batch": batches.first().map(|b| b.0),
        "movement_no": sm2_no,
        "expected_cost": 100.0,
        "actual_cost": fifo_result.weighted_avg_cost,
        "correct": (fifo_result.weighted_avg_cost - 100.0).abs() < 0.01,
    }));

    // ===== STEP 3: Buy 1 unit at 105 =====
    let p3_no = format!("PUR-FIFO-{:04}", 3);
    let p3_total = 1.0 * 105.0;
    db.execute(
        "INSERT INTO purchases (purchase_no, item_id, warehouse_id, quantity, unit_cost, total_cost, supplier_name, purchase_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'FIFO Test Supplier', date('now'))",
        rusqlite::params![p3_no, item_id, wh_id, 1.0, 105.0, p3_total],
    ).ok();
    let p3_id = db.last_insert_rowid();

    let b3_no = format!("{}-BATCH", p3_no);
    db.execute(
        "INSERT INTO stock_batches (batch_no, item_id, warehouse_id, source_type, source_id,
            quantity_original, quantity_remaining, unit_cost, received_date)
         VALUES (?1, ?2, ?3, 'PURCHASE', ?4, ?5, ?5, ?6, datetime('now'))",
        rusqlite::params![b3_no, item_id, wh_id, p3_id, 1.0, 105.0],
    ).ok();
    let b3_id = db.last_insert_rowid();
    db.execute("UPDATE purchases SET batch_id = ?1 WHERE id = ?2", rusqlite::params![b3_id, p3_id]).ok();

    db.execute("UPDATE stock_balances SET quantity = quantity + 1 WHERE item_id = ?1 AND warehouse_id = ?2",
        rusqlite::params![item_id, wh_id]).ok();
    db.execute("UPDATE items SET current_stock = current_stock + 1 WHERE id = ?1", [item_id]).ok();

    trace.push(json!({
        "step": 3, "action": "BUY", "quantity": 1.0, "unit_cost": 105.0,
        "batch_created": b3_no, "batch_id": b3_id,
    }));

    // ===== STEP 4: Sell 1 unit (OUT) =====
    let mut stmt2 = db.prepare(
        "SELECT id, quantity_remaining, unit_cost FROM stock_batches
         WHERE item_id = ?1 AND warehouse_id = ?2 AND quantity_remaining > 0
         ORDER BY received_date ASC, id ASC"
    ).unwrap();
    let batches2: Vec<(i64, f64, f64)> = stmt2.query_map(
        rusqlite::params![item_id, wh_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap().filter_map(|r| r.ok()).collect();

    let batch_refs2: Vec<crate::calculations::stock::StockBatch> = batches2.iter()
        .map(|(id, qty, cost)| crate::calculations::stock::StockBatch { id: *id, quantity_remaining: *qty, unit_cost: *cost })
        .collect();
    let fifo_result2 = crate::calculations::stock::consume_fifo_batches(&batch_refs2, 1.0);

    for updated in &fifo_result2.updated_batches {
        db.execute("UPDATE stock_batches SET quantity_remaining = ?1 WHERE id = ?2",
            rusqlite::params![updated.quantity_remaining, updated.id]).ok();
    }

    let sm4_no = format!("SM-FIFO-{:04}", 4);
    let last_batch_id2 = batches2.last().map(|b| b.0);
    db.execute(
        "INSERT INTO stock_movements (movement_no, item_id, warehouse_id, movement_type,
            quantity, unit_cost, batch_id, notes)
         VALUES (?1, ?2, ?3, 'OUT', ?4, ?5, ?6, 'FIFO test - sell 1')",
        rusqlite::params![sm4_no, item_id, wh_id, 1.0, fifo_result2.weighted_avg_cost, last_batch_id2],
    ).ok();

    db.execute("UPDATE stock_balances SET quantity = quantity - 1 WHERE item_id = ?1 AND warehouse_id = ?2",
        rusqlite::params![item_id, wh_id]).ok();
    db.execute("UPDATE items SET current_stock = current_stock - 1 WHERE id = ?1", [item_id]).ok();

    trace.push(json!({
        "step": 4, "action": "SELL", "quantity": 1.0,
        "fifo_unit_cost": fifo_result2.weighted_avg_cost,
        "consumed_from_batch": batches2.first().map(|b| b.0),
        "movement_no": sm4_no,
        "expected_cost": 105.0,
        "actual_cost": fifo_result2.weighted_avg_cost,
        "correct": (fifo_result2.weighted_avg_cost - 105.0).abs() < 0.01,
    }));

    // ===== FINAL STATE =====
    let final_stock: f64 = db.query_row(
        "SELECT current_stock FROM items WHERE id = ?1", [item_id], |row| row.get(0),
    ).unwrap_or(0.0);

    let remaining_batches: Vec<serde_json::Value> = query_report!(db,
        &format!("SELECT batch_no, quantity_original, quantity_remaining, unit_cost
                  FROM stock_batches WHERE item_id = {} ORDER BY received_date, id", item_id)
    );

    // FIFO COGS
    let cogs: f64 = db.query_row(
        "SELECT COALESCE(SUM((quantity_original - quantity_remaining) * unit_cost), 0)
         FROM stock_batches WHERE item_id = ?1 AND quantity_remaining < quantity_original",
        [item_id], |row| row.get(0),
    ).unwrap_or(0.0);

    // Clean up test data
    db.execute("DELETE FROM stock_movements WHERE item_id = ?1 AND warehouse_id = ?2", rusqlite::params![item_id, wh_id]).ok();
    db.execute("DELETE FROM purchases WHERE item_id = ?1 AND warehouse_id = ?2", rusqlite::params![item_id, wh_id]).ok();
    db.execute("DELETE FROM stock_batches WHERE item_id = ?1", [item_id]).ok();
    db.execute("DELETE FROM stock_balances WHERE item_id = ?1", [item_id]).ok();
    db.execute("DELETE FROM items WHERE id = ?1", [item_id]).ok();
    db.execute("DELETE FROM warehouses WHERE id = ?1", [wh_id]).ok();

    let all_correct = trace.iter().all(|t| t["correct"].as_bool().unwrap_or(false));

    (StatusCode::OK, Json(json!({
        "success": true,
        "scenario": "Buy at 100, sell at 110, buy at 105, sell at 115",
        "trace": trace,
        "summary": {
            "total_revenue": 110.0 + 115.0,
            "total_cogs": cogs,
            "gross_profit": (110.0 + 115.0) - cogs,
            "final_stock": final_stock,
            "remaining_batches": remaining_batches,
        },
        "expected_cogs": 205.0,
        "actual_cogs": cogs,
        "fifo_correct": all_correct,
    })))
}
