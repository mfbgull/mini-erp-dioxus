//! Forecasts Dashboard Page — Overview of forecast models, KPIs, and navigation.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.fc-page { max-width: 1000px; margin: 0 auto; }
.fc-header { margin-bottom: 16px; }
.fc-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.fc-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.fc-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px; }
@media (max-width: 768px) { .fc-columns { grid-template-columns: 1fr; } }

.fc-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.fc-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.fc-section-header h2 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }

.fc-actions { display: flex; flex-direction: column; gap: 8px; }
.fc-actions button { width: 100%; }

.fc-nav-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 14px; margin-bottom: 20px; }

.fc-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 18px; cursor: pointer; transition: all 0.15s ease; display: flex; flex-direction: column; gap: 8px; }
.fc-card:hover { border-color: var(--accent, #4a90d9); box-shadow: 0 2px 8px rgba(74,144,217,0.12); }
.fc-card-icon { font-size: 28px; line-height: 1; }
.fc-card-title { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }
.fc-card-desc { font-size: 12px; color: var(--text-secondary); line-height: 1.5; margin: 0; }
.fc-card-arrow { font-size: 16px; color: var(--accent, #4a90d9); align-self: flex-end; margin-top: auto; }

.fc-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; background: rgba(40, 167, 69, 0.1); color: #28a745; }
.fc-status-badge-warning { background: rgba(255, 193, 7, 0.15); color: #d4a017; }

.loading-text { text-align: center; padding: 40px; color: var(--text-secondary); font-size: 13px; }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, PartialEq)]
struct ForecastNavItem {
    icon: String,
    title: String,
    description: String,
    route: &'static str,
}

fn nav_items() -> Vec<ForecastNavItem> {
    vec![
        ForecastNavItem {
            icon: "📈".to_string(),
            title: "Demand Forecast".to_string(),
            description: "View historical and forecasted demand for products with confidence intervals.".to_string(),
            route: "/forecasts/demand",
        },
        ForecastNavItem {
            icon: "📊".to_string(),
            title: "Trend Analysis".to_string(),
            description: "Analyze linear, exponential, and seasonal trends in your data.".to_string(),
            route: "/forecasts/trends",
        },
        ForecastNavItem {
            icon: "🎯".to_string(),
            title: "Forecast Accuracy".to_string(),
            description: "Track MAE, MAPE, RMSE, and bias across forecasting models.".to_string(),
            route: "/forecasts/accuracy",
        },
        ForecastNavItem {
            icon: "⚙".to_string(),
            title: "Model Configuration".to_string(),
            description: "Configure ARIMA, ETS, Prophet, and Neural forecasting models.".to_string(),
            route: "/forecasts/model-config",
        },
        ForecastNavItem {
            icon: "🗓".to_string(),
            title: "Seasonal Events".to_string(),
            description: "Manage seasonal events and their impact factors for forecast adjustment.".to_string(),
            route: "/forecasts/seasonal-events",
        },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ForecastsDashboardPage() -> Element {
    let api = use_auth().api;
    let navigator = use_navigator();
    let items = nav_items();

    let forecasts_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_forecasts().await.ok()
        }
    });

    let runs_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_forecast_runs().await.ok()
        }
    });

    let accuracy_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_forecast_accuracy().await.ok()
        }
    });

    let configs_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_forecast_configs().await.ok()
        }
    });

    let is_loading = forecasts_resource.read().is_none();
    let forecasts_data = forecasts_resource.read().as_ref().cloned().flatten();
    let runs_data = runs_resource.read().as_ref().cloned().flatten();
    let accuracy_data = accuracy_resource.read().as_ref().cloned().flatten();
    let configs_data = configs_resource.read().as_ref().cloned().flatten();

    let active_models = configs_data.as_ref().map(|v| v.len()).unwrap_or(0);
    let model_breakdown = configs_data
        .as_ref()
        .map(|v| {
            let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for c in v {
                let model_type = c.get("model_type").and_then(|m| m.as_str()).unwrap_or("Unknown").to_string();
                *counts.entry(model_type).or_insert(0) += 1;
            }
            counts
                .iter()
                .map(|(k, v)| format!("{} {}", v, k))
                .collect::<Vec<_>>()
                .join(" • ")
        })
        .unwrap_or_default();

    let avg_accuracy = accuracy_data
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| {
            let sum: f64 = v.iter()
                .filter_map(|a| a.get("accuracy").or(a.get("mape")).and_then(|m| m.as_f64()))
                .sum();
            let count = v.iter()
                .filter(|a| a.get("accuracy").or(a.get("mape")).is_some())
                .count();
            if count > 0 { sum / count as f64 } else { 0.0 }
        })
        .unwrap_or(0.0);

    let latest_run = runs_data
        .as_ref()
        .filter(|v| !v.is_empty())
        .and_then(|v| v.first())
        .and_then(|r| r.get("created_at").or(r.get("run_date")).and_then(|d| d.as_str()))
        .unwrap_or("N/A");

    let total_forecasts = forecasts_data.as_ref().map(|v| v.len()).unwrap_or(0);
    let total_runs = runs_data.as_ref().map(|v| v.len()).unwrap_or(0);
    let total_accuracy_records = accuracy_data.as_ref().map(|v| v.len()).unwrap_or(0);
    let accuracy_footer = format!("Across {} accuracy records", total_accuracy_records);
    let runs_footer = format!("{} total runs", total_runs);
    let kpi_model_footer = if model_breakdown.is_empty() { "No models configured".to_string() } else { model_breakdown };

    let model_status_items: Vec<String> = configs_data
        .as_ref()
        .map(|v| {
            v.iter()
                .map(|c| {
                    let name = c.get("model_type").and_then(|m| m.as_str()).unwrap_or("Unknown");
                    let last_trained = c.get("last_trained").or(c.get("updated_at")).and_then(|d| d.as_str()).unwrap_or("Never");
                    let mape = c.get("mape").and_then(|m| m.as_f64()).map(|m| format!("{:.1}%", m)).unwrap_or_else(|| "N/A".to_string());
                    format!("• {} — Last trained: {} — MAPE: {}", name, last_trained, mape)
                })
                .collect()
        })
        .unwrap_or_default();

    let all_healthy = !model_status_items.is_empty();
    let accuracy_value = if avg_accuracy > 0.0 { format!("{:.1}%", avg_accuracy) } else { "N/A".to_string() };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page fc-page",

            div { class: "fc-header",
                h1 { "Forecasts Dashboard" }
                p { class: "page-subtitle", "Demand forecasting, trend analysis, and model management." }
            }

            if is_loading {
                div { class: "loading-text", "Loading forecast data…" }
            } else {
                div { class: "fc-kpi-grid",
                    StatCard {
                        title: "Active Models".to_string(),
                        value: active_models.to_string(),
                        icon: Some("🧠".to_string()),
                        variant: StatCardVariant::Primary,
                        footer: Some(kpi_model_footer),
                    }
                    StatCard {
                        title: "Last Forecast Accuracy".to_string(),
                        value: accuracy_value,
                        icon: Some("🎯".to_string()),
                        variant: StatCardVariant::Success,
                        footer: Some(accuracy_footer),
                    }
                    StatCard {
                        title: "Last Forecast Run".to_string(),
                        value: latest_run.to_string(),
                        icon: Some("🕐".to_string()),
                        variant: StatCardVariant::Default,
                        footer: Some(runs_footer),
                    }
                    StatCard {
                        title: "Forecast Items".to_string(),
                        value: total_forecasts.to_string(),
                        icon: Some("📊".to_string()),
                        variant: StatCardVariant::Default,
                        footer: Some("Products with forecasts".to_string()),
                    }
                }

                div { class: "fc-nav-grid",
                    {items.into_iter().map(|item| {
                        let nav = navigator.clone();
                        let route = item.route;
                        rsx! {
                            div {
                                key: "{item.title}",
                                class: "fc-card",
                                onclick: move |_| { nav.push(route); },
                                div { class: "fc-card-icon", "{item.icon}" }
                                h3 { class: "fc-card-title", "{item.title}" }
                                p { class: "fc-card-desc", "{item.description}" }
                                div { class: "fc-card-arrow", "→" }
                            }
                        }
                    })}
                }

                div { class: "fc-columns",
                    div { class: "fc-section",
                        div { class: "fc-section-header",
                            h2 { "⚡ Quick Actions" }
                        }
                        div { class: "fc-actions",
                            button { class: "toolbar-btn toolbar-btn-primary", r#type: "button",
                                onclick: move |_| { navigator.push("/forecasts/demand"); },
                                "📈 Run All Forecasts"
                            }
                            button { class: "toolbar-btn", r#type: "button",
                                onclick: move |_| { navigator.push("/forecasts/model-config"); },
                                "🔄 Update Models"
                            }
                            button { class: "toolbar-btn", r#type: "button",
                                onclick: move |_| { navigator.push("/forecasts/accuracy"); },
                                "📊 View Accuracy Report"
                            }
                        }
                    }

                    div { class: "fc-section",
                        div { class: "fc-section-header",
                            h2 { "🔗 Related" }
                        }
                        div { class: "fc-actions",
                            button { class: "toolbar-btn", r#type: "button",
                                onclick: move |_| { navigator.push("/reports/sales"); },
                                "📈 Sales Report"
                            }
                            button { class: "toolbar-btn", r#type: "button",
                                onclick: move |_| { navigator.push("/inventory/items"); },
                                "📦 Inventory Items"
                            }
                        }
                    }
                }

                div { class: "fc-section",
                    div { class: "fc-section-header",
                        h2 { "🤖 Model Status" }
                        if all_healthy {
                            span { class: "fc-status-badge", "🟢 All models healthy" }
                        } else {
                            span { class: "fc-status-badge fc-status-badge-warning", "⚠ No models configured" }
                        }
                    }
                    div { style: "font-size: 13px; color: var(--text-secondary); line-height: 1.8;",
                        if model_status_items.is_empty() {
                            p { "No model configurations found. Configure models to start forecasting." }
                        } else {
                            {model_status_items.iter().map(|item| {
                                rsx! { p { "{item}" } }
                            })}
                        }
                    }
                }
            }
        }
    }
}
