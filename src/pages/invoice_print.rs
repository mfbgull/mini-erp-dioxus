use dioxus::prelude::*;
use super::print_shared::{PRINT_CSS, DEFAULT_COMPANY, trigger_print};
use crate::auth::use_auth;
use crate::models;

// ============================================================================
// Data
// ============================================================================

#[derive(Clone, Debug)]
struct PrintLineItem {
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct PrintData {
    invoice_no: String,
    invoice_date: String,
    due_date: String,
    customer_name: String,
    customer_code: String,
    customer_address: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    paid_amount: f64,
    balance_amount: f64,
    payment_method: String,
    payment_date: String,
    notes: String,
    terms: String,
    items: Vec<PrintLineItem>,
}



fn status_color(status: &str) -> &'static str {
    match status {
        "Paid" => "#2e7d32",
        "Partially Paid" => "#ed6c02",
        "Unpaid" => "#d32f2f",
        "Cancelled" => "#757575",
        _ => "#1976d2",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn InvoicePrintPage(id: String) -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let print_resource = use_resource(move || {
        let api = api.clone();
        let id = id.clone();
        async move {
            let parsed = id.parse::<i64>().ok()?;
            let client = api.with(|c| c.clone());
            let result = client.get_invoice(parsed).await.ok()?;
            let response_data = result.get("data")?;
            let inv: models::Invoice = serde_json::from_value(response_data.get("invoice")?.clone()).ok()?;
            let items: Vec<models::InvoiceItem> = serde_json::from_value(response_data.get("items")?.clone()).ok()?;

            Some(PrintData {
                invoice_no: inv.invoice_no,
                invoice_date: inv.invoice_date,
                due_date: inv.due_date,
                customer_name: inv.customer_name.clone().unwrap_or_default(),
                customer_code: String::new(),
                customer_address: String::new(),
                status: inv.status,
                subtotal: items.iter().map(|i| i.amount).sum::<f64>(),
                discount_percent: inv.discount_value.unwrap_or(0.0),
                discount_amount: 0.0,
                tax_rate: inv.tax_rate.unwrap_or(0.0),
                tax_amount: 0.0,
                total: inv.total_amount,
                paid_amount: inv.paid_amount,
                balance_amount: inv.balance_amount,
                payment_method: String::new(),
                payment_date: String::new(),
                notes: inv.notes.clone().unwrap_or_default(),
                terms: String::new(),
                items: items.into_iter().map(|ii| PrintLineItem {
                    item_code: ii.item_code.clone().unwrap_or_default(),
                    item_name: ii.item_name.clone().unwrap_or_default(),
                    quantity: ii.quantity,
                    unit_price: ii.unit_price,
                    net_amount: ii.amount,
                }).collect(),
            })
        }
    });

    let data_opt = print_resource.read().as_ref().and_then(|d| d.clone());

    if data_opt.is_none() {
        return rsx! {
            style { "{PRINT_CSS}" }
            div { class: "print-loading", style: "display: flex; justify-content: center; align-items: center; min-height: 60vh;",
                p { "Loading invoice..." }
            }
        };
    }

    let data = data_opt.unwrap();

    rsx! {
        style { "{PRINT_CSS}" }
        style { ".print-title {{ color: #4a90d9; }}" }

        div { class: "print-page",

            // Back + Print buttons (hidden on paper)
            div { class: "print-no-print", style: "margin-bottom: 16px; display: flex; gap: 8px;",
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #e0e0e0; border-radius: 6px; background: #fff; cursor: pointer; font-size: 13px;",
                    onclick: move |_| { navigator.push("/sales/invoices"); },
                    "← Back to Invoices"
                }
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #4a90d9; border-radius: 6px; background: #4a90d9; color: #fff; cursor: pointer; font-size: 13px;",
                    onclick: move |_| trigger_print(),
                    "🖨 Print"
                }
            }

            // Company header
            div { class: "print-header",
                div { class: "print-company",
                    h1 { "{DEFAULT_COMPANY.name}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.address}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.phone_email}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.tax_id}" }
                }
                div { style: "text-align: right;",
                    p { class: "print-title", "INVOICE" }
                    p { style: "font-size: 13px; color: {status_color(&data.status)}; margin: 4px 0 0 0; font-weight: 600;", "{data.status}" }
                    p { style: "font-size: 11px; color: #6c757d; margin: 2px 0 0 0;", "Original" }
                }
            }

            // Info rows
            div { class: "print-info-row",
                div { class: "print-info-block",
                    h3 { "Bill To" }
                    p { style: "font-weight: 600;", "{data.customer_name}" }
                    p { "Code: {data.customer_code}" }
                    p { "{data.customer_address}" }
                }
                div { class: "print-info-block",
                    h3 { "Invoice Details" }
                    p { "Invoice #:  {data.invoice_no}" }
                    p { "Date:  {data.invoice_date}" }
                    p { "Due Date:  {data.due_date}" }
                }
            }

            // Line items
            table { class: "print-table",
                thead { tr {
                    th { "#" } th { "Item Code" } th { "Description" }
                    th { class: "text-right", "Qty" }
                    th { class: "text-right", "Rate" }
                    th { class: "text-right", "Amount" }
                }}
                tbody {
                    {data.items.iter().enumerate().map(|(i, li)| {
                        rsx! {
                            tr {
                                td { "{i + 1}" }
                                td { "{li.item_code}" }
                                td { "{li.item_name}" }
                                td { class: "text-right", "{li.quantity:.0}" }
                                td { class: "text-right", "PKR {li.unit_price:.2}" }
                                td { class: "text-right", "PKR {li.net_amount:.2}" }
                            }
                        }
                    })}
                }
            }

            // Totals
            div { class: "print-totals",
                table {
                    tbody {
                        tr { td { "Subtotal" } td { class: "text-right", "PKR {data.subtotal:.2}" } }
                        tr { td { "Discount ({data.discount_percent}%)" } td { class: "text-right", "PKR {data.discount_amount:.2}" } }
                        tr { td { "Tax ({data.tax_rate}%)" } td { class: "text-right", "PKR {data.tax_amount:.2}" } }
                        tr { class: "total-row", td { "Total" } td { class: "text-right", "PKR {data.total:.2}" } }
                    }
                }
            }

            // Payment info (only if any payment recorded)
            if data.paid_amount > 0.0 {
                div { class: "print-payment-info",
                    h4 { "Payment Information" }
                    p { "Amount Paid:  PKR {data.paid_amount:.2}" }
                    p { "Balance Due:  PKR {data.balance_amount:.2}" }
                    if !data.payment_method.is_empty() {
                        p { "Method:  {data.payment_method}" }
                    }
                    if !data.payment_date.is_empty() {
                        p { "Payment Date:  {data.payment_date}" }
                    }
                }
            }

            // Notes
            if !data.notes.is_empty() {
                div { class: "print-notes",
                    p { style: "margin: 0 0 4px 0; font-weight: 600;", "Notes:" }
                    p { style: "margin: 0;", "{data.notes}" }
                }
            }

            // Terms & Conditions
            if !data.terms.is_empty() {
                div { class: "print-terms",
                    h4 { "Terms & Conditions" }
                    {data.terms.split('\n').map(|line| {
                        rsx! { p { style: "margin: 2px 0;", "{line}" } }
                    })}
                }
            }

            // Authorized signature
            div { class: "print-signature",
                div { class: "print-signature-box",
                    div { class: "print-signature-line", "Authorized Signature" }
                }
            }

            // Footer
            div { class: "print-footer",
                p { style: "margin: 0 0 4px 0;", "Thank you for your business!" }
                p { style: "margin: 0;", "This is a computer-generated invoice. No signature required." }
            }
        }
    }
}
