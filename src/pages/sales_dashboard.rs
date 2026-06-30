//! Sales Dashboard Page — Overview of sales performance with KPI cards,
//! quick actions, navigation links, and recent invoices.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant, StatTrend};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.sales-page { max-width: 1000px; margin: 0 auto; }

.sales-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.sales-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.sales-month-label { font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 4px 12px; border-radius: 6px; }

.sales-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; margin-bottom: 20px; }

.sales-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px; }

.sales-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.sales-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sales-section-header h2 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }

.sales-actions { display: flex; flex-direction: column; gap: 8px; }
.sales-actions button { width: 100%; }

.sales-recent-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.sales-recent-table thead th { text-align: left; padding: 6px 8px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.sales-recent-table thead th.text-right { text-align: right; }
.sales-recent-table tbody td { padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.sales-recent-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.sales-recent-table tbody tr:last-child td { border-bottom: none; }
.sales-recent-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.sales-badge { display: inline-flex; align-items: center; padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; }
.sales-badge-green { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.sales-badge-yellow { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.sales-badge-blue { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.sales-badge-red { background: rgba(220, 53, 69, 0.12); color: #dc3545; }

@media (max-width: 768px) {
    .sales-columns { grid-template-columns: 1fr; }
    .sales-kpi-grid { grid-template-columns: 1fr 1fr; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, PartialEq)]
struct SalesKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

#[derive(Clone, Debug)]
struct RecentInvoice {
    invoice_no: String,
    customer: String,
    date: String,
    status: String,
    amount: f64,
}

fn format_pkru(amount: f64) -> String {
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
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Paid" => "sales-badge sales-badge-green",
        "Unpaid" => "sales-badge sales-badge-yellow",
        "Partially Paid" => "sales-badge sales-badge-blue",
        "Overdue" => "sales-badge sales-badge-red",
        _ => "sales-badge sales-badge-blue",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesDashboardPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let dashboard_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let dashboard = client.get_sales_dashboard().await.unwrap_or_default();
            let invoices = client.list_invoices().await.unwrap_or_default();
            (dashboard, invoices)
        }
    });

    let is_loading = dashboard_resource.read().is_none();
    let (dashboard, invoices_list) = dashboard_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| (serde_json::Value::Null, vec![]));

    let total_revenue = dashboard["total_revenue"].as_f64().unwrap_or(0.0);
    let total_invoices = dashboard["total_invoices"].as_i64().unwrap_or(0);
    let draft_quotations = dashboard["draft_quotations"].as_i64().unwrap_or(0);
    let pending_sales_orders = dashboard["pending_sales_orders"].as_i64().unwrap_or(0);

    let unpaid_count = invoices_list.iter().filter(|i| matches!(i.status.as_str(), "Unpaid" | "Partially Paid")).count();
    let overdue_count = invoices_list.iter().filter(|i| i.status == "Overdue").count();

    let kpis = vec![
        SalesKpi {
            title: "Total Revenue".to_string(),
            value: format_pkru(total_revenue),
            icon: "💰".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some("All time".to_string()),
        },
        SalesKpi {
            title: "Invoices".to_string(),
            value: total_invoices.to_string(),
            icon: "🧾".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some(format!("{} unpaid / {} overdue", unpaid_count, overdue_count)),
        },
        SalesKpi {
            title: "Draft Quotations".to_string(),
            value: draft_quotations.to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: None,
        },
        SalesKpi {
            title: "Pending Sales Orders".to_string(),
            value: pending_sales_orders.to_string(),
            icon: "📦".to_string(),
            variant: StatCardVariant::Warning,
            trend: None,
            footer: None,
        },
    ];

    let mut recent: Vec<RecentInvoice> = invoices_list
        .iter()
        .take(5)
        .map(|inv| RecentInvoice {
            invoice_no: inv.invoice_no.clone(),
            customer: inv.customer_name.clone().unwrap_or_default(),
            date: inv.invoice_date.clone(),
            status: inv.status.clone(),
            amount: inv.total_amount,
        })
        .collect();
    recent.sort_by(|a, b| b.date.cmp(&a.date));
    recent.truncate(5);

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page sales-page",

            div { class: "sales-header",
                div {
                    h1 { "Sales Dashboard" }
                    p { class: "page-subtitle", "Overview of your sales performance and pipeline." }
                }
                span { class: "sales-month-label", "📅 {chrono::Utc::now().format(\"%B %Y\")}" }
            }

            if is_loading {
                div { class: "sales-kpi-grid",
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
                div { class: "sales-kpi-grid",
                    {kpis.iter().map(|kpi| {
                        rsx! {
                            StatCard {
                                key: "{kpi.title}",
                                title: kpi.title.clone(),
                                value: kpi.value.clone(),
                                icon: kpi.icon.clone(),
                                variant: kpi.variant,
                                trend: kpi.trend.clone(),
                                footer: kpi.footer.clone(),
                            }
                        }
                    })}
                }
            }

            div { class: "sales-columns",

                // Quick Actions
                div { class: "sales-section",
                    div { class: "sales-section-header",
                        h2 { "⚡ Quick Actions" }
                    }
                    div { class: "sales-actions",
                        button {
                            class: "toolbar-btn toolbar-btn-primary",
                            onclick: move |_| { navigator.push("/sales/invoices/new"); },
                            "＋ New Invoice"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/quotations/new"); },
                            "＋ New Quotation"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/orders/new"); },
                            "＋ New Sales Order"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/pos"); },
                            "🛒 POS Terminal"
                        }
                    }
                }

                // Navigation
                div { class: "sales-section",
                    div { class: "sales-section-header",
                        h2 { "🔗 Navigation" }
                    }
                    div { class: "sales-actions",
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/invoices"); },
                            "📋 Invoice List"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/quotations"); },
                            "📄 Quotations"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/orders"); },
                            "📦 Sales Orders"
                        }
                        button {
                            class: "toolbar-btn",
                            onclick: move |_| { navigator.push("/sales/returns"); },
                            "↩ Sales Returns"
                        }
                    }
                }
            }

            // Recent Invoices
            div { class: "sales-section",
                div { class: "sales-section-header",
                    h2 { "🧾 Recent Invoices" }
                    button {
                        class: "toolbar-btn",
                        style: "font-size: 12px; padding: 4px 10px;",
                        onclick: move |_| { navigator.push("/sales/invoices"); },
                        "View All →"
                    }
                }
                table { class: "sales-recent-table",
                    thead {
                        tr {
                            th { "Invoice #" }
                            th { "Customer" }
                            th { "Date" }
                            th { "Status" }
                            th { class: "text-right", "Amount" }
                        }
                    }
                    tbody {
                        {recent.iter().map(|inv| {
                            let badge_cls = status_badge_class(&inv.status);
                            rsx! {
                                tr {
                                    td { style: "font-family: monospace;", "{inv.invoice_no}" }
                                    td { "{inv.customer}" }
                                    td { "{inv.date}" }
                                    td { span { class: "{badge_cls}", "{inv.status}" } }
                                    td { class: "text-right", "PKR {inv.amount:.0}" }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
