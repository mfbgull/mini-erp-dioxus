//! Forecast Accuracy Page — Model accuracy metrics and per-product accuracy breakdown.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
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
// Mock Data
// ============================================================================

fn monthly_mape() -> Vec<AccuracyMetric> {
    vec![
        AccuracyMetric { period: "Jan".to_string(), mape: 14.2 },
        AccuracyMetric { period: "Feb".to_string(), mape: 12.8 },
        AccuracyMetric { period: "Mar".to_string(), mape: 15.5 },
        AccuracyMetric { period: "Apr".to_string(), mape: 11.0 },
        AccuracyMetric { period: "May".to_string(), mape: 10.2 },
        AccuracyMetric { period: "Jun".to_string(), mape: 11.8 },
    ]
}

fn product_accuracy() -> Vec<ProductAccuracy> {
    vec![
        ProductAccuracy { product: "Premium Widget Alpha".to_string(), forecast: 1550.0, actual: 1620.0, error: -70.0, mape: 4.3 },
        ProductAccuracy { product: "Standard Widget Beta".to_string(), forecast: 2200.0, actual: 1950.0, error: 250.0, mape: 12.8 },
        ProductAccuracy { product: "Steel Rod 12mm".to_string(), forecast: 800.0, actual: 920.0, error: -120.0, mape: 13.0 },
        ProductAccuracy { product: "Hydraulic Pump HPD-200".to_string(), forecast: 45.0, actual: 38.0, error: 7.0, mape: 18.4 },
        ProductAccuracy { product: "Rubber Gasket Set".to_string(), forecast: 300.0, actual: 285.0, error: 15.0, mape: 5.3 },
        ProductAccuracy { product: "LED Panel Light 24W".to_string(), forecast: 500.0, actual: 530.0, error: -30.0, mape: 5.7 },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ForecastAccuracyPage() -> Element {
    let monthly = monthly_mape();
    let products = product_accuracy();
    let max_mape = monthly.iter().map(|m| m.mape).fold(0.0_f64, f64::max);
    let bar_count = monthly.len();
    let bar_width_pct = 100.0 / bar_count as f64;
    let bar_gap_pct = bar_width_pct * 0.25;
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 160.0;

    let avg_mape: f64 = monthly.iter().map(|m| m.mape).sum::<f64>() / monthly.len() as f64;
    let total_abs_error: f64 = products.iter().map(|p| p.error.abs()).sum();
    let avg_error: f64 = products.iter().map(|p| p.error).sum::<f64>() / products.len() as f64;

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
                    value: format!("{:.1}", total_abs_error / products.len() as f64),
                    icon: "📉".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some("Mean Absolute Error".to_string()),
                }
                StatCard {
                    title: "MAPE".to_string(),
                    value: format!("{:.1}%", avg_mape),
                    icon: "🎯".to_string(),
                    variant: if avg_mape < 10.0 { StatCardVariant::Success } else if avg_mape < 15.0 { StatCardVariant::Warning } else { StatCardVariant::Danger },
                    trend: Some(StatTrend { direction: TrendDirection::Down, label: "Improved 2.4%".to_string() }),
                    footer: Some("Mean Absolute Percentage Error".to_string()),
                }
                StatCard {
                    title: "RMSE".to_string(),
                    value: "138.2".to_string(),
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
                        view_box: "0 0 100 180",
                        preserve_aspect_ratio: "xMidYMid meet",
                        line { x1: "0", y1: "20", x2: "100", y2: "20", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "60", x2: "100", y2: "60", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "100", x2: "100", y2: "100", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "140", x2: "100", y2: "140", stroke: "#f0f0f0", stroke_width: "0.5" }

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
