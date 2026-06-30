//! Financial Report Page — Profit & Loss and Balance Sheet with period comparison.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, StatTrend, TrendDirection, use_toast};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.fr-page { max-width: 1000px; margin: 0 auto; }
.fr-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.fr-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.fr-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.fr-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.fr-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.fr-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.fr-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 2px solid var(--border-color, #e0e0e0); }
.fr-tab { padding: 10px 20px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border: none; background: none; white-space: nowrap; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.15s ease; }
.fr-tab:hover { color: var(--text-primary); background: rgba(74, 144, 217, 0.04); }
.fr-tab-active { color: var(--accent, #4a90d9); border-bottom-color: var(--accent, #4a90d9); font-weight: 600; }

.fr-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.fr-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.fr-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.fr-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.fr-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.fr-table thead th.text-right { text-align: right; }
.fr-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.fr-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.fr-table tbody td.text-success { color: #28a745; }
.fr-table tbody td.text-danger { color: #dc3545; }
.fr-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.fr-table tbody tr.total-row td { font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); background: #f8f9fa; }
.fr-table tbody tr.section-header td { font-weight: 600; color: var(--text-secondary); background: #f8f9fa; font-size: 12px; text-transform: uppercase; letter-spacing: 0.3px; }

.fr-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }

@media (max-width: 768px) {
    .fr-kpi-grid { grid-template-columns: 1fr 1fr; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone)]
struct PnlLine {
    label: String,
    amount: f64,
    is_header: bool,
    is_total: bool,
}

// ============================================================================
// Helpers — parse API JSON into view structs
// ============================================================================

fn parse_pnl_lines(data: &serde_json::Value) -> Vec<PnlLine> {
    let items = data.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    items.iter().map(|item| {
        PnlLine {
            label: item.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            amount: item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
            is_header: item.get("is_header").and_then(|v| v.as_bool()).unwrap_or(false),
            is_total: item.get("is_total").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }).collect()
}

fn parse_balance_lines(data: &serde_json::Value) -> Vec<PnlLine> {
    let items = data.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    items.iter().map(|item| {
        PnlLine {
            label: item.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            amount: item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
            is_header: item.get("is_header").and_then(|v| v.as_bool()).unwrap_or(false),
            is_total: item.get("is_total").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }).collect()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn FinancialReportPage() -> Element {
    let toast = use_toast();
    let api = use_auth().api;
    let active_tab = use_signal(|| 0usize);
    let tabs = ["Profit & Loss", "Balance Sheet"];

    let pnl_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_profit_loss().await.unwrap_or_default()
        }
    });

    let bs_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_balance_sheet().await.unwrap_or_default()
        }
    });

    let loading = pnl_resource.read().is_none() || bs_resource.read().is_none();

    let pnl_data = pnl_resource.read().clone().unwrap_or_default();
    let bs_data = bs_resource.read().clone().unwrap_or_default();

    let income = parse_pnl_lines(&pnl_data);
    let balance = parse_balance_lines(&bs_data);

    let net_profit = pnl_data.get("net_profit").and_then(|v| v.as_f64()).unwrap_or(
        income.last().map(|l| l.amount).unwrap_or(0.0)
    );
    let total_revenue = pnl_data.get("total_revenue").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_assets = bs_data.get("total_assets").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_liabilities = bs_data.get("total_liabilities").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_equity = bs_data.get("total_equity").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let profit_margin = if total_revenue > 0.0 { (net_profit / total_revenue) * 100.0 } else { 0.0 };

    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Financial report will be exported as PDF."); }
    };

    let on_print = {
        let mut t = toast.clone();
        move |_| { t.info("Print", "Print dialog will open."); }
    };

    if loading {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page fr-page",
                div { class: "fr-header",
                    div {
                        h1 { "Financial Report" }
                        p { class: "page-subtitle", "Profit & Loss Statement and Balance Sheet." }
                    }
                }
                div { class: "fr-loading", "Loading financial data..." }
            }
        }
    } else {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page fr-page",

                div { class: "fr-header",
                    div {
                        h1 { "Financial Report" }
                        p { class: "page-subtitle", "Profit & Loss Statement and Balance Sheet." }
                    }
                    div { style: "display: flex; gap: 8px;",
                        Button { variant: ButtonVariant::Secondary, icon: Some("🖨".to_string()), onclick: on_print, "Print" }
                        Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export PDF" }
                    }
                }

                // Period filter
                div { class: "fr-filter-bar",
                    label { "Period" }
                    select {
                        option { value: "h1-2026", selected: true, "H1 2026 (Jan — Jun)" }
                        option { value: "q2-2026", "Q2 2026 (Apr — Jun)" }
                        option { value: "2025", "FY 2025" }
                    }
                    label { "Comparison" }
                    select {
                        option { value: "none", selected: true, "No Comparison" }
                        option { value: "prev", "vs Previous Period" }
                    }
                }

                // KPI cards
                div { class: "fr-kpi-grid",
                    StatCard {
                        title: "Net Profit".to_string(),
                        value: format!("PKR {:.0}", net_profit),
                        icon: "📈".to_string(),
                        variant: if net_profit > 0.0 { StatCardVariant::Success } else { StatCardVariant::Danger },
                        trend: Some(StatTrend { direction: TrendDirection::Up, label: "15.2% vs last period".to_string() }),
                        footer: Some(format!("Margin: {:.1}%", profit_margin)),
                    }
                    StatCard {
                        title: "Total Revenue".to_string(),
                        value: format!("PKR {:.0}", total_revenue),
                        icon: "💰".to_string(),
                        variant: StatCardVariant::Primary,
                        footer: Some("H1 2026".to_string()),
                    }
                    StatCard {
                        title: "Total Assets".to_string(),
                        value: format!("PKR {:.0}", total_assets),
                        icon: "🏢".to_string(),
                        variant: StatCardVariant::Default,
                        footer: Some("As of current period".to_string()),
                    }
                    StatCard {
                        title: "Liabilities / Equity".to_string(),
                        value: format!("PKR {:.0} / PKR {:.0}", total_liabilities, total_equity),
                        icon: "⚖".to_string(),
                        variant: StatCardVariant::Warning,
                        footer: Some(if total_equity > 0.0 { format!("Ratio: {:.2}", total_liabilities / total_equity) } else { "Ratio: N/A".to_string() }),
                    }
                }

                // Tabs
                div { class: "fr-tabs",
                    {tabs.iter().enumerate().map(|(i, tab)| {
                        let is_active = *active_tab.read() == i;
                        let cls = if is_active { "fr-tab fr-tab-active" } else { "fr-tab" };
                        let mut set_tab = active_tab.clone();
                        rsx! {
                            button { key: "{i}", class: "{cls}", r#type: "button",
                                onclick: move |_| { set_tab.set(i); },
                                "{tab}"
                            }
                        }
                    })}
                }

                // P&L tab
                if *active_tab.read() == 0 {
                    div { class: "fr-section",
                        div { class: "fr-section-header",
                            h2 { "Profit & Loss Statement — H1 2026" }
                        }
                        table { class: "fr-table",
                            thead { tr {
                                th { style: "width: 60%;", "Account" }
                                th { class: "text-right", "Amount (PKR)" }
                            }}
                            tbody {
                                {income.into_iter().map(|line| {
                                    if line.is_header {
                                        rsx! { tr { key: "{line.label}", class: "section-header",
                                            td { colspan: "2", "{line.label}" }
                                        }}
                                    } else if line.is_total {
                                        let cls = if line.label == "Net Profit / (Loss)" { "text-success" } else { "" };
                                        rsx! { tr { key: "{line.label}", class: "total-row",
                                            td { "{line.label}" }
                                            td { class: "text-right {cls}", "PKR {line.amount:.0}" }
                                        }}
                                    } else {
                                        let indent = if line.label.starts_with("  ") { "padding-left: 24px;" } else { "" };
                                        rsx! { tr { key: "{line.label}",
                                            td { style: "{indent}", "{line.label}" }
                                            td { class: "text-right", "PKR {line.amount:.0}" }
                                        }}
                                    }
                                })}
                            }
                        }
                    }
                }

                // Balance Sheet tab
                if *active_tab.read() == 1 {
                    div { class: "fr-section",
                        div { class: "fr-section-header",
                            h2 { "Balance Sheet — Current Period" }
                        }
                        table { class: "fr-table",
                            thead { tr {
                                th { style: "width: 60%;", "Account" }
                                th { class: "text-right", "Amount (PKR)" }
                            }}
                            tbody {
                                {balance.into_iter().map(|line| {
                                    if line.is_header {
                                        rsx! { tr { key: "{line.label}", class: "section-header",
                                            td { colspan: "2", "{line.label}" }
                                        }}
                                    } else if line.is_total {
                                        rsx! { tr { key: "{line.label}", class: "total-row",
                                            td { "{line.label}" }
                                            td { class: "text-right", "PKR {line.amount:.0}" }
                                        }}
                                    } else {
                                        rsx! { tr { key: "{line.label}",
                                            td { "{line.label}" }
                                            td { class: "text-right", "PKR {line.amount:.0}" }
                                        }}
                                    }
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}
