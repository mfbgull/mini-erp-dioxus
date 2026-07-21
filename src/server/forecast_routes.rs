use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use crate::query_report;
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
        .route("/api/forecasts", get(list_forecasts))
        .route("/api/forecasts/run", post(run_forecast))
        .route("/api/forecasts/runs", get(list_forecast_runs))
        .route("/api/forecasts/accuracy", get(list_forecast_accuracy))
        .route("/api/forecasts/config", get(list_model_configs).post(create_model_config))
        .route("/api/forecasts/config/{id}", get(get_model_config).put(update_model_config).delete(delete_model_config))
        .route("/api/forecasts/seasonal-events", get(list_seasonal_events).post(create_seasonal_event))
        .route("/api/forecasts/seasonal-events/{id}", put(update_seasonal_event).delete(delete_seasonal_event))
        .route("/api/forecasts/demand-timeline", get(report_demand_timeline))
}

async fn list_forecasts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT df.id, df.item_id, i.item_name, i.item_code, df.forecast_date,
                df.period, df.predicted_quantity, df.confidence_level,
                df.trend_direction, df.model_type, df.created_at
         FROM demand_forecasts df LEFT JOIN items i ON df.item_id = i.id
         ORDER BY df.created_at DESC LIMIT 200"
    ).unwrap();
    let items: Vec<DemandForecast> = stmt.query_map([], |row| {
        Ok(DemandForecast {
            id: row.get(0)?, item_id: row.get(1)?, item_name: row.get(2)?,
            item_code: row.get(3)?, forecast_date: row.get(4)?, period: row.get(5)?,
            predicted_quantity: row.get(6)?, confidence_level: row.get(7)?,
            trend_direction: row.get(8)?, model_type: row.get(9)?, created_at: row.get(10)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn run_forecast(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let run_id = uuid::Uuid::new_v4().to_string();
    let items_count: i64 = db.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1", [], |r| r.get(0)).unwrap_or(0);
    db.execute(
        "INSERT INTO forecast_runs (run_id, run_type, status, items_processed, completed_at)
         VALUES (?1, 'manual', 'completed', ?2, datetime('now'))",
        rusqlite::params![run_id, items_count],
    ).ok();
    (StatusCode::OK, Json(json!({ "success": true, "data": { "run_id": run_id, "items_processed": items_count, "status": "completed" } })))
}

async fn list_forecast_runs(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, run_id, run_type, status, items_processed, started_at, completed_at FROM forecast_runs ORDER BY started_at DESC").unwrap();
    let items: Vec<ForecastRun> = stmt.query_map([], |row| {
        Ok(ForecastRun {
            id: row.get(0)?, run_id: row.get(1)?, run_type: row.get(2)?,
            status: row.get(3)?, items_processed: row.get(4)?,
            started_at: row.get(5)?, completed_at: row.get(6)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_forecast_accuracy(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT fa.id, fa.forecast_id, fa.item_id, i.item_name, fa.period,
                fa.mape, fa.mae, fa.smape
         FROM forecast_accuracy fa LEFT JOIN items i ON fa.item_id = i.id
         ORDER BY fa.mape DESC LIMIT 100"
    ).unwrap();
    let items: Vec<ForecastAccuracy> = stmt.query_map([], |row| {
        Ok(ForecastAccuracy {
            id: row.get(0)?, forecast_id: row.get(1)?, item_id: row.get(2)?,
            item_name: row.get(3)?, period: row.get(4)?,
            mape: row.get(5)?, mae: row.get(6)?, smape: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_model_configs(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, item_id, category, model_type, alpha, beta, gamma, params_json, model_name
         FROM forecast_model_config"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        let params_json: Option<String> = row.get(7).ok().flatten();
        let params: Option<serde_json::Value> = params_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(json!({
            "id": row.get::<_, i64>(0).unwrap_or(0),
            "item_id": row.get::<_, Option<i64>>(1).unwrap_or(None),
            "category": row.get::<_, Option<String>>(2).unwrap_or(None),
            "model_type": row.get::<_, String>(3).unwrap_or_default(),
            "alpha": row.get::<_, Option<f64>>(4).unwrap_or(None),
            "beta": row.get::<_, Option<f64>>(5).unwrap_or(None),
            "gamma": row.get::<_, Option<f64>>(6).unwrap_or(None),
            "params": params,
            "model_name": row.get::<_, Option<String>>(8).unwrap_or(None),
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_model_config(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, item_id, category, model_type, alpha, beta, gamma, params_json, model_name
         FROM forecast_model_config WHERE id = ?1",
        [id],
        |row| {
            let params_json: Option<String> = row.get(7).ok().flatten();
            let params: Option<serde_json::Value> = params_json
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok());
            Ok(json!({
                "id": row.get::<_, i64>(0).unwrap_or(0),
                "item_id": row.get::<_, Option<i64>>(1).unwrap_or(None),
                "category": row.get::<_, Option<String>>(2).unwrap_or(None),
                "model_type": row.get::<_, String>(3).unwrap_or_default(),
                "alpha": row.get::<_, Option<f64>>(4).unwrap_or(None),
                "beta": row.get::<_, Option<f64>>(5).unwrap_or(None),
                "gamma": row.get::<_, Option<f64>>(6).unwrap_or(None),
                "params": params,
                "model_name": row.get::<_, Option<String>>(8).unwrap_or(None),
            }))
        },
    );
    match result {
        Ok(item) => (StatusCode::OK, Json(json!({ "success": true, "data": item }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Config not found." }))),
    }
}

async fn create_model_config(State(_state): State<AppState>, Json(form): Json<ForecastModelConfigForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let params_json = form.params.as_ref().map(|v| v.to_string());
    let result = db.execute(
        "INSERT INTO forecast_model_config (item_id, category, model_type, alpha, beta, gamma, params_json, model_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![form.item_id, form.category, form.model_type, form.alpha, form.beta, form.gamma, params_json, form.model_name],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create config: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create config." }))) }
    }
}

async fn update_model_config(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<ForecastModelConfigForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let params_json = form.params.as_ref().map(|v| v.to_string());
    let result = db.execute(
        "UPDATE forecast_model_config SET item_id=?1, category=?2, model_type=?3, alpha=?4, beta=?5, gamma=?6, params_json=?7, model_name=?8 WHERE id=?9",
        rusqlite::params![form.item_id, form.category, form.model_type, form.alpha, form.beta, form.gamma, params_json, form.model_name, id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Config updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Config not found." }))),
        Err(e) => { tracing::error!("Failed to update config: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update config." }))) }
    }
}

async fn delete_model_config(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("DELETE FROM forecast_model_config WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Config deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Config not found." }))),
        Err(e) => { tracing::error!("Failed to delete config: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete config." }))) }
    }
}

async fn list_seasonal_events(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, event_name, start_date, end_date, multiplier, applies_to_category, applies_to_item_id FROM forecast_seasonal_events ORDER BY start_date").unwrap();
    let items: Vec<SeasonalEvent> = stmt.query_map([], |row| {
        Ok(SeasonalEvent {
            id: row.get(0)?, event_name: row.get(1)?, start_date: row.get(2)?,
            end_date: row.get(3)?, multiplier: row.get(4)?,
            applies_to_category: row.get(5)?, applies_to_item_id: row.get(6)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_seasonal_event(State(_state): State<AppState>, Json(form): Json<SeasonalEventForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "INSERT INTO forecast_seasonal_events (event_name, start_date, end_date, multiplier, applies_to_category, applies_to_item_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![form.event_name, form.start_date, form.end_date, form.multiplier, form.applies_to_category, form.applies_to_item_id],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create event: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create event." }))) }
    }
}

async fn update_seasonal_event(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SeasonalEventForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE forecast_seasonal_events SET event_name=?1, start_date=?2, end_date=?3, multiplier=?4, applies_to_category=?5, applies_to_item_id=?6 WHERE id=?7",
        rusqlite::params![form.event_name, form.start_date, form.end_date, form.multiplier, form.applies_to_category, form.applies_to_item_id, id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Event updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Event not found." }))),
        Err(e) => { tracing::error!("Failed to update event: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update event." }))) }
    }
}

async fn delete_seasonal_event(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("DELETE FROM forecast_seasonal_events WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Event deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Event not found." }))),
        Err(e) => { tracing::error!("Failed to delete event: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete event." }))) }
    }
}

async fn report_demand_timeline(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let month_names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    // Historical: monthly invoice item quantities
    let historical = query_report!(db,
        "SELECT strftime('%Y-%m', i.invoice_date) as ym, SUM(ii.quantity) as qty
         FROM invoice_items ii JOIN invoices i ON ii.invoice_id = i.id
         WHERE i.status != 'Cancelled'
         GROUP BY ym ORDER BY ym"
    );
    // Forecasted: from demand_forecasts table
    let forecasted = query_report!(db,
        "SELECT period, SUM(predicted_quantity) as qty,
                AVG(confidence_level) as confidence
         FROM demand_forecasts GROUP BY period ORDER BY period"
    );
    // Build timeline
    let mut periods: std::collections::BTreeMap<String, serde_json::Value> = std::collections::BTreeMap::new();
    for row in &historical {
        let ym = row["ym"].as_str().unwrap_or("").to_string();
        let qty = row["qty"].as_f64().unwrap_or(0.0);
        let month_num = if ym.len() >= 7 { ym[5..7].parse::<usize>().unwrap_or(1) } else { 1 };
        let period = month_names.get(month_num - 1).unwrap_or(&"?").to_string();
        periods.insert(ym.clone(), json!({
            "period": period,
            "historical": qty,
            "forecasted": serde_json::Value::Null,
            "lower_bound": serde_json::Value::Null,
            "upper_bound": serde_json::Value::Null,
        }));
    }
    for row in &forecasted {
        let period = row["period"].as_str().unwrap_or("").to_string();
        let qty = row["qty"].as_f64().unwrap_or(0.0);
        let conf = row["confidence"].as_f64().unwrap_or(0.0);
        let lb = qty * (1.0 - conf / 200.0);
        let ub = qty * (1.0 + conf / 200.0);
        if let Some(entry) = periods.get_mut(&period) {
            entry["forecasted"] = json!(qty);
            entry["lower_bound"] = json!(lb);
            entry["upper_bound"] = json!(ub);
        } else {
            let month_num = if period.len() >= 7 { period[5..7].parse::<usize>().unwrap_or(1) } else { 1 };
            let p = month_names.get(month_num - 1).unwrap_or(&"?").to_string();
            periods.insert(period.clone(), json!({
                "period": p,
                "historical": serde_json::Value::Null,
                "forecasted": qty,
                "lower_bound": lb,
                "upper_bound": ub,
            }));
        }
    }
    let items: Vec<serde_json::Value> = periods.into_values().collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}
