//! Trend Analysis Page — Linear, exponential, and seasonal trend decomposition.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.ta-page { max-width: 1000px; margin: 0 auto; }
.ta-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.ta-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.ta-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.ta-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.ta-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.ta-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.ta-chart-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 20px; }
.ta-chart-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.ta-chart-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.ta-chart { width: 100%; height: 220px; }
.ta-chart svg { width: 100%; height: 100%; }

.ta-dot { fill: var(--accent, #4a90d9); opacity: 0.7; }
.ta-line { fill: none; stroke: #28a745; stroke-width: 2; }

.ta-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.ta-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.ta-table thead th.text-right { text-align: right; }
.ta-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.ta-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.ta-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.ta-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.ta-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.ta-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone)]
struct DecompositionRow {
    period: String,
    actual: f64,
    trend: f64,
    seasonal: f64,
    residual: f64,
}

fn parse_trend_data(val: &serde_json::Value) -> Vec<DecompositionRow> {
    val.as_array().map(|arr| arr.iter().map(|v| DecompositionRow {
        period: v.get("period").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        actual: v.get("actual").and_then(|n| n.as_f64()).unwrap_or(0.0),
        trend: v.get("trend").and_then(|n| n.as_f64()).unwrap_or(0.0),
        seasonal: v.get("seasonal").and_then(|n| n.as_f64()).unwrap_or(0.0),
        residual: v.get("residual").and_then(|n| n.as_f64()).unwrap_or(0.0),
    }).collect()).unwrap_or_default()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn TrendAnalysisPage() -> Element {
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_trend_decomposition().await.unwrap_or_default()
        }
    });
    let raw = resource.read().clone().unwrap_or_default();
    let data = parse_trend_data(&raw);
    let loading = resource.read().is_none();
    let max_val = data.iter().map(|d| d.actual).fold(0.0_f64, f64::max);
    let width = (data.len() as f64 * 40.0).max(500.0);
    let height = 200.0;
    let point_count = data.len();

    // Build SVG polyline points for actual data
    let points_actual: String = data.iter().enumerate().map(|(i, d)| {
        let x = (i as f64 / (point_count - 1) as f64) * width;
        let y = height - ((d.actual / max_val) * height * 0.8 + height * 0.1);
        format!("{:.1},{:.1}", x, y)
    }).collect::<Vec<_>>().join(" ");

    let points_trend: String = data.iter().enumerate().map(|(i, d)| {
        let x = (i as f64 / (point_count - 1) as f64) * width;
        let y = height - ((d.trend / max_val) * height * 0.8 + height * 0.1);
        format!("{:.1},{:.1}", x, y)
    }).collect::<Vec<_>>().join(" ");

    if loading {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page ta-page",
                div { class: "ta-header",
                    div {
                        h1 { "Trend Analysis" }
                        p { class: "page-subtitle", "Loading trend data..." }
                    }
                }
            }
        }
    } else {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page ta-page",

            div { class: "ta-header",
                div {
                    h1 { "Trend Analysis" }
                    p { class: "page-subtitle", "Seasonal decomposition and trend analysis for demand data." }
                }
            }

            // Filters
            div { class: "ta-filter-bar",
                label { "Product / Category" }
                select { style: "min-width: 220px;",
                    option { value: "alpha", selected: true, "Premium Widget Alpha (ITM-0001)" }
                    option { value: "widgets", "Widgets Category" }
                    option { value: "all", "All Products" }
                }
                label { "Trend Type" }
                select {
                    option { value: "linear", selected: true, "Linear" }
                    option { value: "exponential", "Exponential" }
                    option { value: "seasonal", "Seasonal" }
                }
            }

            // KPI cards
            div { class: "ta-kpi-grid",
                StatCard {
                    title: "Trend Strength".to_string(),
                    value: "0.92".to_string(),
                    icon: "📈".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some("Strong upward trend (0-1 scale)".to_string()),
                }
                StatCard {
                    title: "Seasonal Strength".to_string(),
                    value: "0.68".to_string(),
                    icon: "🔄".to_string(),
                    variant: StatCardVariant::Warning,
                    footer: Some("Moderate seasonality detected".to_string()),
                }
                StatCard {
                    title: "R".to_string(),
                    value: "0.94".to_string(),
                    icon: "📊".to_string(),
                    variant: StatCardVariant::Success,
                    footer: Some("R-squared of trend fit".to_string()),
                }
                StatCard {
                    title: "Growth Rate".to_string(),
                    value: "+3.2% / mo".to_string(),
                    icon: "📈".to_string(),
                    variant: StatCardVariant::Default,
                    footer: Some("Linear trend slope".to_string()),
                }
            }

            // Chart
            div { class: "ta-chart-section",
                div { class: "ta-chart-header",
                    h2 { "📊 Actual vs Trend" }
                    div { style: "display: flex; gap: 16px;",
                        span { style: "font-size: 10px; display: flex; align-items: center; gap: 4px;", span { style: "display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: var(--accent, #4a90d9);" }, "Actual" }
                        span { style: "font-size: 10px; display: flex; align-items: center; gap: 4px;", span { style: "display: inline-block; width: 16px; height: 3px; background: #28a745;" }, "Trend" }
                    }
                }
                div { class: "ta-chart",
                    svg {
                        view_box: "0 0 {width:.0} 220",
                        preserve_aspect_ratio: "none",
                        line { x1: "0", y1: "20", x2: "{width:.0}", y2: "20", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "60", x2: "{width:.0}", y2: "60", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "100", x2: "{width:.0}", y2: "100", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "140", x2: "{width:.0}", y2: "140", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "180", x2: "{width:.0}", y2: "180", stroke: "#f0f0f0", stroke_width: "0.5" }

                        // Actual data points
                        polyline { points: "{points_actual}", fill: "none", stroke: "#4a90d9", stroke_width: "1.5", opacity: "0.6" }
                        // Trend line
                        polyline { points: "{points_trend}", fill: "none", stroke: "#28a745", stroke_width: "2.5" }

                        // X-axis labels
                        {data.iter().enumerate().filter(|(i, _)| i % 2 == 0).map(|(i, d)| {
                            let x = (i as f64 / (point_count - 1) as f64) * width;
                            rsx! {
                                text {
                                    key: "{i}",
                                    x: "{x:.1}", y: "210",
                                    font_size: "9", fill: "#6c757d", text_anchor: "middle",
                                    "{d.period}"
                                }
                            }
                        })}
                    }
                }
            }

            // Decomposition table
            div { class: "ta-section",
                div { class: "ta-section-header",
                    h2 { "📋 Seasonality Decomposition" }
                }
                table { class: "ta-table",
                    thead { tr {
                        th { "Period" } th { class: "text-right", "Actual" }
                        th { class: "text-right", "Trend" }
                        th { class: "text-right", "Seasonal" }
                        th { class: "text-right", "Residual" }
                    }}
                    tbody {
                        {data.into_iter().map(|d| {
                            rsx! {
                                tr {
                                    td { "{d.period}" }
                                    td { class: "text-right", "{d.actual:.0}" }
                                    td { class: "text-right", "{d.trend:.0}" }
                                    td { class: "text-right", "{d.seasonal:.0}" }
                                    td { class: "text-right", "{d.residual:.0}" }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
}
