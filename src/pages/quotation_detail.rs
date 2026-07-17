//! Quotation Detail Page — View a single quotation with header, KPI cards,
//! line items, action bar, status change, and conversion to invoice.

use crate::auth::use_auth;
use crate::components::common::{
    use_toast,
};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.qdetail-page { max-width: 960px; margin: 0 auto; }

.qdetail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.qdetail-title-group { display: flex; flex-direction: column; gap: 4px; }
.qdetail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.qdetail-back:hover { text-decoration: underline; }
.qdetail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.qdetail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }

.qdetail-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.qstatus-draft { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.qstatus-sent { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.qstatus-accepted { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.qstatus-rejected { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.qstatus-expired { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.qdetail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.qdetail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.qdetail-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.qdetail-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.qdetail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.qdetail-field { display: flex; flex-direction: column; gap: 3px; }
.qdetail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.qdetail-field-value { font-size: 14px; color: var(--text-primary); }

.qdetail-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }

.qdetail-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.qdetail-items-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.qdetail-items-table thead th.text-right { text-align: right; }
.qdetail-items-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.qdetail-items-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.qdetail-items-table tbody tr:last-child td { border-bottom: none; }
.qdetail-items-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.qdetail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.qdetail-actions-left, .qdetail-actions-right { display: flex; align-items: center; gap: 8px; }

.qdetail-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.qdetail-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: qd-spin 0.8s linear infinite; }
@keyframes qd-spin { to { transform: rotate(360deg); } }

@media (max-width: 768px) {
    .qdetail-header { flex-direction: column; }
    .qdetail-title-row { flex-direction: column; align-items: flex-start; }
    .qdetail-kpis { grid-template-columns: 1fr 1fr; }
    .qdetail-info-grid { grid-template-columns: 1fr; }
    .qdetail-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct QdetailLineItem {
    line_no: i32,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    discount: f64,
    tax_rate: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct QuotationDetail {
    id: i64,
    quotation_no: String,
    customer_name: String,
    customer_code: String,
    date: String,
    valid_until: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    notes: String,
    items: Vec<QdetailLineItem>,
}



fn qstatus_class(status: &str) -> &'static str {
    match status {
        "Draft" => "qstatus-draft",
        "Sent" => "qstatus-sent",
        "Accepted" => "qstatus-accepted",
        "Rejected" => "qstatus-rejected",
        "Expired" => "qstatus-expired",
        _ => "qstatus-draft",
    }
}

fn to_quotation_detail(q: crate::models::Quotation, items: Vec<crate::models::QuotationItem>) -> QuotationDetail {
    QuotationDetail {
        id: q.id,
        quotation_no: q.quotation_no,
        customer_name: q.customer_name.unwrap_or_default(),
        customer_code: String::new(),
        date: q.quotation_date,
        valid_until: q.expiry_date,
        status: q.status,
        subtotal: 0.0,
        discount_percent: 0.0,
        discount_amount: 0.0,
        tax_rate: 0.0,
        tax_amount: 0.0,
        total: q.total_amount,
        notes: q.notes.unwrap_or_default(),
        items: items.into_iter().enumerate().map(|(i, li)| QdetailLineItem {
            line_no: (i + 1) as i32,
            item_code: li.item_code.unwrap_or_default(),
            item_name: li.item_name.unwrap_or_default(),
            quantity: li.quantity,
            unit_price: li.unit_price,
            discount: li.discount,
            tax_rate: li.tax,
            net_amount: li.amount,
        }).collect(),
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let api = use_auth().api;
    let resource = use_resource(move || {
        let fetch_id = id.clone();
        async move {
            let parsed = fetch_id.parse::<i64>().ok()?;
            let api = api.read();
            let client = api.clone();
            drop(api);
            let resp = client.get_quotation(parsed).await.ok()?;
            let data = resp.get("data")?;
            let q: crate::models::Quotation = serde_json::from_value(data.get("quotation")?.clone()).ok()?;
            let items: Vec<crate::models::QuotationItem> = serde_json::from_value(data.get("items")?.clone()).ok()?;
            Some(to_quotation_detail(q, items))
        }
    });

    let is_loading = resource.read().is_none();
    let q_opt = resource.read().as_ref().cloned().flatten();

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page qdetail-page",
                div { class: "qdetail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading..." }
                }
            }
        };
    }
    if q_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page qdetail-page",
                div { class: "qdetail-loading",
                    div { style: "font-size: 40px;", "📋" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Quotation Not Found" }
                    p { "No record found." }
                }
            }
        };
    }
    let q = q_opt.as_ref().unwrap();
    let qid = q.id;

    let on_back = { let nav = navigator; move |_| { nav.push("/sales/quotations"); } };
    let on_convert = {
        let toast = toast.clone();
        move |_| {
            let mut toast = toast.clone();
            let nav = navigator;
            spawn(async move {
                let client = api.read().clone();
                match client.convert_quotation(qid).await {
                    Ok(_) => {
                        toast.success("Converted", "Quotation converted to invoice.");
                        nav.push("/sales/quotations");
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page qdetail-page",
            div { class: "qdetail-header",
                div { class: "qdetail-title-group",
                    button { class: "qdetail-back", onclick: on_back, "← Back to Quotations" }
                    div { class: "qdetail-title-row",
                        h1 { "{q.quotation_no}" }
                        span { class: "qdetail-status-badge {qstatus_class(&q.status)}", "{q.status}" }
                    }
                }
            }

            div { class: "qdetail-section",
                div { class: "qdetail-section-header", h2 { "Details" } }
                div { class: "qdetail-info-grid",
                    div { class: "qdetail-field",
                        span { class: "qdetail-field-label", "Customer" }
                        span { class: "qdetail-field-value", "{q.customer_name}" }
                    }
                    div { class: "qdetail-field",
                        span { class: "qdetail-field-label", "Quotation Date" }
                        span { class: "qdetail-field-value", "{q.date}" }
                    }
                    div { class: "qdetail-field",
                        span { class: "qdetail-field-label", "Valid Until" }
                        span { class: "qdetail-field-value", "{q.valid_until}" }
                    }
                    div { class: "qdetail-field",
                        span { class: "qdetail-field-label", "Total" }
                        span { class: "qdetail-field-value", "{q.total:.2}" }
                    }
                }
            }

            div { class: "qdetail-section",
                div { class: "qdetail-section-header", h2 { "Line Items" } }
                table { class: "qdetail-items-table",
                    thead {
                        tr {
                            th { "#" }
                            th { "Code" }
                            th { "Item" }
                            th { class: "text-right", "Qty" }
                            th { class: "text-right", "Unit Price" }
                            th { class: "text-right", "Discount" }
                            th { class: "text-right", "Tax %" }
                            th { class: "text-right", "Net Amount" }
                        }
                    }
                    tbody {
                        for item in q.items.iter() {
                            tr {
                                td { "{item.line_no}" }
                                td { "{item.item_code}" }
                                td { "{item.item_name}" }
                                td { class: "text-right", "{item.quantity:.2}" }
                                td { class: "text-right", "{item.unit_price:.2}" }
                                td { class: "text-right", "{item.discount:.2}" }
                                td { class: "text-right", "{item.tax_rate:.2}" }
                                td { class: "text-right", "{item.net_amount:.2}" }
                            }
                        }
                    }
                }

                if !q.notes.is_empty() {
                    p { class: "qdetail-notes", style: "margin-top: 16px;", "{q.notes}" }
                }

                div { class: "qdetail-actions",
                    div { class: "qdetail-actions-left" }
                    div { class: "qdetail-actions-right",
                        button { class: "qdetail-back", onclick: on_convert, "Convert to Invoice" }
                    }
                }
            }
        }
    }
}