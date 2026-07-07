//! Accounting Dashboard Page — Overview of accounting health and quick links.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant, StatTrend};
use crate::models::AccountBalance;
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

#[derive(Clone, Debug)]
struct RecentTransaction {
    date: String,
    ref_no: String,
    description: String,
    amount: f64,
    type_label: String,
}

#[component]
pub fn AccountingDashboardPage() -> Element {
    let api = use_auth().api;
    let navigator = use_navigator();

    let ar_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_ar_summary().await.ok()
        }
    });

    let balances_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_account_balances().await.ok().unwrap_or_default()
        }
    });

    let is_loading = ar_resource.read().is_none() || balances_resource.read().is_none();
    let ar_data = ar_resource.read().as_ref().cloned().flatten();
    let balances: Vec<AccountBalance> = balances_resource.read().as_ref().cloned().unwrap_or_default();

    let total_assets: f64 = balances.iter()
        .filter(|b| matches!(b.account_type.as_str(), "Asset" | "asset" | "Current Asset" | "Fixed Asset"))
        .map(|b| b.debit - b.credit)
        .sum();
    let total_liabilities: f64 = balances.iter()
        .filter(|b| matches!(b.account_type.as_str(), "Liability" | "liability" | "Current Liability" | "Long-term Liability"))
        .map(|b| b.credit - b.debit)
        .sum();
    let total_equity: f64 = balances.iter()
        .filter(|b| matches!(b.account_type.as_str(), "Equity" | "equity" | "Owner's Equity" | "Retained Earnings"))
        .map(|b| b.credit - b.debit)
        .sum();

    let ar_total = ar_data
        .as_ref()
        .and_then(|d| d.get("total_receivable"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let kpis = vec![
        AccountingKpi {
            title: "Total Assets".to_string(),
            value: format!("PKR {:.0}", total_assets),
            icon: "🏦".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some("Current + Fixed Assets".to_string()),
        },
        AccountingKpi {
            title: "Liabilities".to_string(),
            value: format!("PKR {:.0}", total_liabilities),
            icon: "📋".to_string(),
            variant: StatCardVariant::Warning,
            footer: Some("Accounts payable + Loans".to_string()),
            trend: None,
        },
        AccountingKpi {
            title: "Equity".to_string(),
            value: format!("PKR {:.0}", total_equity),
            icon: "📈".to_string(),
            variant: StatCardVariant::Success,
            footer: Some("Owner's equity + Reserves".to_string()),
            trend: None,
        },
        AccountingKpi {
            title: "Accounts Receivable".to_string(),
            value: format!("PKR {:.0}", ar_total),
            icon: "💰".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some("Outstanding receivables".to_string()),
        },
    ];

    let transactions: Vec<RecentTransaction> = ar_data
        .as_ref()
        .and_then(|d| d.get("recent_transactions"))
        .and_then(|v| v.as_array())
        .map(|v| {
            v.iter()
                .take(5)
                .map(|t| RecentTransaction {
                    date: t.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    ref_no: t.get("reference").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: t.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    amount: t.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    type_label: t.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Accounting Dashboard" }
                    p { class: "page-subtitle", "Overview of your financial position and accounting activities." }
                }
            }

            if is_loading {
                div { class: "loading-text", "Loading accounting data…" }
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
}
