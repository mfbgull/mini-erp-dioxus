//! Customer Statements Page — Detailed account statements with running balance.

use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, use_toast};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.cs-page { max-width: 1000px; margin: 0 auto; }
.cs-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.cs-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.cs-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.cs-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.cs-filter-bar input[type="date"],
.cs-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.cs-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.cs-info-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; display: flex; flex-wrap: wrap; gap: 20px; }
.cs-info-field { display: flex; flex-direction: column; gap: 2px; }
.cs-info-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.cs-info-value { font-size: 14px; color: var(--text-primary); font-weight: 500; }

.cs-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.cs-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.cs-table thead th.text-right { text-align: right; }
.cs-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.cs-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.cs-table tbody td.text-success { color: #28a745; }
.cs-table tbody td.text-danger { color: #dc3545; }
.cs-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.cs-table tfoot td { padding: 8px 10px; font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); font-size: 13px; }
.cs-table tfoot td.text-right { text-align: right; font-family: monospace; }
.cs-table .opening { background: #f8f9fa; font-style: italic; }
.cs-table .closing { background: #f0f4ff; font-weight: 600; }

.cs-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }
.cs-empty { text-align: center; padding: 40px; color: var(--text-secondary); }

@media (max-width: 768px) {
    .cs-filter-bar { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug)]
struct StatementLine {
    date: String,
    reference: String,
    description: String,
    debit: f64,
    credit: f64,
    balance: f64,
}

#[derive(Clone)]
struct CustomerInfo {
    name: String,
    code: String,
    address: String,
    phone: String,
    email: String,
}

// ============================================================================
// Mock Data
// ============================================================================

fn sample_customer() -> CustomerInfo {
    CustomerInfo {
        name: "Alpha Traders".to_string(),
        code: "CUST-001".to_string(),
        address: "42-B, Commercial Area, Gulberg, Lahore".to_string(),
        phone: "+92 300 1234567".to_string(),
        email: "info@alphatraders.pk".to_string(),
    }
}

fn sample_statement() -> Vec<StatementLine> {
    vec![
        StatementLine { date: "2026-06-01".to_string(), reference: "OP-BAL".to_string(), description: "Opening Balance".to_string(), debit: 0.0, credit: 0.0, balance: 45_000.00 },
        StatementLine { date: "2026-06-03".to_string(), reference: "INV-2026-0042".to_string(), description: "Sale — Widgets & Fasteners".to_string(), debit: 156_000.00, credit: 0.0, balance: 201_000.00 },
        StatementLine { date: "2026-06-07".to_string(), reference: "PAY-2026-0030".to_string(), description: "Payment Received — Bank Transfer".to_string(), debit: 0.0, credit: 50_000.00, balance: 151_000.00 },
        StatementLine { date: "2026-06-10".to_string(), reference: "INV-2026-0046".to_string(), description: "Sale — Steel Rods & Consumables".to_string(), debit: 98_500.00, credit: 0.0, balance: 249_500.00 },
        StatementLine { date: "2026-06-15".to_string(), reference: "PAY-2026-0035".to_string(), description: "Payment Received — Cheque".to_string(), debit: 0.0, credit: 100_000.00, balance: 149_500.00 },
        StatementLine { date: "2026-06-18".to_string(), reference: "INV-2026-0050".to_string(), description: "Sale — Electrical Supplies".to_string(), debit: 234_000.00, credit: 0.0, balance: 383_500.00 },
        StatementLine { date: "2026-06-22".to_string(), reference: "CRN-2026-0002".to_string(), description: "Credit Note — Damaged Goods".to_string(), debit: 0.0, credit: 12_000.00, balance: 371_500.00 },
        StatementLine { date: "2026-06-25".to_string(), reference: "PAY-2026-0038".to_string(), description: "Payment Received — Bank Transfer".to_string(), debit: 0.0, credit: 150_000.00, balance: 221_500.00 },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn CustomerStatementsPage() -> Element {
    let toast = use_toast();
    let customer = sample_customer();
    let statement = sample_statement();
    let opening_balance = statement.first().map(|s| s.balance).unwrap_or(0.0);
    let closing_balance = statement.last().map(|s| s.balance).unwrap_or(0.0);
    let total_debits: f64 = statement.iter().map(|s| s.debit).sum();
    let total_credits: f64 = statement.iter().map(|s| s.credit).sum();

    let on_print = {
        let mut t = toast.clone();
        move |_| { t.info("Print", "Statement print dialog will open."); }
    };
    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Statement will be exported as PDF."); }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page cs-page",

            div { class: "cs-header",
                div {
                    h1 { "Customer Statement" }
                    p { class: "page-subtitle", "Detailed account statement with running balance." }
                }
                div { style: "display: flex; gap: 8px;",
                    Button { variant: ButtonVariant::Secondary, icon: Some("🖨".to_string()), onclick: on_print, "Print" }
                    Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export PDF" }
                }
            }

            // Filter bar
            div { class: "cs-filter-bar",
                label { "Customer" }
                select {
                    style: "min-width: 200px;",
                    option { value: "alpha", selected: true, "Alpha Traders (CUST-001)" }
                    option { value: "beta", "Beta Industries (CUST-002)" }
                    option { value: "gamma", "Gamma Supplies (CUST-003)" }
                }
                label { "From" }
                input { r#type: "date", value: "2026-06-01" }
                label { "To" }
                input { r#type: "date", value: "2026-06-27" }
            }

            // KPI cards
            div { class: "cs-kpi-grid",
                StatCard {
                    title: "Opening Balance".to_string(),
                    value: format!("PKR {:.0}", opening_balance),
                    icon: "📋".to_string(),
                    variant: StatCardVariant::Default,
                    footer: Some("As of Jun 1, 2026".to_string()),
                }
                StatCard {
                    title: "Closing Balance".to_string(),
                    value: format!("PKR {:.0}", closing_balance),
                    icon: "💰".to_string(),
                    variant: if closing_balance > 300_000.0 { StatCardVariant::Warning } else { StatCardVariant::Primary },
                    footer: Some("As of Jun 27, 2026".to_string()),
                }
                StatCard {
                    title: "Total Debits".to_string(),
                    value: format!("PKR {:.0}", total_debits),
                    icon: "📤".to_string(),
                    variant: StatCardVariant::Danger,
                    footer: Some("Invoiced amount".to_string()),
                }
                StatCard {
                    title: "Total Credits".to_string(),
                    value: format!("PKR {:.0}", total_credits),
                    icon: "📥".to_string(),
                    variant: StatCardVariant::Success,
                    footer: Some("Payments & credits".to_string()),
                }
            }

            // Customer info card
            div { class: "cs-info-card",
                div { class: "cs-info-field",
                    span { class: "cs-info-label", "Customer" }
                    span { class: "cs-info-value", "{customer.name}" }
                }
                div { class: "cs-info-field",
                    span { class: "cs-info-label", "Code" }
                    span { class: "cs-info-value", "{customer.code}" }
                }
                div { class: "cs-info-field",
                    span { class: "cs-info-label", "Address" }
                    span { class: "cs-info-value", "{customer.address}" }
                }
                div { class: "cs-info-field",
                    span { class: "cs-info-label", "Phone" }
                    span { class: "cs-info-value", "{customer.phone}" }
                }
                div { class: "cs-info-field",
                    span { class: "cs-info-label", "Email" }
                    span { class: "cs-info-value", "{customer.email}" }
                }
            }

            // Statement table
            table { class: "cs-table",
                thead { tr {
                    th { "Date" } th { "Reference" } th { "Description" }
                    th { class: "text-right", "Debit (PKR)" }
                    th { class: "text-right", "Credit (PKR)" }
                    th { class: "text-right", "Balance (PKR)" }
                }}
                tbody {
                    // Opening balance row
                    tr { class: "opening",
                        td { "—" } td { "OP-BAL" } td { "Opening Balance" }
                        td { class: "text-right", "—" } td { class: "text-right", "—" }
                        td { class: "text-right", "PKR {opening_balance:.0}" }
                    }
                    {statement.iter().skip(1).map(|s| {
                        let bal_class = if s.balance > 0.0 { "text-danger" } else { "text-success" };
                        rsx! {
                            tr {
                                td { "{s.date}" }
                                td { style: "font-family: monospace; font-size: 12px;", "{s.reference}" }
                                td { "{s.description}" }
                                td { class: "text-right text-danger",
                                    if s.debit > 0.0 { "PKR {s.debit:.0}" } else { "—" }
                                }
                                td { class: "text-right text-success",
                                    if s.credit > 0.0 { "PKR {s.credit:.0}" } else { "—" }
                                }
                                td { class: "text-right {bal_class}", "PKR {s.balance:.0}" }
                            }
                        }
                    })}
                    // Closing balance row
                    tr { class: "closing",
                        td { "—" } td { "" } td { "Closing Balance" }
                        td { class: "text-right", "PKR {total_debits:.0}" }
                        td { class: "text-right text-success", "PKR {total_credits:.0}" }
                        td { class: "text-right", "PKR {closing_balance:.0}" }
                    }
                }
            }
        }
    }
}
