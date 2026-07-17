//! Sales Order Detail Page — View a single sales order with header, KPI cards,
//! line items, action bar, and conversion to invoice.

use crate::components::common::{
    use_toast,
};
use dioxus::prelude::*;
use crate::auth::use_auth;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.sodetail-page { max-width: 960px; margin: 0 auto; }

.sodetail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.sodetail-title-group { display: flex; flex-direction: column; gap: 4px; }
.sodetail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.sodetail-back:hover { text-decoration: underline; }
.sodetail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.sodetail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }

.sodetail-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.sostatus-draft { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.sostatus-confirmed { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.sostatus-processing { background: rgba(128, 0, 128, 0.1); color: #800080; }
.sostatus-shipped { background: rgba(0, 188, 212, 0.1); color: #00bcd4; }
.sostatus-delivered { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.sostatus-cancelled { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.sodetail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.sodetail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.sodetail-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sodetail-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.sodetail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.sodetail-field { display: flex; flex-direction: column; gap: 3px; }
.sodetail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.sodetail-field-value { font-size: 14px; color: var(--text-primary); }

.sodetail-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }

.sodetail-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.sodetail-items-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.sodetail-items-table thead th.text-right { text-align: right; }
.sodetail-items-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.sodetail-items-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.sodetail-items-table tbody tr:last-child td { border-bottom: none; }
.sodetail-items-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.sodetail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.sodetail-actions-left, .sodetail-actions-right { display: flex; align-items: center; gap: 8px; }

.sodetail-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.sodetail-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: sod-spin 0.8s linear infinite; }
@keyframes sod-spin { to { transform: rotate(360deg); } }

@media (max-width: 768px) {
    .sodetail-header { flex-direction: column; }
    .sodetail-title-row { flex-direction: column; align-items: flex-start; }
    .sodetail-kpis { grid-template-columns: 1fr 1fr; }
    .sodetail-info-grid { grid-template-columns: 1fr; }
    .sodetail-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct SoLineItem {
    line_no: i32,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct SalesOrderDetail {
    id: i64,
    order_no: String,
    customer_name: String,
    customer_code: String,
    order_date: String,
    delivery_date: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    notes: String,
    items: Vec<SoLineItem>,
}



fn sostatus_class(status: &str) -> &'static str {
    match status {
        "Draft" => "sostatus-draft",
        "Confirmed" => "sostatus-confirmed",
        "Processing" => "sostatus-processing",
        "Shipped" => "sostatus-shipped",
        "Delivered" => "sostatus-delivered",
        "Cancelled" => "sostatus-cancelled",
        _ => "sostatus-draft",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesOrderDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let api = use_auth().api;
    let resource = use_resource(move || {
        let pid = id.clone();
        async move {
            let parsed = pid.parse::<i64>().ok()?;
            let client = api.with(|c| c.clone());
            let result = client.get_sales_order(parsed).await.ok()?;
            let order: crate::models::SalesOrder = serde_json::from_value(result.get("order")?.clone()).ok()?;
            let items: Vec<crate::models::SalesOrderItem> = serde_json::from_value(result.get("items")?.clone()).ok()?;
            Some(SalesOrderDetail {
                id: order.id,
                order_no: order.so_no,
                customer_name: order.customer_name.unwrap_or_default(),
                customer_code: order.customer_code.unwrap_or_default(),
                order_date: order.so_date.clone(),
                delivery_date: order.delivery_date.unwrap_or_default(),
                status: order.status,
                subtotal: order.total_amount,  // ponytail: server only has total_amount
                discount_percent: 0.0,
                discount_amount: 0.0,
                tax_rate: 0.0,
                tax_amount: 0.0,
                total: order.total_amount,
                notes: order.notes.unwrap_or_default(),
                items: items.into_iter().enumerate().map(|(i, item)| SoLineItem {
                    line_no: (i + 1) as i32,
                    item_code: item.item_code.unwrap_or_default(),
                    item_name: item.item_name.unwrap_or_default(),
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    net_amount: item.amount,
                }).collect(),
            })
        }
    });

    let is_loading = resource.read().is_none();
    let so_opt = resource.read().as_ref().cloned().flatten();

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sodetail-page",
                div { class: "sodetail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading..." }
                }
            }
        };
    }
    if so_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sodetail-page",
                div { class: "sodetail-loading",
                    div { style: "font-size: 40px;", "📦" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Sales Order Not Found" }
                    p { "No record found." }
                }
            }
        };
    }
    let so = so_opt.as_ref().unwrap();
    let sid = so.id;

    let on_back = { let nav = navigator; move |_| { nav.push("/sales/sales-orders"); } };
    let on_convert = {
        let toast = toast.clone();
        move |_| {
            let mut toast = toast.clone();
            let nav = navigator;
            spawn(async move {
                let client = api.read().clone();
                match client.convert_sales_order(sid).await {
                    Ok(_) => {
                        toast.success("Converted", "Sales order converted to invoice.");
                        nav.push("/sales/sales-orders");
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };
    let on_cancel = {
        let toast = toast.clone();
        move |_| {
            let mut toast = toast.clone();
            let nav = navigator;
            spawn(async move {
                let client = api.read().clone();
                match client.cancel_sales_order(sid).await {
                    Ok(_) => {
                        toast.success("Cancelled", "Sales order has been cancelled.");
                        nav.push("/sales/sales-orders");
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page sodetail-page",
            div { class: "sodetail-header",
                div { class: "sodetail-title-group",
                    button { class: "sodetail-back", onclick: on_back, "← Back to Sales Orders" }
                    div { class: "sodetail-title-row",
                        h1 { "{so.order_no}" }
                        span { class: "sodetail-status-badge {sostatus_class(&so.status)}", "{so.status}" }
                    }
                }
            }

            div { class: "sodetail-section",
                div { class: "sodetail-section-header", h2 { "Details" } }
                div { class: "sodetail-info-grid",
                    div { class: "sodetail-field",
                        span { class: "sodetail-field-label", "Customer" }
                        span { class: "sodetail-field-value", "{so.customer_name}" }
                    }
                    div { class: "sodetail-field",
                        span { class: "sodetail-field-label", "Order Date" }
                        span { class: "sodetail-field-value", "{so.order_date}" }
                    }
                    div { class: "sodetail-field",
                        span { class: "sodetail-field-label", "Delivery Date" }
                        span { class: "sodetail-field-value", "{so.delivery_date}" }
                    }
                    div { class: "sodetail-field",
                        span { class: "sodetail-field-label", "Total" }
                        span { class: "sodetail-field-value", "{so.total:.2}" }
                    }
                }
            }

            div { class: "sodetail-section",
                div { class: "sodetail-section-header", h2 { "Line Items" } }
                table { class: "sodetail-items-table",
                    thead {
                        tr {
                            th { "#" }
                            th { "Code" }
                            th { "Item" }
                            th { class: "text-right", "Qty" }
                            th { class: "text-right", "Unit Price" }
                            th { class: "text-right", "Net Amount" }
                        }
                    }
                    tbody {
                        for item in so.items.iter() {
                            tr {
                                td { "{item.line_no}" }
                                td { "{item.item_code}" }
                                td { "{item.item_name}" }
                                td { class: "text-right", "{item.quantity:.2}" }
                                td { class: "text-right", "{item.unit_price:.2}" }
                                td { class: "text-right", "{item.net_amount:.2}" }
                            }
                        }
                    }
                }

                if !so.notes.is_empty() {
                    p { class: "sodetail-notes", style: "margin-top: 16px;", "{so.notes}" }
                }

                div { class: "sodetail-actions",
                    div { class: "sodetail-actions-left",
                        button { class: "sodetail-back", onclick: on_cancel, "Cancel Order" }
                    }
                    div { class: "sodetail-actions-right",
                        button { class: "sodetail-back", onclick: on_convert, "Convert to Invoice" }
                    }
                }
            }
        }
    }
}