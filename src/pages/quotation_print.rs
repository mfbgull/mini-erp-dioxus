use crate::auth::use_auth;
use dioxus::prelude::*;
use super::print_shared::{PRINT_CSS, DEFAULT_COMPANY, trigger_print};

// ============================================================================
// Data
// ============================================================================

#[derive(Clone, Debug)]
struct QPrintLineItem {
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct QPrintData {
    quotation_no: String,
    date: String,
    valid_until: String,
    customer_name: String,
    customer_code: String,
    customer_address: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    notes: String,
    terms: String,
    items: Vec<QPrintLineItem>,
}



fn to_print_data(q: crate::models::Quotation, items: Vec<crate::models::QuotationItem>) -> QPrintData {
    QPrintData {
        quotation_no: q.quotation_no,
        date: q.quotation_date,
        valid_until: q.expiry_date,
        customer_name: q.customer_name.unwrap_or_default(),
        customer_code: String::new(),
        customer_address: String::new(),
        subtotal: 0.0,
        discount_percent: 0.0,
        discount_amount: 0.0,
        tax_rate: 0.0,
        tax_amount: 0.0,
        total: q.total_amount,
        notes: q.notes.unwrap_or_default(),
        terms: String::new(),
        items: items.into_iter().map(|li| QPrintLineItem {
            item_code: li.item_code.unwrap_or_default(),
            item_name: li.item_name.unwrap_or_default(),
            quantity: li.quantity,
            unit_price: li.unit_price,
            net_amount: li.amount,
        }).collect(),
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationPrintPage(id: String) -> Element {
    let navigator = use_navigator();
    let auth = use_auth();
    let resource = use_resource(move || {
        let fetch_id = id.clone();
        async move {
            let parsed = fetch_id.parse::<i64>().ok()?;
            let api = auth.api.read();
            let client = api.clone();
            drop(api);
            let resp = client.get_quotation(parsed).await.ok()?;
            let data = resp.get("data")?;
            let q: crate::models::Quotation = serde_json::from_value(data.get("quotation")?.clone()).ok()?;
            let items: Vec<crate::models::QuotationItem> = serde_json::from_value(data.get("items")?.clone()).ok()?;
            Some(to_print_data(q, items))
        }
    });

    let pd: QPrintData = match resource.read().as_ref().cloned().flatten() {
        Some(d) => d,
        None => return rsx! {
            div { class: "page", style: "display: flex; align-items: center; justify-content: center; min-height: 60vh; color: var(--text-secondary);",
                span { "Loading…" }
            }
        },
    };

    rsx! {
        style { "{PRINT_CSS}" }
        style { ".print-title {{ color: #d4a017; }}" }

        div { class: "print-page",

            div { class: "print-no-print", style: "margin-bottom: 16px; display: flex; gap: 8px;",
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #e0e0e0; border-radius: 6px; background: #fff; cursor: pointer; font-size: 13px;",
                    onclick: move |_| { navigator.push("/sales/quotations"); },
                    "← Back to Quotations"
                }
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #d4a017; border-radius: 6px; background: #d4a017; color: #fff; cursor: pointer; font-size: 13px;",
                    onclick: move |_| trigger_print(),
                    "🖨 Print"
                }
            }

            div { class: "print-header",
                div { class: "print-company",
                    h1 { "{DEFAULT_COMPANY.name}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.address}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.phone_email}" }
                    span { class: "print-company-subtitle", "{DEFAULT_COMPANY.tax_id}" }
                }
                div { style: "text-align: right;",
                    p { class: "print-title", "QUOTATION" }
                    p { style: "font-size: 11px; color: #6c757d; margin: 2px 0 0 0;", "Original" }
                }
            }

            div { class: "print-info-row",
                div { class: "print-info-block",
                    h3 { "Bill To" }
                    p { style: "font-weight: 600;", "{pd.customer_name}" }
                    p { "Code: {pd.customer_code}" }
                    p { "{pd.customer_address}" }
                }
                div { class: "print-info-block",
                    h3 { "Quotation Details" }
                    p { "Quote #:  {pd.quotation_no}" }
                    p { "Date:  {pd.date}" }
                    p { "Valid Until:  {pd.valid_until}" }
                }
            }

            table { class: "print-table",
                thead { tr {
                    th { "#" } th { "Item Code" } th { "Description" }
                    th { class: "text-right", "Qty" }
                    th { class: "text-right", "Rate" }
                    th { class: "text-right", "Amount" }
                }}
                tbody {
                    {pd.items.iter().enumerate().map(|(i, li)| {
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

            div { class: "print-totals",
                table {
                    tbody {
                        tr { td { "Subtotal" } td { class: "text-right", "PKR {pd.subtotal:.2}" } }
                        tr { td { "Discount ({pd.discount_percent}%)" } td { class: "text-right", "PKR {pd.discount_amount:.2}" } }
                        tr { td { "Tax ({pd.tax_rate}%)" } td { class: "text-right", "PKR {pd.tax_amount:.2}" } }
                        tr { class: "total-row", td { "Total" } td { class: "text-right", "PKR {pd.total:.2}" } }
                    }
                }
            }

            if !pd.notes.is_empty() {
                div { class: "print-notes",
                    p { style: "margin: 0 0 4px 0; font-weight: 600;", "Notes:" }
                    p { style: "margin: 0;", "{pd.notes}" }
                }
            }

            if !pd.terms.is_empty() {
                div { class: "print-terms",
                    h4 { "Terms & Conditions" }
                    {pd.terms.split('\n').map(|line| {
                        rsx! { p { style: "margin: 2px 0;", "{line}" } }
                    })}
                }
            }

            div { class: "print-signature",
                div { class: "print-signature-box",
                    div { class: "print-signature-line", "Authorized Signature" }
                }
            }

            div { class: "print-footer",
                p { style: "margin: 0 0 4px 0;", "Thank you for considering MiniERP for your requirements." }
                p { style: "margin: 0;", "This is a computer-generated quotation. No signature required." }
            }
        }
    }
}
