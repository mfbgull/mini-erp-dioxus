//! Accounting Dashboard Page — Overview of accounting health and quick links.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct AccountingKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

fn kpi_data() -> Vec<AccountingKpi> {
    vec![
        AccountingKpi {
            title: "Total Assets".to_string(),
            value: "PKR 5,200,000".to_string(),
            icon: "🏦".to_string(),
            variant: StatCardVariant::Primary,
            trend: Some(StatTrend { direction: TrendDirection::Up, label: "2.1% vs last month".to_string() }),
            footer: Some("Current + Fixed Assets".to_string()),
        },
        AccountingKpi {
            title: "Liabilities".to_string(),
            value: "PKR 1,800,000".to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Warning,
            trend: Some(StatTrend { direction: TrendDirection::Down, label: "1.5% decrease".to_string() }),
            footer: Some("Accounts payable + Loans".to_string()),
        },
        AccountingKpi {
            title: "Equity".to_string(),
            value: "PKR 3,400,000".to_string(),
            icon: "📈".to_string(),
            variant: StatCardVariant::Success,
            footer: Some("Owner's equity + Reserves".to_string()),
            trend: None,
        },
        AccountingKpi {
            title: "Net Income".to_string(),
            value: "PKR 890,000".to_string(),
            icon: "💰".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend { direction: TrendDirection::Up, label: "12.3% vs last quarter".to_string() }),
            footer: Some("Year to date (FY 2026)".to_string()),
        },
    ]
}

#[derive(Clone, Debug)]
struct RecentTransaction {
    date: String,
    ref_no: String,
    description: String,
    amount: f64,
    type_label: String,
}

fn recent_transactions() -> Vec<RecentTransaction> {
    vec![
        RecentTransaction { date: "2025-06-25".to_string(), ref_no: "JV-2026-0045".to_string(), description: "Sales revenue recognition".to_string(), amount: 345_000.00, type_label: "Journal".to_string() },
        RecentTransaction { date: "2025-06-24".to_string(), ref_no: "PAY-2026-0033".to_string(), description: "Rent payment - June".to_string(), amount: 200_000.00, type_label: "Payment".to_string() },
        RecentTransaction { date: "2025-06-22".to_string(), ref_no: "INV-2026-0045".to_string(), description: "Alpha Traders - Widgets".to_string(), amount: 156_000.00, type_label: "Invoice".to_string() },
        RecentTransaction { date: "2025-06-20".to_string(), ref_no: "PO-2026-0045".to_string(), description: "Raw material purchase".to_string(), amount: 320_000.00, type_label: "Purchase".to_string() },
        RecentTransaction { date: "2025-06-18".to_string(), ref_no: "JV-2026-0044".to_string(), description: "Depreciation - Equipment".to_string(), amount: 45_000.00, type_label: "Journal".to_string() },
    ]
}

#[component]
pub fn AccountingDashboardPage() -> Element {
    let kpis = kpi_data();
    let transactions = recent_transactions();
    let navigator = use_navigator();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Accounting Dashboard" }
                    p { class: "page-subtitle", "Overview of your financial position and accounting activities." }
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
                            button { class: "toolbar-btn toolbar-btn-primary", onclick: move |_| { navigator.push("/accounting/chart-of-accounts"); }, "📋 Chart of Accounts" }
                            button { class: "toolbar-btn", onclick: move |_| { navigator.push("/accounting/journal/new"); }, "📝 Journal Entry" }
                            button { class: "toolbar-btn", onclick: move |_| { navigator.push("/accounting/trial-balance"); }, "📊 Trial Balance" }
                        }
                    }
                }

                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "🔗 Navigation" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button { class: "toolbar-btn", onclick: move |_| { navigator.push("/accounting/chart-of-accounts"); }, "📋 Chart of Accounts" }
                            button { class: "toolbar-btn", onclick: move |_| { navigator.push("/accounting/periods"); }, "📅 Accounting Periods" }
                        }
                    }
                }
            }

            div { class: "dashboard-section", style: "margin-top: 16px;",
                div { class: "dashboard-section-header",
                    h2 { "📄 Recent Transactions" }
                }
                div { class: "dashboard-section-body",
                    if transactions.is_empty() {
                        div { class: "customer-table-empty", "No recent transactions." }
                    } else {
                        table { class: "customer-table",
                            thead { tr {
                                th { "Date" } th { "Reference" } th { "Description" }
                                th { class: "text-right", "Amount" } th { "Type" }
                            }}
                            tbody {
                                {transactions.iter().map(|t| {
                                    rsx! { tr {
                                        td { "{t.date}" }
                                        td { style: "font-family: monospace;", "{t.ref_no}" }
                                        td { "{t.description}" }
                                        td { class: "text-right", "PKR {t.amount:.0}" }
                                        td { span { class: "customer-table-badge customer-table-badge-blue", "{t.type_label}" } }
                                    }}
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}
