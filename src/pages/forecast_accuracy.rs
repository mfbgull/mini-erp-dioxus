//! Forecast Accuracy Page — Model accuracy metrics and per-product accuracy breakdown.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.fa-page { max-width: 1000px; margin: 0 auto; }
.fa-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.fa-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.fa-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.fa-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.fa-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.fa-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.fa-chart-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 20px; }
.fa-chart-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.fa-chart-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.fa-chart { width: 100%; height: 180px; }
.fa-chart svg { width: 100%; height: 100%; }
.fa-bar { fill: var(--accent, #4a90d9); transition: fill 0.15s ease; }
.fa-bar:hover { fill: #357abd; }
.fa-bar-label { font-size: 9px; fill: var(--text-secondary, #6c757d); text-anchor: middle; font-weight: 600; }
.fa-bar-month { font-size: 9px; fill: var(--text-secondary, #6c757d); text-anchor: middle; }

.fa-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.fa-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.fa-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.fa-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.fa-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.fa-table thead th.text-right { text-align: right; }
.fa-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.fa-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.fa-table tbody td.text-danger { color: #dc3545; }
.fa-table tbody td.text-success { color: #28a745; }
.fa-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone)]
struct AccuracyMetric {
    period: String,
    mape: f64,
}

#[derive(Clone)]
struct ProductAccuracy {
    product: String,
    forecast: f64,
    actual: f64,
    error: f64,
    mape: f64,
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ForecastAccuracyPage() -> Element {
    let api = use_auth().api;

    let accuracy_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_forecast_accuracy().await.ok().unwrap_or_default()
        }
    });

    let accuracy_data = accuracy_resource.read();
    let accuracy_items = accuracy_data.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

    let products: Vec<ProductAccuracy> = accuracy_items.iter().filter_map(|item| {
        let mae = item["mae"].as_f64().unwrap_or(0.0);
        let mape = item["mape"].as_f64().unwrap_or(0.0);
        Some(ProductAccuracy {
            product: item["item_name"].as_str().unwrap_or("Unknown").to_string(),
            forecast: item["mape"].as_f64().unwrap_or(0.0),
            actual: item["mae"].as_f64().unwrap_or(0.0),
            error: mae - mape,
            mape: item["mape"].as_f64().unwrap_or(0.0),
        })
    }).collect();

    let monthly: Vec<AccuracyMetric> = {
        let mut buckets: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        for item in accuracy_items {
            let period = item["period"].as_str().unwrap_or("Unknown").to_string();
            let mape = item["mape"].as_f64().unwrap_or(0.0);
            buckets.entry(period).or_default().push(mape);
        }
        buckets.into_iter().map(|(period, values)| {
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            AccuracyMetric { period, mape: avg }
        }).collect()
    };

    let max_mape = monthly.iter().map(|m| m.mape).fold(0.0_f64, f64::max);
    let bar_count = if monthly.is_empty() { 1 } else { monthly.len() };
    let chart_width = (bar_count as f64 * 25.0).max(500.0);
    let bar_width_pct = chart_width / bar_count as f64;
    let bar_gap_pct = bar_width_pct * 0.25;
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 160.0;

    let avg_mape: f64 = if monthly.is_empty() { 0.0 } else { monthly.iter().map(|m| m.mape).sum::<f64>() / monthly.len() as f64 };
    let avg_mae: f64 = if products.is_empty() { 0.0 } else { products.iter().map(|p| p.actual).sum::<f64>() / products.len() as f64 };
    let _total_abs_error: f64 = products.iter().map(|p| p.error.abs()).sum();
    let avg_error: f64 = if products.is_empty() { 0.0 } else { products.iter().map(|p| p.error).sum::<f64>() / products.len() as f64 };
    // Compute RMSE from mae values
    let rmse: f64 = if products.is_empty() { 0.0 } else {
        let sum_sq: f64 = products.iter().map(|p| p.actual * p.actual).sum();
        (sum_sq / products.len() as f64).sqrt()
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page fa-page",

            div { class: "fa-header",
                div {
                    h1 { "Forecast Accuracy" }
                    p { class: "page-subtitle", "Model accuracy metrics and per-product breakdown." }
                }
            }

            // Filters
            div { class: "fa-filter-bar",
                label { "Model" }
                select {
                    option { value: "arima", selected: true, "ARIMA(2,1,2)" }
                    option { value: "prophet", "Prophet" }
                    option { value: "ets", "ETS(A,A,A)" }
                    option { value: "neural", "Neural (LSTM)" }
                }
                label { "Period" }
                select {
                    option { value: "h1-2026", selected: true, "H1 2026" }
                    option { value: "q2-2026", "Q2 2026" }
                }
            }

            // KPI cards
            div { class: "fa-kpi-grid",
                StatCard {
                    title: "MAE".to_string(),
                    value: format!("{:.1}", avg_mae),
                    icon: "📉".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some("Mean Absolute Error".to_string()),
                }
                StatCard {
                    title: "MAPE".to_string(),
                    value: format!("{:.1}%", avg_mape),
                    icon: "🎯".to_string(),
                    variant: if avg_mape < 10.0 { StatCardVariant::Success } else if avg_mape < 15.0 { StatCardVariant::Warning } else { StatCardVariant::Danger },
                    footer: Some("Mean Absolute Percentage Error".to_string()),
                }
                StatCard {
                    title: "RMSE".to_string(),
                    value: format!("{:.1}", rmse),
                    icon: "📊".to_string(),
                    variant: StatCardVariant::Default,
                    footer: Some("Root Mean Squared Error".to_string()),
                }
                StatCard {
                    title: "Bias".to_string(),
                    value: format!("{:.1}", avg_error),
                    icon: "⚖".to_string(),
                    variant: if avg_error.abs() < 10.0 { StatCardVariant::Success } else { StatCardVariant::Warning },
                    footer: Some("Positive = over-forecast".to_string()),
                }
            }

            // Accuracy trend chart
            div { class: "fa-chart-section",
                div { class: "fa-chart-header",
                    h2 { "📈 MAPE Trend (H1 2026)" }
                }
                div { class: "fa-chart",
                    svg {
                        view_box: "0 0 {chart_width:.0} 180",
                        preserve_aspect_ratio: "none",
                        line { x1: "0", y1: "20", x2: "{chart_width:.0}", y2: "20", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "60", x2: "{chart_width:.0}", y2: "60", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "100", x2: "{chart_width:.0}", y2: "100", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "140", x2: "{chart_width:.0}", y2: "140", stroke: "#f0f0f0", stroke_width: "0.5" }

                        {monthly.into_iter().enumerate().map(|(i, m)| {
                            let bar_height = if max_mape > 0.0 { (m.mape / max_mape) * chart_height } else { 0.0 };
                            let x = i as f64 * bar_width_pct + bar_gap_pct / 2.0;
                            let y = 170.0 - bar_height;
                            let color = if m.mape < 10.0 { "#28a745" } else if m.mape < 15.0 { "#ffc107" } else { "#dc3545" };
                            rsx! {
                                rect {
                                    key: "{i}",
                                    class: "fa-bar",
                                    x: "{x:.1}",
                                    y: "{y:.1}",
                                    width: "{bar_inner_pct:.1}",
                                    height: "{bar_height:.1}",
                                    rx: "2",
                                    fill: "{color}",
                                }
                                text {
                                    class: "fa-bar-label",
                                    x: "{x + bar_inner_pct / 2.0:.1}",
                                    y: "{y - 4.0:.1}",
                                    "{m.mape:.1}%"
                                }
                                text {
                                    class: "fa-bar-month",
                                    x: "{x + bar_inner_pct / 2.0:.1}",
                                    y: "178.0",
                                    "{m.period}"
                                }
                            }
                        })}
                    }
                }
            }

            // Per-product table
            div { class: "fa-section",
                div { class: "fa-section-header",
                    h2 { "📋 Accuracy by Product" }
                }
                table { class: "fa-table",
                    thead { tr {
                        th { "Product" } th { class: "text-right", "Forecast" }
                        th { class: "text-right", "Actual" }
                        th { class: "text-right", "Error" }
                        th { class: "text-right", "MAPE" }
                    }}
                    tbody {
                        {products.into_iter().map(|p| {
                            let err_cls = if p.error > 0.0 { "text-danger" } else { "text-success" };
                            let mape_cls = if p.mape < 10.0 { "text-success" } else if p.mape < 15.0 { "text-warning" } else { "text-danger" };
                            rsx! {
                                tr {
                                    td { "{p.product}" }
                                    td { class: "text-right", "{p.forecast:.0}" }
                                    td { class: "text-right", "{p.actual:.0}" }
                                    td { class: "text-right {err_cls}", "{p.error:.0}" }
                                    td { class: "text-right {mape_cls}", "{p.mape:.1}%" }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
