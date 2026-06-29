//! Forecasts Dashboard Page — Overview of forecast models, KPIs, and navigation.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
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
    let navigator = use_navigator();
    let items = nav_items();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page fc-page",

            div { class: "fc-header",
                h1 { "Forecasts Dashboard" }
                p { class: "page-subtitle", "Demand forecasting, trend analysis, and model management." }
            }

            // KPI cards
            div { class: "fc-kpi-grid",
                StatCard {
                    title: "Active Models".to_string(),
                    value: "6".to_string(),
                    icon: "🧠".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some("3 ARIMA • 2 Prophet • 1 ETS".to_string()),
                }
                StatCard {
                    title: "Last Forecast Accuracy".to_string(),
                    value: "87.3%".to_string(),
                    icon: Some("🎯".to_string()),
                    variant: StatCardVariant::Success,
                    trend: Some(StatTrend { direction: TrendDirection::Up, label: "2.1% vs last run".to_string() }),
                    footer: Some("MAPE: 12.7%".to_string()),
                }
                StatCard {
                    title: "Next Scheduled Run".to_string(),
                    value: "Tonight 02:00".to_string(),
                    icon: Some("🕐".to_string()),
                    variant: StatCardVariant::Default,
                    footer: Some("Daily automatic retrain".to_string()),
                }
                StatCard {
                    title: "Data Points".to_string(),
                    value: "2,400".to_string(),
                    icon: Some("📊".to_string()),
                    variant: StatCardVariant::Default,
                    footer: Some("Across 50 products".to_string()),
                }
            }

            // Navigation grid
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

            // Quick actions
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

            // Status indicator
            div { class: "fc-section",
                div { class: "fc-section-header",
                    h2 { "🤖 Model Status" }
                    span { class: "fc-status-badge", "🟢 All models healthy" }
                }
                div { style: "font-size: 13px; color: var(--text-secondary); line-height: 1.8;",
                    p { "• ARIMA(2,1,2) — Last trained: Jun 26, 2026 02:00 — MAPE: 11.2%" }
                    p { "• Prophet — Last trained: Jun 26, 2026 02:00 — MAPE: 13.8%" }
                    p { "• ETS(A,A,A) — Last trained: Jun 25, 2026 02:00 — MAPE: 14.5%" }
                    p { "• Neural (LSTM) — Last trained: Jun 24, 2026 02:00 — MAPE: 9.7%" }
                }
            }
        }
    }
}
