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

fn mock_print_data() -> QPrintData {
    QPrintData {
        quotation_no: "QOT-2026-0001".to_string(),
        date: "2026-06-22".to_string(),
        valid_until: "2026-07-22".to_string(),
        customer_name: "Alpha Traders".to_string(),
        customer_code: "CUST-001".to_string(),
        customer_address: "42 Model Town, Lahore, Punjab 54000".to_string(),
        subtotal: 159_250.00,
        discount_percent: 5.0,
        discount_amount: 7_962.50,
        tax_rate: 16.0,
        tax_amount: 24_206.00,
        total: 156_000.00,
        notes: "This quotation is valid for 30 days from the date of issue. Prices may change after the validity period.".to_string(),
        terms: "1. This quotation is valid for 30 days.\n2. Prices are exclusive of delivery charges unless stated otherwise.\n3. Payment terms: 50% advance, 50% on delivery.\n4. Subject to availability of stock at the time of order confirmation.".to_string(),
        items: vec![
            QPrintLineItem { item_code: "ITM-0001".to_string(), item_name: "Premium Widget Alpha".to_string(), quantity: 50.0, unit_price: 1500.0, net_amount: 71_250.00 },
            QPrintLineItem { item_code: "ITM-0003".to_string(), item_name: "Steel Rod 12mm x 6m".to_string(), quantity: 200.0, unit_price: 350.0, net_amount: 70_000.00 },
            QPrintLineItem { item_code: "ITM-0005".to_string(), item_name: "Rubber Gasket Set".to_string(), quantity: 100.0, unit_price: 180.0, net_amount: 18_000.00 },
        ],
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationPrintPage(id: String) -> Element {
    let navigator = use_navigator();
    let data = mock_print_data();

    rsx! {
        style { "{PRINT_CSS}" }
        style { ".print-title {{ color: #d4a017; }}" }

        div { class: "print-page",

            // Back + Print buttons
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

            // Company header
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

            // Info rows
            div { class: "print-info-row",
                div { class: "print-info-block",
                    h3 { "Bill To" }
                    p { style: "font-weight: 600;", "{data.customer_name}" }
                    p { "Code: {data.customer_code}" }
                    p { "{data.customer_address}" }
                }
                div { class: "print-info-block",
                    h3 { "Quotation Details" }
                    p { "Quote #:  {data.quotation_no}" }
                    p { "Date:  {data.date}" }
                    p { "Valid Until:  {data.valid_until}" }
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
                p { style: "margin: 0 0 4px 0;", "Thank you for considering MiniERP for your requirements." }
                p { style: "margin: 0;", "This is a computer-generated quotation. No signature required." }
            }
        }
    }
}
