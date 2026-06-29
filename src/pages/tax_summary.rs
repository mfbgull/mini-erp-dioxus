//! Tax Summary Page — Sales Tax, Income Tax, and Withholding Tax summaries by period.

use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, use_toast};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.tx-page { max-width: 1000px; margin: 0 auto; }
.tx-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.tx-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.tx-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.tx-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.tx-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.tx-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.tx-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 2px solid var(--border-color, #e0e0e0); }
.tx-tab { padding: 10px 20px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border: none; background: none; white-space: nowrap; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.15s ease; }
.tx-tab:hover { color: var(--text-primary); background: rgba(74, 144, 217, 0.04); }
.tx-tab-active { color: var(--accent, #4a90d9); border-bottom-color: var(--accent, #4a90d9); font-weight: 600; }

.tx-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.tx-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.tx-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.tx-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.tx-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.tx-table thead th.text-right { text-align: right; }
.tx-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.tx-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.tx-table tbody td.text-success { color: #28a745; }
.tx-table tbody td.text-danger { color: #dc3545; }
.tx-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.tx-table tfoot td { padding: 8px 10px; font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); }
.tx-table tfoot td.text-right { text-align: right; font-family: monospace; }

.tx-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }

@media (max-width: 768px) {
    .tx-kpi-grid { grid-template-columns: 1fr 1fr; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone)]
struct TaxPeriodRow {
    period: String,
    tax_base: f64,
    rate: f64,
    tax_amount: f64,
    paid_amount: f64,
    balance: f64,
}

// ============================================================================
// Mock Data
// ============================================================================

fn sales_tax_data() -> Vec<TaxPeriodRow> {
    vec![
        TaxPeriodRow { period: "Jan 2026".to_string(), tax_base: 185_000.0, rate: 15.0, tax_amount: 27_750.0, paid_amount: 27_750.0, balance: 0.0 },
        TaxPeriodRow { period: "Feb 2026".to_string(), tax_base: 220_000.0, rate: 15.0, tax_amount: 33_000.0, paid_amount: 33_000.0, balance: 0.0 },
        TaxPeriodRow { period: "Mar 2026".to_string(), tax_base: 195_000.0, rate: 15.0, tax_amount: 29_250.0, paid_amount: 20_000.0, balance: 9_250.0 },
        TaxPeriodRow { period: "Apr 2026".to_string(), tax_base: 278_000.0, rate: 15.0, tax_amount: 41_700.0, paid_amount: 41_700.0, balance: 0.0 },
        TaxPeriodRow { period: "May 2026".to_string(), tax_base: 312_000.0, rate: 15.0, tax_amount: 46_800.0, paid_amount: 46_800.0, balance: 0.0 },
        TaxPeriodRow { period: "Jun 2026".to_string(), tax_base: 289_500.0, rate: 15.0, tax_amount: 43_425.0, paid_amount: 0.0, balance: 43_425.0 },
    ]
}

fn income_tax_data() -> Vec<TaxPeriodRow> {
    vec![
        TaxPeriodRow { period: "Jan 2026".to_string(), tax_base: 485_000.0, rate: 29.0, tax_amount: 140_650.0, paid_amount: 35_000.0, balance: 105_650.0 },
        TaxPeriodRow { period: "Feb 2026".to_string(), tax_base: 520_000.0, rate: 29.0, tax_amount: 150_800.0, paid_amount: 35_000.0, balance: 115_800.0 },
        TaxPeriodRow { period: "Mar 2026".to_string(), tax_base: 495_000.0, rate: 29.0, tax_amount: 143_550.0, paid_amount: 35_000.0, balance: 108_550.0 },
        TaxPeriodRow { period: "Apr 2026".to_string(), tax_base: 578_000.0, rate: 29.0, tax_amount: 167_620.0, paid_amount: 35_000.0, balance: 132_620.0 },
        TaxPeriodRow { period: "May 2026".to_string(), tax_base: 612_000.0, rate: 29.0, tax_amount: 177_480.0, paid_amount: 35_000.0, balance: 142_480.0 },
        TaxPeriodRow { period: "Jun 2026".to_string(), tax_base: 589_500.0, rate: 29.0, tax_amount: 170_955.0, paid_amount: 0.0, balance: 170_955.0 },
    ]
}

fn withholding_tax_data() -> Vec<TaxPeriodRow> {
    vec![
        TaxPeriodRow { period: "Jan 2026".to_string(), tax_base: 120_000.0, rate: 5.0, tax_amount: 6_000.0, paid_amount: 6_000.0, balance: 0.0 },
        TaxPeriodRow { period: "Feb 2026".to_string(), tax_base: 145_000.0, rate: 5.0, tax_amount: 7_250.0, paid_amount: 7_250.0, balance: 0.0 },
        TaxPeriodRow { period: "Mar 2026".to_string(), tax_base: 98_000.0, rate: 5.0, tax_amount: 4_900.0, paid_amount: 4_900.0, balance: 0.0 },
        TaxPeriodRow { period: "Apr 2026".to_string(), tax_base: 165_000.0, rate: 5.0, tax_amount: 8_250.0, paid_amount: 8_250.0, balance: 0.0 },
        TaxPeriodRow { period: "May 2026".to_string(), tax_base: 180_000.0, rate: 5.0, tax_amount: 9_000.0, paid_amount: 9_000.0, balance: 0.0 },
        TaxPeriodRow { period: "Jun 2026".to_string(), tax_base: 135_000.0, rate: 5.0, tax_amount: 6_750.0, paid_amount: 0.0, balance: 6_750.0 },
    ]
}

fn total_row(data: &[TaxPeriodRow]) -> TaxPeriodRow {
    TaxPeriodRow {
        period: "Total".to_string(),
        tax_base: data.iter().map(|r| r.tax_base).sum(),
        rate: 0.0,
        tax_amount: data.iter().map(|r| r.tax_amount).sum(),
        paid_amount: data.iter().map(|r| r.paid_amount).sum(),
        balance: data.iter().map(|r| r.balance).sum(),
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn TaxSummaryPage() -> Element {
    let toast = use_toast();
    let active_tab = use_signal(|| 0usize);
    let tabs = ["Sales Tax", "Income Tax", "Withholding Tax"];

    let sales = sales_tax_data();
    let income = income_tax_data();
    let withholding = withholding_tax_data();

    let st_total = total_row(&sales);
    let it_total = total_row(&income);
    let wt_total = total_row(&withholding);

    let all_tax_liability = st_total.tax_amount + it_total.tax_amount + wt_total.tax_amount;
    let all_paid = st_total.paid_amount + it_total.paid_amount + wt_total.paid_amount;
    let all_outstanding = st_total.balance + it_total.balance + wt_total.balance;

    let current_data = move || -> Vec<TaxPeriodRow> {
        match *active_tab.read() {
            0 => sales.clone(),
            1 => income.clone(),
            2 => withholding.clone(),
            _ => sales.clone(),
        }
    };

    let current_total = move || -> TaxPeriodRow {
        match *active_tab.read() {
            0 => st_total.clone(),
            1 => it_total.clone(),
            2 => wt_total.clone(),
            _ => st_total.clone(),
        }
    };

    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Tax summary will be exported as PDF."); }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page tx-page",

            div { class: "tx-header",
                div {
                    h1 { "Tax Summary" }
                    p { class: "page-subtitle", "Sales Tax, Income Tax, and Withholding Tax summaries." }
                }
                Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export Report" }
            }

            // Filter
            div { class: "tx-filter-bar",
                label { "Period" }
                select {
                    option { value: "monthly", selected: true, "Monthly" }
                    option { value: "quarterly", "Quarterly" }
                    option { value: "yearly", "Yearly" }
                }
                label { "Year" }
                select {
                    option { value: "2026", selected: true, "2026" }
                    option { value: "2025", "2025" }
                }
            }

            // KPI cards
            div { class: "tx-kpi-grid",
                StatCard {
                    title: "Total Tax Liability".to_string(),
                    value: format!("PKR {:.0}", all_tax_liability),
                    icon: "🧾".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some("All tax types".to_string()),
                }
                StatCard {
                    title: "Total Paid".to_string(),
                    value: format!("PKR {:.0}", all_paid),
                    icon: "✅".to_string(),
                    variant: StatCardVariant::Success,
                    footer: Some(format!("{:.1}% paid", if all_tax_liability > 0.0 { (all_paid / all_tax_liability) * 100.0 } else { 0.0 })),
                }
                StatCard {
                    title: "Outstanding".to_string(),
                    value: format!("PKR {:.0}", all_outstanding),
                    icon: "⚠".to_string(),
                    variant: if all_outstanding > 500_000.0 { StatCardVariant::Danger } else { StatCardVariant::Warning },
                    footer: Some("Total unpaid".to_string()),
                }
            }

            // Tax type tabs
            div { class: "tx-tabs",
                {tabs.iter().enumerate().map(|(i, tab)| {
                    let is_active = *active_tab.read() == i;
                    let cls = if is_active { "tx-tab tx-tab-active" } else { "tx-tab" };
                    let mut set_tab = active_tab.clone();
                    rsx! {
                        button { key: "{i}", class: "{cls}", r#type: "button",
                            onclick: move |_| { set_tab.set(i); },
                            "{tab}"
                        }
                    }
                })}
            }

            // Tax data table
            div { class: "tx-section",
                div { class: "tx-section-header",
                    h2 { "{tabs[*active_tab.read()]} — H1 2026" }
                }
                table { class: "tx-table",
                    thead { tr {
                        th { "Period" } th { class: "text-right", "Tax Base (PKR)" }
                        th { class: "text-right", "Rate (%)" }
                        th { class: "text-right", "Tax Amount (PKR)" }
                        th { class: "text-right", "Paid (PKR)" }
                        th { class: "text-right", "Balance (PKR)" }
                    }}
                    tbody {
                        {current_data().iter().map(|r| {
                            let bal_cls = if r.balance > 0.0 { "text-danger" } else { "text-success" };
                            rsx! {
                                tr {
                                    td { "{r.period}" }
                                    td { class: "text-right", "PKR {r.tax_base:.0}" }
                                    td { class: "text-right", "{r.rate:.0}%" }
                                    td { class: "text-right", "PKR {r.tax_amount:.0}" }
                                    td { class: "text-right", "PKR {r.paid_amount:.0}" }
                                    td { class: "text-right {bal_cls}", "PKR {r.balance:.0}" }
                                }
                            }
                        })}
                    }
                    tfoot {
                        tr {
                            td { "{current_total().period}" }
                            td { class: "text-right", "PKR {current_total().tax_base:.0}" }
                            td { class: "text-right", "—" }
                            td { class: "text-right", "PKR {current_total().tax_amount:.0}" }
                            td { class: "text-right", "PKR {current_total().paid_amount:.0}" }
                            td { class: "text-right", "PKR {current_total().balance:.0}" }
                        }
                    }
                }
            }
        }
    }
}
