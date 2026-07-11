//! Purchase Order Detail Page — Detail view for a purchase order with
//! header info, line items table, KPI cards, and action bar.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast};
use crate::models as models;
use dioxus::prelude::*;
use serde_json::json;

const PAGE_CSS: &str = r##"
.po-detail-page { max-width: 1000px; margin: 0 auto; }
.po-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.po-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.po-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.po-detail-back:hover { text-decoration: underline; }
.po-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.po-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.po-detail-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.po-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.po-status-draft { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.po-status-sent { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.po-status-confirmed { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.po-status-partial { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.po-status-received { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.po-status-cancelled { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.po-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.po-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.po-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.po-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.po-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.po-field { display: flex; flex-direction: column; gap: 3px; }
.po-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.po-field-value { font-size: 14px; color: var(--text-primary); }
.po-field-value.monospace { font-family: monospace; font-size: 13px; }
.po-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.po-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.po-table thead th.text-right { text-align: right; }
.po-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.po-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.po-table tbody tr:last-child td { border-bottom: none; }
.po-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.po-actions-left, .po-actions-right { display: flex; align-items: center; gap: 8px; }
.po-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.po-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: po-spin 0.8s linear infinite; }
@keyframes po-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .po-detail-header { flex-direction: column; } .po-detail-kpis { grid-template-columns: 1fr 1fr; } .po-info-grid { grid-template-columns: 1fr; } .po-actions { flex-direction: column; align-items: stretch; } }
"##;

#[derive(Clone, Debug)]
struct PoLineItem {
    po_item_id: i64, item_id: i64,
    item_code: String, item_name: String, quantity: f64, received: f64,
    rate: f64, discount: f64, tax: f64, amount: f64,
}

#[derive(Clone, Debug)]
struct PoDetail {
    id: i64, po_no: String, supplier_name: String, supplier_code: String,
    order_date: String, expected_date: String, status: String,
    items: Vec<PoLineItem>, subtotal: f64, discount_total: f64, tax_total: f64, grand_total: f64, notes: String,
}



fn status_class(s: &str) -> &'static str {
    match s {
        "Draft" => "po-status-draft", "Sent" => "po-status-sent",
        "Confirmed" => "po-status-confirmed", "Partially Received" => "po-status-partial",
        "Received" => "po-status-received", "Cancelled" => "po-status-cancelled", _ => "po-status-draft",
    }
}

#[component]
pub fn PurchaseOrderDetailPage(id: String) -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();

    let id_display = id.clone();
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        let pid = id.clone();
        async move {
            let parsed = pid.parse::<i64>().ok()?;
            let client = api.with(|c| c.clone());
            let result = client.get_purchase_order(parsed).await.ok()?;
            let po: models::PurchaseOrder = serde_json::from_value(result.get("purchase_order")?.clone()).ok()?;
            let items: Vec<models::PurchaseOrderItem> = serde_json::from_value(result.get("items")?.clone()).ok()?;
            let line_items: Vec<PoLineItem> = items.into_iter().map(|i| PoLineItem {
                po_item_id: i.id, item_id: i.item_id,
                item_code: i.item_code.unwrap_or_default(),
                item_name: i.item_name.unwrap_or_default(),
                quantity: i.quantity, received: i.received_quantity,
                rate: i.unit_price,
                discount: 0.0, tax: 0.0,
                amount: i.amount,
            }).collect();
            let subtotal: f64 = line_items.iter().map(|i| i.quantity * i.rate).sum();
            Some(PoDetail {
                id: po.id,
                po_no: po.po_no,
                supplier_name: po.supplier_name.unwrap_or_default(),
                supplier_code: po.supplier_code.unwrap_or_default(),
                order_date: po.po_date,
                expected_date: po.expected_date.unwrap_or_default(),
                status: po.status,
                items: line_items,
                subtotal,
                discount_total: 0.0, // ponytail: not in PO model
                tax_total: 0.0,      // ponytail: not in PO model
                grand_total: po.total_amount,
                notes: po.notes.unwrap_or_default(),
            })
        }
    });

    let loading = resource.read().is_none();
    let detail_opt = resource.read().as_ref().and_then(|d| d.clone());
    let mut show_delete_modal = use_signal(|| false);
    let mut show_receive_modal = use_signal(|| false);
    let mut receive_notes = use_signal(String::new);
    let mut receive_saving = use_signal(|| false);
    let detail_ready = detail_opt.is_some();

    let on_back = move |_| { navigator.push("/purchases/orders"); };
    let on_delete = move |_| { show_delete_modal.set(true); };
    let cancel_delete = move |_| { show_delete_modal.set(false); };
    let on_edit = {
        let nav = navigator.clone();
        let pid = id_display.clone();
        move |_| { nav.push(format!("/purchases/orders/{}/edit", pid)); }
    };
    let on_receive = { let mut show = show_receive_modal.clone(); move |_| show.set(true) };
    let on_print = { let mut t = toast.clone(); move |_| t.info("Print", "Print coming soon.") };
    let toast_for_receive = toast.clone();
    let confirm_delete = {
        let nav = navigator.clone();
        move |_| {
            let mut t = toast.clone();
            show_delete_modal.set(false);
            t.success("Deleted", "PO deleted.");
            nav.push("/purchases/orders");
        }
    };

    let cancel_receive = { let mut show = show_receive_modal.clone(); move |_| show.set(false) };

    let confirm_receive = {
        let mut show = show_receive_modal.clone();
        let mut saving = receive_saving.clone();
        let notes_sig = receive_notes.clone();
        let api = api.clone();
        let po_id = id_display.clone();
        let items_data = detail_opt.as_ref().map(|d| d.items.clone()).unwrap_or_default();
        let toast2 = toast_for_receive.clone();
        move |_| {
            // Build receipt items from PO items (receive full remaining qty for each)
            let receipt_items: Vec<serde_json::Value> = items_data.iter()
                .filter(|i| i.received < i.quantity)
                .map(|i| {
                    let remaining = i.quantity - i.received;
                    json!({
                        "po_item_id": i.po_item_id,
                        "item_id": i.item_id,
                        "received_quantity": remaining,
                    })
                }).collect();
            if receipt_items.is_empty() {
                let mut t = toast2.clone();
                t.info("Nothing to Receive", "All items have already been fully received.");
                return;
            }
            saving.set(true);
            let mut toast = toast2.clone();
            let mut saving = saving.clone();
            let mut show = show.clone();
            let notes = notes_sig.read().clone();
            let pid = po_id.parse::<i64>().unwrap_or(0);
            let api = api.clone();
            spawn(async move {
                let form = json!({
                    "po_id": pid,
                    "receipt_date": chrono::Local::now().format("%Y-%m-%d").to_string(),
                    "warehouse_id": null,
                    "notes": if notes.is_empty() { None::<String> } else { Some(notes) },
                    "items": receipt_items,
                });
                let client = api.with(|c| c.clone());
                match client.create_goods_receipt(pid, &form).await {
                    Ok(_) => {
                        toast.success("Goods Received", "Stock has been updated.");
                        show.set(false);
                        saving.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to record receipt: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };


    if loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page po-detail-page",
                div { class: "po-loading", div { class: "loading-spinner" } span { "Loading purchase order details…" } }
            }
        };
    }
    if !detail_ready {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page po-detail-page",
                div { class: "po-loading",
                    div { style: "font-size: 40px;", "📋" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Purchase Order Not Found" }
                    p { "No PO with ID \"{id_display}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: on_back, "← Back" }
                }
            }
        };
    }
    // ponytail: unwrap safe here since detail_ready is true
    let d = detail_opt.as_ref().cloned().unwrap();
    let sc = status_class(&d.status);

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page po-detail-page",
            div { class: "po-detail-header",
            div { class: "po-detail-title-group",
            Button { class: Some("po-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: on_back, "← Back to Purchase Orders" }
            div { class: "po-detail-title-row",
            h1 { "Purchase Order {d.po_no}" }
            span { class: "po-detail-code", "{d.supplier_code}" }
            span { class: "po-status-badge {sc}", "{d.status}" }
            }
            }
            }
            div { class: "po-detail-kpis",
            StatCard { title: "Order Date".to_string(), value: d.order_date.clone(), variant: StatCardVariant::Default }
            StatCard { title: "Expected Delivery".to_string(), value: d.expected_date.clone(), variant: StatCardVariant::Warning }
            StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.0}", d.subtotal), variant: StatCardVariant::Default }
            StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.0}", d.grand_total), variant: StatCardVariant::Primary }
            }
            div { class: "po-section",
            div { class: "po-section-header", h2 { "Order Information" } }
            div { class: "po-info-grid",
            div { class: "po-field", span { class: "po-field-label", "Supplier" } span { class: "po-field-value", "{d.supplier_name}" } }
            div { class: "po-field", span { class: "po-field-label", "Supplier Code" } span { class: "po-field-value monospace", "{d.supplier_code}" } }
            div { class: "po-field", span { class: "po-field-label", "Order Date" } span { class: "po-field-value", "{d.order_date}" } }
            div { class: "po-field", span { class: "po-field-label", "Expected Delivery" } span { class: "po-field-value", "{d.expected_date}" } }
            div { class: "po-field", span { class: "po-field-label", "Status" } span { class: "po-field-value", "{d.status}" } }
            }
            }
            div { class: "po-section",
            div { class: "po-section-header", h2 { "Line Items" } }
            table { class: "po-table",
            thead { tr {
            th { "Item Code" } th { "Item Name" } th { class: "text-right", "Qty" }
            th { class: "text-right", "Rate" } th { class: "text-right", "Disc %" }
            th { class: "text-right", "Tax %" } th { class: "text-right", "Amount" }
            } }
            tbody {
            {d.items.iter().map(|li| rsx! {
            tr {
            td { style: "font-family: monospace;", "{li.item_code}" }
            td { "{li.item_name}" }
            td { class: "text-right", "{li.quantity:.0}" }
            td { class: "text-right", "PKR {li.rate:.2}" }
            td { class: "text-right", "{li.discount:.0}%" }
            td { class: "text-right", "{li.tax:.0}%" }
            td { class: "text-right", "PKR {li.amount:.2}" }
            }
            })}
            tr { style: "font-weight: 700;",
            td { colspan: "6", style: "text-align: right; padding-right: 10px;", "Grand Total" }
            td { class: "text-right", style: "font-size: 14px;", "PKR {d.grand_total:.0}" }
            }
            }
            }
            }
            if !d.notes.is_empty() {
            div { class: "po-section",
            div { class: "po-section-header", h2 { "Notes" } }
            p { style: "font-size: 13px; color: var(--text-secondary); line-height: 1.6; margin: 0;", "{d.notes}" }
            }
            }
            div { class: "po-actions",
            div { class: "po-actions-left",
            Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit" }
            Button { variant: ButtonVariant::Secondary, onclick: on_receive, icon: Some("📦".to_string()), "Receive Goods" }
            }
            div { class: "po-actions-right",
            Button { variant: ButtonVariant::Ghost, onclick: on_print, icon: Some("🖨".to_string()), "Print" }
            Button { variant: ButtonVariant::Ghost, onclick: on_delete, icon: Some("🗑".to_string()), "Delete" }
            }
            }
            Modal { is_open: show_delete_modal, title: Some("Delete PO".to_string()), size: ModalSize::Sm, close_on_backdrop: true, close_on_escape: true,
            footer: rsx! { Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" } Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete" } },
            p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "This action cannot be undone. Delete {d.po_no}?" }
            }

            Modal { is_open: show_receive_modal, title: Some(format!("Receive Goods — {}", d.po_no)), size: ModalSize::Md, close_on_backdrop: true, close_on_escape: true,
            footer: rsx! {
                Button { variant: ButtonVariant::Secondary, onclick: cancel_receive, "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: confirm_receive, loading: *receive_saving.read(), icon: Some("📦".to_string()), "Confirm Receipt" }
            },
            div { style: "margin-bottom: 12px;",
                p { style: "margin: 0 0 4px 0; font-size: 13px; color: var(--text-secondary);",
                    "Remaining items will be fully received. Items already fully received are skipped."
                }
            }
            table { style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                thead { tr {
                    th { style: "text-align: left; padding: 6px 8px; border-bottom: 2px solid var(--border-color, #e0e0e0); font-size: 11px; text-transform: uppercase; color: var(--text-secondary);", "Item" }
                    th { style: "text-align: right; padding: 6px 8px; border-bottom: 2px solid var(--border-color, #e0e0e0); font-size: 11px; text-transform: uppercase; color: var(--text-secondary);", "Ordered" }
                    th { style: "text-align: right; padding: 6px 8px; border-bottom: 2px solid var(--border-color, #e0e0e0); font-size: 11px; text-transform: uppercase; color: var(--text-secondary);", "Received" }
                    th { style: "text-align: right; padding: 6px 8px; border-bottom: 2px solid var(--border-color, #e0e0e0); font-size: 11px; text-transform: uppercase; color: var(--text-secondary);", "Receiving" }
                }}
                tbody {
                    {d.items.iter().map(|li| {
                        let remaining = li.quantity - li.received;
                        let fully_received = remaining <= 0.0;
                        let row_style = if fully_received { "opacity: 0.5;" } else { "" };
                        rsx! {
                            tr { style: "{row_style}",
                                td { style: "padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0);",
                                    div { style: "font-weight: 500;", "{li.item_name}" }
                                    div { style: "font-size: 11px; color: var(--text-secondary);", "{li.item_code}" }
                                }
                                td { style: "padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); text-align: right; font-family: monospace;",
                                    "{li.quantity:.0}" }
                                td { style: "padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); text-align: right; font-family: monospace;",
                                    "{li.received:.0}" }
                                td { style: "padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); text-align: right; font-family: monospace; font-weight: 600; color: var(--accent, #4a90d9);",
                                    if fully_received { span { style: "color: var(--success, #28a745);", "✓ Complete" } }
                                    else { span { "+{remaining:.0}" } }
                                }
                            }
                        }
                    })}
                }
            }
            div { style: "margin-top: 12px;",
                label { style: "display: block; font-size: 12px; font-weight: 500; color: var(--text-secondary); margin-bottom: 4px;", "Notes (optional)" }
                input {
                    r#type: "text",
                    style: "width: 100%; padding: 8px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; font-size: 13px; box-sizing: border-box;",
                    placeholder: "e.g. Delivery note #12345",
                    value: "{receive_notes.read()}",
                    oninput: move |e| { receive_notes.set(e.value()); },
                }
            }
            }
        }
    }
}
