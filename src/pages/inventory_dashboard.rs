//! Inventory Dashboard Page — Overview of inventory health with KPI cards,
//! low stock alerts, and quick actions.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct InventoryKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

fn kpi_data() -> Vec<InventoryKpi> {
    vec![
        InventoryKpi {
            title: "Total Items".to_string(),
            value: "10".to_string(),
            icon: "📦".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some("Across 8 categories".to_string()),
        },
        InventoryKpi {
            title: "Stock Value".to_string(),
            value: "PKR 284,500".to_string(),
            icon: "💰".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "5.2% vs last month".to_string(),
            }),
            footer: Some("At standard cost".to_string()),
        },
        InventoryKpi {
            title: "Low Stock Items".to_string(),
            value: "4".to_string(),
            icon: "⚠".to_string(),
            variant: StatCardVariant::Danger,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "1 more than last week".to_string(),
            }),
            footer: Some("2 items critically low".to_string()),
        },
        InventoryKpi {
            title: "Warehouses".to_string(),
            value: "2".to_string(),
            icon: "🏭".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: Some("Both active".to_string()),
        },
    ]
}

#[component]
pub fn InventoryDashboardPage() -> Element {
    let kpis = kpi_data();
    let navigator = use_navigator();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Inventory Dashboard" }
                    p { class: "page-subtitle", "Overview of your inventory health and stock levels." }
                }
            }

            div { class: "dashboard-kpi-grid",
                {kpis.into_iter().map(|kpi| {
                    rsx! {
                        StatCard {
                            key: "{kpi.title}",
                            title: kpi.title,
                            value: kpi.value,
                            icon: kpi.icon,
                            variant: kpi.variant,
                            trend: kpi.trend,
                            footer: kpi.footer,
                        }
                    }
                })}
            }

            div { class: "dashboard-columns",
                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "📦 Quick Actions" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn toolbar-btn-primary",
                                onclick: move |_| { navigator.push("/inventory/items/new"); },
                                "＋ New Item"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/warehouses/new"); },
                                "＋ New Warehouse"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/stock-movements/new"); },
                                "＋ Stock Movement"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/physical-counts/new"); },
                                "＋ Physical Count"
                            }
                        }
                    }
                }

                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "🔗 Navigation" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/items"); },
                                "📋 Item List"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/warehouses"); },
                                "🏭 Warehouses"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/stock-movements"); },
                                "📊 Stock Movements"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/physical-counts"); },
                                "📝 Physical Counts"
                            }
                        }
                    }
                }
            }
        }
    }
}
