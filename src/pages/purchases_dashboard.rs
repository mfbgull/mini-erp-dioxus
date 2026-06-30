//! Purchases Dashboard Page — Overview of purchasing KPIs, quick actions,
//! and recent purchase orders.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant, StatTrend};
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

#[derive(Clone, PartialEq)]
struct RecentPo {
    po_no: String,
    supplier: String,
    date: String,
    amount: String,
    status: String,
    status_class: &'static str,
}

fn po_status_class(status: &str) -> &'static str {
    match status {
        "Draft" => "badge-gray",
        "Sent" => "badge-blue",
        "Confirmed" => "badge-green",
        "Partially Received" => "badge-yellow",
        "Received" => "badge-green",
        "Cancelled" => "badge-red",
        _ => "badge-gray",
    }
}

#[component]
pub fn PurchasesDashboardPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let dashboard_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let orders = client.list_purchase_orders().await.unwrap_or_default();
            orders
        }
    });

    let is_loading = dashboard_resource.read().is_none();
    let orders = dashboard_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let total_purchases: f64 = orders.iter().map(|o| o.total_amount).sum();
    let order_count = orders.len();
    let open_count = orders.iter().filter(|o| matches!(o.status.as_str(), "Draft" | "Sent" | "Confirmed" | "Partially Received")).count();

    let format_pkru = |amount: f64| -> String {
        let formatted = amount as u64;
        let s = formatted.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        let rev: String = result.chars().rev().collect();
        format!("PKR {}", rev)
    };

    let kpis = vec![
        PurchaseKpi {
            title: "Total Purchases".to_string(),
            value: format_pkru(total_purchases),
            icon: "💳".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some("All time".to_string()),
        },
        PurchaseKpi {
            title: "Purchase Orders".to_string(),
            value: order_count.to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some(format!("{} open", open_count)),
        },
        PurchaseKpi {
            title: "Draft Orders".to_string(),
            value: orders.iter().filter(|o| o.status == "Draft").count().to_string(),
            icon: "📥".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: None,
        },
        PurchaseKpi {
            title: "Pending Receipt".to_string(),
            value: orders.iter().filter(|o| o.status == "Partially Received").count().to_string(),
            icon: "📦".to_string(),
            variant: StatCardVariant::Danger,
            trend: None,
            footer: None,
        },
    ];

    let mut pos: Vec<RecentPo> = orders
        .iter()
        .take(5)
        .map(|po| RecentPo {
            po_no: po.po_no.clone(),
            supplier: po.supplier_name.clone().unwrap_or_default(),
            date: po.po_date.clone(),
            amount: format_pkru(po.total_amount),
            status: po.status.clone(),
            status_class: po_status_class(&po.status),
        })
        .collect();
    pos.sort_by(|a, b| b.date.cmp(&a.date));
    pos.truncate(5);

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Purchases Dashboard" }
                    p { class: "page-subtitle", "Overview of purchasing activity, open orders, and receipts." }
                }
            }

            if is_loading {
                div { class: "dashboard-kpi-grid",
                    { (0..4).map(|_| rsx! {
                        StatCard {
                            title: "Loading...".to_string(),
                            value: "--".to_string(),
                            icon: "⏳".to_string(),
                            variant: StatCardVariant::Default,
                        }
                    })}
                }
            } else {
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
