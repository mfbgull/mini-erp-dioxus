//! Customer Statements Page — Detailed account statements with running balance.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, use_toast};
use crate::models;
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.cs-page { max-width: 1000px; margin: 0 auto; }
.cs-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.cs-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.cs-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.cs-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.cs-filter-bar input[type="date"], .cs-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }
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
.cs-empty { text-align: center; padding: 40px; color: var(--text-secondary); }
@media (max-width: 768px) { .cs-filter-bar { flex-direction: column; align-items: stretch; } }
"##;

#[derive(Clone, Default)]
struct CustomerInfo { name: String, code: String, address: String, phone: String, email: String }

#[derive(Clone, Default)]
struct StatementLine { date: String, reference: String, description: String, debit: f64, credit: f64, balance: f64 }

#[component]
pub fn CustomerStatementsPage() -> Element {
    let toast = use_toast();
    let api = use_auth().api;
    let customers = use_signal(Vec::<models::Customer>::new);
    let selected_id = use_signal(|| 0i64);
    let from_date = use_signal(|| "2026-06-01".to_string());
    let to_date = use_signal(|| "2026-06-30".to_string());
    let cust_info = use_signal(CustomerInfo::default);
    let statement = use_signal(Vec::<StatementLine>::new);

    // Load customer list on mount
    {
        let api = api.clone();
        let mut customers = customers.clone();
        use_effect(move || {
            let api = api.clone();
            let mut customers = customers.clone();
            spawn(async move {
                let client = api.read().clone();
                if let Ok(list) = client.list_customers().await {
                    customers.set(list);
                }
            });
        });
    }

    // Load statement when customer or date range changes
    {
        let api = api.clone();
        let id = *selected_id.read();
        let from = from_date.read().clone();
        let to = to_date.read().clone();
        let mut cust_info = cust_info.clone();
        let mut statement = statement.clone();
        if id > 0 {
            use_effect(move || {
                let api = api.clone();
                let mut cust_info = cust_info.clone();
                let mut statement = statement.clone();
                let from = from.clone();
                let to = to.clone();
                spawn(async move {
                    let client = api.read().clone();
                    let customer = client.get_customer(id).await.ok();
                    let ledger = client.get_customer_ledger(id).await.ok().unwrap_or_default();
                    if let Some(c) = customer {
                        cust_info.set(CustomerInfo {
                            name: c.customer_name, code: c.customer_code,
                            address: c.billing_address, phone: c.phone, email: c.email,
                        });
                    }
                    let lines: Vec<StatementLine> = ledger.iter()
                        .filter(|l| l.transaction_date >= from && l.transaction_date <= to)
                        .map(|l| StatementLine {
                            date: l.transaction_date.clone(), reference: l.reference_no.clone(),
                            description: l.transaction_type.clone(),
                            debit: l.debit, credit: l.credit, balance: l.balance,
                        })
                        .collect();
                    statement.set(lines);
                });
            });
        }
    }

    let on_customer_change = {
        let mut selected_id = selected_id;
        move |e: Event<FormData>| {
            if let Ok(id) = e.value().parse::<i64>() { selected_id.set(id); }
        }
    };
    let on_from_change = { let mut from_date = from_date; move |e: Event<FormData>| { from_date.set(e.value()); } };
    let on_to_change = { let mut to_date = to_date; move |e: Event<FormData>| { to_date.set(e.value()); } };
    let on_print = { let mut t = toast.clone(); move |_| { t.info("Print", "Statement print dialog will open."); } };
    let on_export = { let mut t = toast.clone(); move |_| { t.info("Export", "Statement will be exported as PDF."); } };

    let c = cust_info.read();
    let s = statement.read();
    let opening = s.first().map(|x| x.balance).unwrap_or(0.0);
    let closing = s.last().map(|x| x.balance).unwrap_or(0.0);
    let total_dr: f64 = s.iter().map(|x| x.debit).sum();
    let total_cr: f64 = s.iter().map(|x| x.credit).sum();
    let cv = if closing > 300_000.0 { StatCardVariant::Warning } else { StatCardVariant::Primary };
    let cust_list = customers.read();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page cs-page",
            div { class: "cs-header",
                div { h1 { "Customer Statement" } p { class: "page-subtitle", "Detailed account statement with running balance." } }
                div { style: "display: flex; gap: 8px;",
                    Button { variant: ButtonVariant::Secondary, icon: Some("🖨".to_string()), onclick: on_print, "Print" }
                    Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export PDF" }
                }
            }
            div { class: "cs-filter-bar",
                label { "Customer" }
                select { style: "min-width: 200px;", onchange: on_customer_change,
                    option { value: "0", "Select a customer…" }
                    {cust_list.iter().map(|c| {
                        let label = format!("{} ({})", c.customer_name, c.customer_code);
                        rsx! { option { value: "{c.id}", "{label}" } }
                    })}
                }
                label { "From" }
                input { r#type: "date", value: "{from_date}", onchange: on_from_change }
                label { "To" }
                input { r#type: "date", value: "{to_date}", onchange: on_to_change }
            }
            div { class: "cs-kpi-grid",
                StatCard { title: "Opening Balance".to_string(), value: format!("PKR {opening:.0}"), icon: "📋".to_string(), variant: StatCardVariant::Default, footer: Some("Period start".to_string()) }
                StatCard { title: "Closing Balance".to_string(), value: format!("PKR {closing:.0}"), icon: "💰".to_string(), variant: cv, footer: Some("Period end".to_string()) }
                StatCard { title: "Total Debits".to_string(), value: format!("PKR {total_dr:.0}"), icon: "📤".to_string(), variant: StatCardVariant::Danger, footer: Some("Invoiced amount".to_string()) }
                StatCard { title: "Total Credits".to_string(), value: format!("PKR {total_cr:.0}"), icon: "📥".to_string(), variant: StatCardVariant::Success, footer: Some("Payments & credits".to_string()) }
            }
            div { class: "cs-info-card",
                div { class: "cs-info-field", span { class: "cs-info-label", "Customer" }, span { class: "cs-info-value", "{c.name}" } }
                div { class: "cs-info-field", span { class: "cs-info-label", "Code" }, span { class: "cs-info-value", "{c.code}" } }
                div { class: "cs-info-field", span { class: "cs-info-label", "Address" }, span { class: "cs-info-value", "{c.address}" } }
                div { class: "cs-info-field", span { class: "cs-info-label", "Phone" }, span { class: "cs-info-value", "{c.phone}" } }
                div { class: "cs-info-field", span { class: "cs-info-label", "Email" }, span { class: "cs-info-value", "{c.email}" } }
            }
            table { class: "cs-table",
                thead { tr { th { "Date" } th { "Reference" } th { "Description" } th { class: "text-right", "Debit (PKR)" } th { class: "text-right", "Credit (PKR)" } th { class: "text-right", "Balance (PKR)" } } }
                tbody {
                    {s.iter().map(|line| {
                        let bc = if line.balance > 0.0 { "text-danger" } else { "text-success" };
                        let dt = if line.debit > 0.0 { format!("PKR {:.0}", line.debit) } else { "—".to_string() };
                        let ct = if line.credit > 0.0 { format!("PKR {:.0}", line.credit) } else { "—".to_string() };
                        let bt = format!("PKR {:.0}", line.balance);
                        rsx! {
                            tr { td { "{line.date}" } td { style: "font-family: monospace; font-size: 12px;", "{line.reference}" }
                                td { "{line.description}" } td { class: "text-right text-danger", "{dt}" }
                                td { class: "text-right text-success", "{ct}" } td { class: "text-right {bc}", "{bt}" } }
                        }
                    })}
                    if s.is_empty() {
                        tr { td { class: "cs-empty", colspan: "6", "No transactions found for this period." } }
                    }
                }
                if !s.is_empty() {
                    tfoot { tr { td { colspan: "3", "Totals" } td { class: "text-right", "PKR {total_dr:.0}" }
                        td { class: "text-right text-success", "PKR {total_cr:.0}" } td { class: "text-right", "PKR {closing:.0}" } } }
                }
            }
        }
    }
}
