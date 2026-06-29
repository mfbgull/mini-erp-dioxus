//! Purchases Dashboard Page — Overview of purchasing KPIs, quick actions,
//! and recent purchase orders.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct PurchaseKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

fn kpi_data() -> Vec<PurchaseKpi> {
    vec![
        PurchaseKpi {
            title: "Total Purchases".to_string(),
            value: "PKR 890,000".to_string(),
            icon: "💳".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some("This month".to_string()),
        },
        PurchaseKpi {
            title: "Purchase Orders".to_string(),
            value: "15".to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "3 new this week".to_string(),
            }),
            footer: Some("Open orders".to_string()),
        },
        PurchaseKpi {
            title: "Direct Purchases".to_string(),
            value: "8".to_string(),
            icon: "📥".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: Some("This month".to_string()),
        },
        PurchaseKpi {
            title: "Goods Receipts Pending".to_string(),
            value: "3".to_string(),
            icon: "📦".to_string(),
            variant: StatCardVariant::Danger,
            trend: Some(StatTrend {
                direction: TrendDirection::Down,
                label: "2 overdue".to_string(),
            }),
            footer: Some("Awaiting delivery".to_string()),
        },
    ]
}

#[derive(Clone, PartialEq)]
struct RecentPo {
    po_no: String,
    supplier: String,
    date: String,
    amount: String,
    status: String,
    status_class: &'static str,
}

fn recent_pos() -> Vec<RecentPo> {
    vec![
        RecentPo { po_no: "PO-2026-0021".to_string(), supplier: "SteelMart Industries".to_string(), date: "2026-06-25".to_string(), amount: "PKR 156,000".to_string(), status: "Sent".to_string(), status_class: "badge-blue" },
        RecentPo { po_no: "PO-2026-0020".to_string(), supplier: "Pak Hardware Supplies".to_string(), date: "2026-06-24".to_string(), amount: "PKR 43,200".to_string(), status: "Confirmed".to_string(), status_class: "badge-green" },
        RecentPo { po_no: "PO-2026-0019".to_string(), supplier: "Rawal Electricals".to_string(), date: "2026-06-22".to_string(), amount: "PKR 98,765".to_string(), status: "Partially Received".to_string(), status_class: "badge-yellow" },
        RecentPo { po_no: "PO-2026-0018".to_string(), supplier: "United Traders Lahore".to_string(), date: "2026-06-20".to_string(), amount: "PKR 234,500".to_string(), status: "Draft".to_string(), status_class: "badge-gray" },
        RecentPo { po_no: "PO-2026-0017".to_string(), supplier: "ChemiCorp Pakistan".to_string(), date: "2026-06-18".to_string(), amount: "PKR 67,890".to_string(), status: "Received".to_string(), status_class: "badge-green" },
    ]
}

#[component]
pub fn PurchasesDashboardPage() -> Element {
    let kpis = kpi_data();
    let pos = recent_pos();
    let navigator = use_navigator();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Purchases Dashboard" }
                    p { class: "page-subtitle", "Overview of purchasing activity, open orders, and receipts." }
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
                        h2 { "⚡ Quick Actions" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn toolbar-btn-primary",
                                onclick: move |_| { navigator.push("/purchases/orders/new"); },
                                "＋ New Purchase Order"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/purchases/direct/new"); },
                                "＋ New Direct Purchase"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/purchases/receipts"); },
                                "📦 Record Goods Receipt"
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
                                onclick: move |_| { navigator.push("/purchases/orders"); },
                                "📋 Purchase Orders"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/purchases/direct"); },
                                "📥 Direct Purchases"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/purchases/receipts"); },
                                "📦 Goods Receipts"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/purchases/returns"); },
                                "↩ Purchase Returns"
                            }
                        }
                    }
                }
            }

            // ── Recent Purchase Orders Table ──
            div { class: "dashboard-section", style: "margin-top: 20px;",
                div { class: "dashboard-section-header",
                    h2 { "📋 Recent Purchase Orders" }
                    button {
                        class: "toolbar-btn",
                        onclick: move |_| { navigator.push("/purchases/orders"); },
                        "View All →"
                    }
                }
                div { class: "dashboard-section-body", style: "padding: 0;",
                    table { class: "customer-table",
                        thead {
                            tr {
                                th { "PO #" }
                                th { "Supplier" }
                                th { "Date" }
                                th { "Amount" }
                                th { "Status" }
                            }
                        }
                        tbody {
                            {pos.into_iter().map(|po| {
                                rsx! {
                                    tr {
                                        td { style: "font-family: monospace;", "{po.po_no}" }
                                        td { "{po.supplier}" }
                                        td { "{po.date}" }
                                        td { style: "font-family: monospace;", "{po.amount}" }
                                        td { span { class: "customer-table-badge {po.status_class}", "{po.status}" } }
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
