use crate::auth::use_auth;
use crate::models as models;
use dioxus::prelude::*;
use super::print_shared::{PRINT_CSS, DEFAULT_COMPANY, trigger_print};

#[derive(Clone, Debug)]
struct POLineItem {
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct POData {
    po_no: String,
    po_date: String,
    supplier_name: String,
    supplier_code: String,
    supplier_address: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    delivery_date: String,
    payment_terms: String,
    notes: String,
    terms: String,
    items: Vec<POLineItem>,
}



fn status_color(status: &str) -> &'static str {
    match status {
        "Approved" => "#2e7d32",
        "Pending" => "#ed6c02",
        "Received" => "#1565c0",
        "Cancelled" => "#757575",
        _ => "#1976d2",
    }
}

#[component]
pub fn PurchaseOrderPrintPage(id: String) -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        let pid = id.clone();
        async move {
            let parsed = pid.parse::<i64>().ok()?;
            let client = api.with(|c| c.clone());
            let result = client.get_purchase_order(parsed).await.ok()?;
            let po: models::PurchaseOrder = serde_json::from_value(result.get("po")?.clone()).ok()?;
            let items: Vec<models::PurchaseOrderItem> = serde_json::from_value(result.get("items")?.clone()).ok()?;
            Some(POData {
                po_no: po.po_no,
                po_date: po.po_date,
                supplier_name: po.supplier_name.unwrap_or_default(),
                supplier_code: String::new(),
                supplier_address: String::new(),
                status: po.status,
                subtotal: po.total_amount,
                discount_percent: 0.0,
                discount_amount: 0.0,
                tax_rate: 0.0,
                tax_amount: 0.0,
                total: po.total_amount,
                delivery_date: String::new(),
                payment_terms: String::new(),
                notes: po.notes.unwrap_or_default(),
                terms: String::new(),
                items: items.into_iter().map(|i| POLineItem {
                    item_code: i.item_code.unwrap_or_default(),
                    item_name: i.item_name.unwrap_or_default(),
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                    net_amount: i.amount,
                }).collect(),
            })
        }
    });

    let is_loading = resource.read().is_none();
    let data_opt = resource.read().as_ref().cloned().flatten();

    if is_loading {
        return rsx! {
            style { "{PRINT_CSS}" }
            div { class: "page", style: "display: flex; align-items: center; justify-content: center; min-height: 40vh;",
                span { "Loading purchase order…" }
            }
        };
    }

    let data = match data_opt {
        Some(d) => d,
        None => {
            return rsx! {
                style { "{PRINT_CSS}" }
                div { class: "page", style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 12px;",
                    div { style: "font-size: 40px;", "\u{1f4cb}" }
                    h2 { style: "margin: 0;", "Purchase Order Not Found" }
                    button {
                        r#type: "button",
                        style: "padding: 8px 16px; border: 1px solid #1565c0; border-radius: 6px; background: #1565c0; color: #fff; cursor: pointer; font-size: 13px;",
                        onclick: move |_| { navigator.push("/purchases/orders"); },
                        "\u{2190} Back"
                    }
                }
            };
        }
    };

    rsx! {
        style { "{PRINT_CSS}" }
        style { ".print-title {{ color: #1565c0; }}" }

        div { class: "print-page",

            div { class: "print-no-print", style: "margin-bottom: 16px; display: flex; gap: 8px;",
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #e0e0e0; border-radius: 6px; background: #fff; cursor: pointer; font-size: 13px;",
                    onclick: move |_| { navigator.push("/purchases/orders"); },
                    "← Back to Purchase Orders"
                }
                button {
                    r#type: "button",
                    style: "padding: 8px 16px; border: 1px solid #1565c0; border-radius: 6px; background: #1565c0; color: #fff; cursor: pointer; font-size: 13px;",
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
                    p { class: "print-title", "PURCHASE ORDER" }
                    p { style: "font-size: 13px; color: {status_color(&data.status)}; margin: 4px 0 0 0; font-weight: 600;", "{data.status}" }
                    p { style: "font-size: 11px; color: #6c757d; margin: 2px 0 0 0;", "Original" }
                }
            }

            div { class: "print-info-row",
                div { class: "print-info-block",
                    h3 { "Supplier" }
                    p { style: "font-weight: 600;", "{data.supplier_name}" }
                    p { "Code: {data.supplier_code}" }
                    p { "{data.supplier_address}" }
                }
                div { class: "print-info-block",
                    h3 { "Purchase Order Details" }
                    p { "PO #:  {data.po_no}" }
                    p { "Date:  {data.po_date}" }
                    p { "Expected Delivery:  {data.delivery_date}" }
                    p { "Payment Terms:  {data.payment_terms}" }
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

            if !data.notes.is_empty() {
                div { class: "print-notes",
                    p { style: "margin: 0 0 4px 0; font-weight: 600;", "Delivery Notes:" }
                    p { style: "margin: 0;", "{data.notes}" }
                }
            }

            if !data.terms.is_empty() {
                div { class: "print-terms",
                    h4 { "Terms & Conditions" }
                    {data.terms.split('\n').map(|line| {
                        rsx! { p { style: "margin: 2px 0;", "{line}" } }
                    })}
                }
            }

            div { style: "display: flex; justify-content: space-between; margin-top: 48px;",
                div { style: "width: 200px; text-align: center;",
                    div { style: "border-top: 1px solid #1a1a1a; margin-top: 60px; padding-top: 6px; font-size: 12px; color: #6c757d;", "Prepared By" }
                }
                div { style: "width: 200px; text-align: center;",
                    div { style: "border-top: 1px solid #1a1a1a; margin-top: 60px; padding-top: 6px; font-size: 12px; color: #6c757d;", "Approved By" }
                }
                div { style: "width: 200px; text-align: center;",
                    div { style: "border-top: 1px solid #1a1a1a; margin-top: 60px; padding-top: 6px; font-size: 12px; color: #6c757d;", "Supplier Acknowledgment" }
                }
            }

            div { class: "print-footer",
                p { style: "margin: 0;", "This is a computer-generated purchase order." }
            }
        }
    }
}
