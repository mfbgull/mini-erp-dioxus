//! AR Aging Report Page — Outstanding receivables by aging buckets.

use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, use_toast};
use crate::pages::print_shared::trigger_print;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.ar-page { max-width: 1100px; margin: 0 auto; }
.ar-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.ar-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.ar-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.ar-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.ar-filter-bar input[type="date"] { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.ar-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.ar-bucket-header { display: flex; align-items: center; gap: 10px; margin: 20px 0 10px 0; padding: 10px 16px; background: #f8f9fa; border-radius: 6px; border-left: 4px solid var(--accent, #4a90d9); }
.ar-bucket-header h3 { font-size: 14px; font-weight: 600; margin: 0; color: var(--text-primary); }
.ar-bucket-header span { font-size: 12px; color: var(--text-secondary); }

.ar-table { width: 100%; border-collapse: collapse; font-size: 13px; margin-bottom: 12px; }
.ar-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.ar-table thead th.text-right { text-align: right; }
.ar-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.ar-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.ar-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.ar-table tfoot td { padding: 10px; font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); font-size: 13px; }
.ar-table tfoot td.text-right { text-align: right; font-family: monospace; }

.ar-bucket-danger { border-left-color: #dc3545; }
.ar-bucket-warning { border-left-color: #ffc107; }
.ar-bucket-info { border-left-color: #17a2b8; }
.ar-bucket-default { border-left-color: #6c757d; }

.text-total { color: var(--text-primary); font-weight: 700; }
.text-danger { color: #dc3545; }
.text-warning { color: #d4a017; }

.ar-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }

@media (max-width: 768px) {
    .ar-kpi-grid { grid-template-columns: 1fr 1fr; }
    .ar-table { font-size: 12px; }
    .ar-table thead th, .ar-table tbody td { padding: 6px; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
struct AgingCustomer {
    customer_name: String,
    total_balance: f64,
    current: f64,
    bucket_0_30: f64,
    bucket_31_60: f64,
    bucket_61_90: f64,
    bucket_90_plus: f64,
}

// ============================================================================
// Mock Data
// ============================================================================

fn aging_data() -> Vec<AgingCustomer> {
    vec![
        AgingCustomer { customer_name: "Alpha Traders".to_string(), total_balance: 342_000.00, current: 120_000.00, bucket_0_30: 156_000.00, bucket_31_60: 66_000.00, bucket_61_90: 0.0, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Beta Industries".to_string(), total_balance: 89_500.00, current: 0.0, bucket_0_30: 0.0, bucket_31_60: 45_000.00, bucket_61_90: 44_500.00, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Gamma Supplies".to_string(), total_balance: 234_500.00, current: 98_000.00, bucket_0_30: 136_500.00, bucket_31_60: 0.0, bucket_61_90: 0.0, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Delta Corp".to_string(), total_balance: 412_000.00, current: 200_000.00, bucket_0_30: 112_000.00, bucket_31_60: 100_000.00, bucket_61_90: 0.0, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Epsilon LLC".to_string(), total_balance: 67_500.00, current: 0.0, bucket_0_30: 0.0, bucket_31_60: 0.0, bucket_61_90: 0.0, bucket_90_plus: 67_500.00 },
        AgingCustomer { customer_name: "Zeta Enterprises".to_string(), total_balance: 12_450.00, current: 12_450.00, bucket_0_30: 0.0, bucket_31_60: 0.0, bucket_61_90: 0.0, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Eta Group".to_string(), total_balance: 178_200.00, current: 50_000.00, bucket_0_30: 78_200.00, bucket_31_60: 50_000.00, bucket_61_90: 0.0, bucket_90_plus: 0.0 },
        AgingCustomer { customer_name: "Theta Corp".to_string(), total_balance: 95_000.00, current: 0.0, bucket_0_30: 0.0, bucket_31_60: 0.0, bucket_61_90: 95_000.00, bucket_90_plus: 0.0 },
    ]
}

// ============================================================================
// Helpers
// ============================================================================

fn bucket_class(bucket: &str) -> &'static str {
    match bucket {
        "90+" => "ar-bucket-danger",
        "61-90" => "ar-bucket-warning",
        "31-60" => "ar-bucket-info",
        _ => "ar-bucket-default",
    }
}

fn sum_by<F>(customers: &[AgingCustomer], f: F) -> f64
where
    F: Fn(&AgingCustomer) -> f64,
{
    customers.iter().map(|c| f(c)).sum()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ArAgingReportPage() -> Element {
    let toast = use_toast();
    let customers = aging_data();

    let total_receivables: f64 = sum_by(&customers, |c| c.total_balance);
    let total_current: f64 = sum_by(&customers, |c| c.current);
    let total_0_30: f64 = sum_by(&customers, |c| c.bucket_0_30);
    let total_31_60: f64 = sum_by(&customers, |c| c.bucket_31_60);
    let total_61_90: f64 = sum_by(&customers, |c| c.bucket_61_90);
    let total_90_plus: f64 = sum_by(&customers, |c| c.bucket_90_plus);
    let overdue_total = total_0_30 + total_31_60 + total_61_90 + total_90_plus;

    let current_customers: Vec<&AgingCustomer> = customers.iter().filter(|c| c.current > 0.0).collect();
    let bucket_0_30_customers: Vec<&AgingCustomer> = customers.iter().filter(|c| c.bucket_0_30 > 0.0).collect();
    let bucket_31_60_customers: Vec<&AgingCustomer> = customers.iter().filter(|c| c.bucket_31_60 > 0.0).collect();
    let bucket_61_90_customers: Vec<&AgingCustomer> = customers.iter().filter(|c| c.bucket_61_90 > 0.0).collect();
    let bucket_90_plus_customers: Vec<&AgingCustomer> = customers.iter().filter(|c| c.bucket_90_plus > 0.0).collect();

    let on_export = {
        let mut toast = toast.clone();
        move |_| {
            toast.info("Export", "AR Aging report will be exported as PDF/Excel.");
        }
    };
    let on_export2 = on_export.clone();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page ar-page",

            div { class: "ar-header",
                div {
                    h1 { "AR Aging Report" }
                    p { class: "page-subtitle", "Accounts Receivable aging as of June 27, 2026" }
                }
                Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export Report" }
            }

            // Filter bar
            div { class: "ar-filter-bar",
                label { "As of Date" }
                input { r#type: "date", value: "2026-06-27" }
                label { "Customer" }
                input { r#type: "text", placeholder: "All customers…", style: "border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff;" }
            }

            // KPI cards
            div { class: "ar-kpi-grid",
                StatCard {
                    title: "Total Receivables".to_string(),
                    value: format!("PKR {:.0}", total_receivables),
                    icon: "💰".to_string(),
                    variant: StatCardVariant::Primary,
                    footer: Some(format!("Current: PKR {:.0}", total_current)),
                }
                StatCard {
                    title: "Overdue Total".to_string(),
                    value: format!("PKR {:.0}", overdue_total),
                    icon: "⚠".to_string(),
                    variant: if overdue_total > 500_000.0 { StatCardVariant::Danger } else { StatCardVariant::Warning },
                    footer: Some(format!("{} of {} customers", customers.len() - current_customers.len(), customers.len())),
                }
                StatCard {
                    title: "90+ Days".to_string(),
                    value: format!("PKR {:.0}", total_90_plus),
                    icon: "🔴".to_string(),
                    variant: if total_90_plus > 0.0 { StatCardVariant::Danger } else { StatCardVariant::Success },
                    footer: Some(format!("{} customer(s)", bucket_90_plus_customers.len())),
                }
                StatCard {
                    title: "Avg Days Outstanding".to_string(),
                    value: "38".to_string(),
                    icon: "📅".to_string(),
                    variant: StatCardVariant::Default,
                    footer: Some("DSO — Days Sales Outstanding".to_string()),
                }
            }

            // ── Current bucket ──
            if !current_customers.is_empty() {
                div { class: "ar-bucket-header ar-bucket-default",
                    h3 { "🟢 Current" }
                    span { "— PKR {total_current:.0}" }
                }
                table { class: "ar-table",
                    thead { tr {
                        th { "Customer" } th { class: "text-right", "Total Balance" }
                        th { class: "text-right", "Current" }
                    }}
                    tbody {
                        {current_customers.iter().map(|c| rsx! {
                            tr {
                                td { "{c.customer_name}" }
                                td { class: "text-right", "PKR {c.total_balance:.0}" }
                                td { class: "text-right", "PKR {c.current:.0}" }
                            }
                        })}
                    }
                }
            }

            // ── 0-30 Days bucket ──
            if !bucket_0_30_customers.is_empty() {
                div { class: "ar-bucket-header ar-bucket-default",
                    h3 { "🔵 0–30 Days" }
                    span { "— PKR {total_0_30:.0}" }
                }
                table { class: "ar-table",
                    thead { tr {
                        th { "Customer" } th { class: "text-right", "Total Balance" }
                        th { class: "text-right", "0–30 Days" }
                    }}
                    tbody {
                        {bucket_0_30_customers.iter().map(|c| rsx! {
                            tr {
                                td { "{c.customer_name}" }
                                td { class: "text-right", "PKR {c.total_balance:.0}" }
                                td { class: "text-right", "PKR {c.bucket_0_30:.0}" }
                            }
                        })}
                    }
                }
            }

            // ── 31-60 Days bucket ──
            if !bucket_31_60_customers.is_empty() {
                div { class: "ar-bucket-header ar-bucket-info",
                    h3 { "🟡 31–60 Days" }
                    span { "— PKR {total_31_60:.0}" }
                }
                table { class: "ar-table",
                    thead { tr {
                        th { "Customer" } th { class: "text-right", "Total Balance" }
                        th { class: "text-right", "31–60 Days" }
                    }}
                    tbody {
                        {bucket_31_60_customers.iter().map(|c| rsx! {
                            tr {
                                td { "{c.customer_name}" }
                                td { class: "text-right", "PKR {c.total_balance:.0}" }
                                td { class: "text-right", "PKR {c.bucket_31_60:.0}" }
                            }
                        })}
                    }
                }
            }

            // ── 61-90 Days bucket ──
            if !bucket_61_90_customers.is_empty() {
                div { class: "ar-bucket-header ar-bucket-warning",
                    h3 { "🟠 61–90 Days" }
                    span { "— PKR {total_61_90:.0}" }
                }
                table { class: "ar-table",
                    thead { tr {
                        th { "Customer" } th { class: "text-right", "Total Balance" }
                        th { class: "text-right", "61–90 Days" }
                    }}
                    tbody {
                        {bucket_61_90_customers.iter().map(|c| rsx! {
                            tr {
                                td { "{c.customer_name}" }
                                td { class: "text-right", "PKR {c.total_balance:.0}" }
                                td { class: "text-right", "PKR {c.bucket_61_90:.0}" }
                            }
                        })}
                    }
                }
            }

            // ── 90+ Days bucket ──
            if !bucket_90_plus_customers.is_empty() {
                div { class: "ar-bucket-header ar-bucket-danger",
                    h3 { "🔴 90+ Days" }
                    span { "— PKR {total_90_plus:.0}" }
                }
                table { class: "ar-table",
                    thead { tr {
                        th { "Customer" } th { class: "text-right", "Total Balance" }
                        th { class: "text-right", "90+ Days" }
                    }}
                    tbody {
                        {bucket_90_plus_customers.iter().map(|c| rsx! {
                            tr {
                                td { "{c.customer_name}" }
                                td { class: "text-right", "PKR {c.total_balance:.0}" }
                                td { class: "text-right", "PKR {c.bucket_90_plus:.0}" }
                            }
                        })}
                    }
                }
            }

            // ── Summary Totals ──
            table { class: "ar-table",
                tfoot {
                    tr {
                        td { class: "text-total", "Grand Total" }
                        td { class: "text-right text-total", "PKR {total_receivables:.0}" }
                        td { class: "text-right text-total", "PKR {total_current:.0}" }
                    }
                    tr {
                        td { "" }
                        td { class: "text-right", "Total Overdue:" }
                        td { class: "text-right text-danger", "PKR {overdue_total:.0}" }
                    }
                }
            }

            // Export actions
            div { class: "ar-actions",
                Button { variant: ButtonVariant::Secondary, icon: Some("🖨".to_string()), onclick: move |_| trigger_print(), "Print" }
                Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export2, "Export PDF" }
            }
        }
    }
}
