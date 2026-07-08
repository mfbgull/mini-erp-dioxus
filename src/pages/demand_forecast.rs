//! Demand Forecast Page — Historical and forecasted demand with confidence intervals.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.df-page { max-width: 1000px; margin: 0 auto; }
.df-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.df-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.df-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.df-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.df-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.df-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.df-chart-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 20px; }
.df-chart-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.df-chart-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.df-chart { width: 100%; height: 220px; }
.df-chart svg { width: 100%; height: 100%; }

.df-bar { fill: var(--accent, #4a90d9); transition: fill 0.15s ease; }
.df-bar:hover { fill: #357abd; }
.df-bar-forecast { fill: #28a745; }
.df-bar-forecast:hover { fill: #1e7e34; }
.df-bar-bound { stroke: #dc3545; stroke_width: 0.5; stroke_dasharray: "2,2"; }

.df-chart-label { font-size: 9px; fill: var(--text-secondary, #6c757d); text-anchor: middle; }
.df-chart-legend { font-size: 10px; }
.df-legend-dot { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 4px; }

.df-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.df-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.df-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.df-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.df-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.df-table thead th.text-right { text-align: right; }
.df-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.df-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.df-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.df-table tbody tr.forecast-row td { background: rgba(40, 167, 69, 0.04); }
.df-table tfoot td { padding: 8px 10px; font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); }
.df-table tfoot td.text-right { text-align: right; font-family: monospace; }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
struct DemandPeriod {
    period: String,
    historical: Option<f64>,
    forecasted: Option<f64>,
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
}

fn parse_timeline(data: &serde_json::Value) -> Vec<DemandPeriod> {
    data.as_array().map(|arr| arr.iter().map(|item| DemandPeriod {
        period: item.get("period").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        historical: item.get("historical").and_then(|v| v.as_f64()),
        forecasted: item.get("forecasted").and_then(|v| v.as_f64()),
        lower_bound: item.get("lower_bound").and_then(|v| v.as_f64()),
        upper_bound: item.get("upper_bound").and_then(|v| v.as_f64()),
    }).collect()).unwrap_or_default()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn DemandForecastPage() -> Element {
    let api = use_auth().api;

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_demand_timeline().await.unwrap_or_default()
        }
    });

    let loading = resource.read().is_none();
    let data_val = resource.read().clone().unwrap_or_default();
    let data = parse_timeline(&data_val);

    if loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page df-page",
                div { class: "df-header",
                    div {
                        h1 { "Demand Forecast" }
                        p { class: "page-subtitle", "Loading demand forecast data..." }
                    }
                }
                div { "Loading..." }
            }
        };
    }

    let max_val = data.iter()
        .map(|d| d.historical.unwrap_or(0.0).max(d.upper_bound.unwrap_or(0.0)))
        .fold(0.0_f64, f64::max);

    let total_hist: f64 = data.iter().filter_map(|d| d.historical).sum();
    let total_fc: f64 = data.iter().filter_map(|d| d.forecasted).sum();
    let hist_count = data.iter().filter(|d| d.historical.is_some()).count();
    let fc_count = data.iter().filter(|d| d.forecasted.is_some()).count();
    let bar_count = data.len();
    let chart_width = (bar_count as f64 * 25.0).max(500.0);
    let bar_width_pct = if bar_count > 0 { chart_width / bar_count as f64 } else { chart_width };
    let bar_gap_pct = bar_width_pct * 0.2;
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 200.0;
    let sep_x = if bar_count > 0 && hist_count > 0 {
        (hist_count as f64 * bar_width_pct + bar_width_pct / 2.0).max(bar_width_pct)
    } else { chart_width / 2.0 };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page df-page",

            div { class: "df-header",
                div {
                    h1 { "Demand Forecast" }
                    p { class: "page-subtitle", "Historical demand and forecasted quantities" }
                }
            }

            // KPI cards
            div { class: "df-kpi-grid",
                StatCard {
                    title: "Historical Total".to_string(),
                    value: format!("{:.0} units", total_hist),
                    icon: "📊".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some(format!("{} periods", hist_count)),
                }
                StatCard {
                    title: "Forecast Total".to_string(),
                    value: format!("{:.0} units", total_fc),
                    icon: "📦".to_string(),
                    variant: StatCardVariant::Success,
                    footer: Some(format!("{} future periods", fc_count)),
                }
            }

            // Chart
            div { class: "df-chart-section",
                div { class: "df-chart-header",
                    h2 { "📈 Demand — Historical & Forecast" }
                    div { style: "display: flex; gap: 16px;",
                        span { class: "df-chart-legend", span { class: "df-legend-dot", style: "background: var(--accent, #4a90d9);" }, "Historical" }
                        span { class: "df-chart-legend", span { class: "df-legend-dot", style: "background: #28a745; opacity: 0.8;" }, "Forecast" }
                        span { class: "df-chart-legend", span { class: "df-legend-dot", style: "background: #dc3545; opacity: 0.3;" }, "Confidence" }
                    }
                }
                div { class: "df-chart",
                    svg {
                        view_box: "0 0 {chart_width:.0} 230",
                        preserve_aspect_ratio: "none",

                        line { x1: "0", y1: "30", x2: "{chart_width:.0}", y2: "30", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "70", x2: "{chart_width:.0}", y2: "70", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "110", x2: "{chart_width:.0}", y2: "110", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "150", x2: "{chart_width:.0}", y2: "150", stroke: "#f0f0f0", stroke_width: "0.5" }
                        line { x1: "0", y1: "190", x2: "{chart_width:.0}", y2: "190", stroke: "#f0f0f0", stroke_width: "0.5" }

                        line { x1: "{sep_x:.1}", y1: "20", x2: "{sep_x:.1}", y2: "210", stroke: "#adb5bd", stroke_width: "1", stroke_dasharray: "4,3" }

                        {data.clone().into_iter().enumerate().map(|(i, d)| {
                            let x = i as f64 * bar_width_pct + bar_gap_pct / 2.0;
                            let bar_class = if d.historical.is_some() { "df-bar" } else { "df-bar df-bar-forecast" };
                            let bar_val = d.historical.or(d.forecasted).unwrap_or(0.0);
                            let bar_height = if max_val > 0.0 { (bar_val / max_val) * chart_height } else { 0.0 };

                            rsx! {
                                if let Some(ub) = d.upper_bound {
                                    line {
                                        x1: "{x:.1}",
                                        y1: "{210.0 - (ub / max_val.max(1.0) * chart_height):.1}",
                                        x2: "{x + bar_inner_pct:.1}",
                                        y2: "{210.0 - (ub / max_val.max(1.0) * chart_height):.1}",
                                        stroke: "#dc3545", stroke_width: "0.5", stroke_dasharray: "2,2"
                                    }
                                }
                                if let Some(lb) = d.lower_bound {
                                    line {
                                        x1: "{x:.1}",
                                        y1: "{210.0 - (lb / max_val.max(1.0) * chart_height):.1}",
                                        x2: "{x + bar_inner_pct:.1}",
                                        y2: "{210.0 - (lb / max_val.max(1.0) * chart_height):.1}",
                                        stroke: "#dc3545", stroke_width: "0.5", stroke_dasharray: "2,2"
                                    }
                                }
                                rect {
                                    class: "{bar_class}",
                                    x: "{x:.1}",
                                    y: "{210.0 - bar_height:.1}",
                                    width: "{bar_inner_pct:.1}",
                                    height: "{bar_height:.1}",
                                    rx: "2",
                                }
                                text {
                                    class: "df-chart-label",
                                    x: "{x + bar_inner_pct / 2.0:.1}",
                                    y: "220.0",
                                    "{d.period}"
                                }
                            }
                        })}
                    }
                }
            }

            // Data table
            div { class: "df-section",
                div { class: "df-section-header",
                    h2 { "📋 Forecast Data" }
                }
                table { class: "df-table",
                    thead { tr {
                        th { "Period" } th { class: "text-right", "Historical" }
                        th { class: "text-right", "Forecasted" }
                        th { class: "text-right", "Lower Bound" }
                        th { class: "text-right", "Upper Bound" }
                    }}
                    tbody {
                        {data.iter().map(|d| {
                            let row_cls = if d.historical.is_none() { "forecast-row" } else { "" };
                            rsx! {
                                tr { key: "{d.period}", class: "{row_cls}",
                                    td { "{d.period}" }
                                    td { class: "text-right",
                                        if let Some(h) = d.historical { "{h:.0}" } else { "—" }
                                    }
                                    td { class: "text-right",
                                        if let Some(f) = d.forecasted { "{f:.0}" } else { "—" }
                                    }
                                    td { class: "text-right",
                                        if let Some(l) = d.lower_bound { "{l:.0}" } else { "—" }
                                    }
                                    td { class: "text-right",
                                        if let Some(u) = d.upper_bound { "{u:.0}" } else { "—" }
                                    }
                                }
                            }
                        })}
                    }
                    tfoot { tr {
                        td { "Total" }
                        td { class: "text-right", "{total_hist:.0}" }
                        td { class: "text-right", "{total_fc:.0}" }
                        td { class: "text-right", "—" }
                        td { class: "text-right", "—" }
                    }}
                }
            }
        }
    }
}
