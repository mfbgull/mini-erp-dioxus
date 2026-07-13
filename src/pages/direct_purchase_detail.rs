//! Direct Purchase Detail Page — Detail view for a direct purchase with
//! header info, line items table, and action bar.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::models;
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.dp-detail-page { max-width: 1000px; margin: 0 auto; }
.dp-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.dp-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.dp-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.dp-detail-back:hover { text-decoration: underline; }
.dp-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.dp-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.dp-detail-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.dp-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.dp-status-draft { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.dp-status-approved { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.dp-status-received { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.dp-status-cancelled { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.dp-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.dp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.dp-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.dp-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.dp-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.dp-field { display: flex; flex-direction: column; gap: 3px; }
.dp-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.dp-field-value { font-size: 14px; color: var(--text-primary); }
.dp-field-value.monospace { font-family: monospace; font-size: 13px; }
.dp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.dp-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.dp-table thead th.text-right { text-align: right; }
.dp-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.dp-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.dp-table tbody tr:last-child td { border-bottom: none; }
.dp-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.dp-actions-left, .dp-actions-right { display: flex; align-items: center; gap: 8px; }
.dp-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.dp-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: dp-spin 0.8s linear infinite; }
@keyframes dp-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .dp-detail-header { flex-direction: column; } .dp-detail-kpis { grid-template-columns: 1fr 1fr; } .dp-info-grid { grid-template-columns: 1fr; } .dp-actions { flex-direction: column; align-items: stretch; } }
"##;

#[derive(Clone, Debug)]
struct LineItem {
    item_code: String,
    item_name: String,
    quantity: f64,
    rate: f64,
    discount: f64,
    tax: f64,
    amount: f64,
}

#[derive(Clone, Debug)]
struct PurchaseDetail {
    id: i64,
    dp_no: String,
    supplier_name: String,
    supplier_code: String,
    date: String,
    status: String,
    items: Vec<LineItem>,
    subtotal: f64,
    discount_total: f64,
    tax_total: f64,
    grand_total: f64,
    notes: String,
}



fn status_class(s: &str) -> &'static str {
    match s { "Draft" => "dp-status-draft", "Approved" => "dp-status-approved", "Received" => "dp-status-received", "Cancelled" => "dp-status-cancelled", _ => "dp-status-draft" }
}

#[component]
pub fn DirectPurchaseDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        let pid = id.clone();
        async move {
            let parsed_id = pid.parse::<i64>().ok()?;
            let result = api.read().clone().get_direct_purchase(parsed_id).await;
            match result {
                Ok(m) => {
                    let line_item = LineItem {
                        item_code: m.item_code.clone().unwrap_or_default(),
                        item_name: m.item_name.clone().unwrap_or_default(),
                        quantity: m.quantity,
                        rate: m.unit_cost,
                        discount: 0.0,
                        tax: 0.0,
                        amount: m.total_cost,
                    };
                    Some(PurchaseDetail {
                        id: m.id,
                        dp_no: m.purchase_no,
                        supplier_name: m.supplier_name,
                        supplier_code: String::new(), // ponytail: server model doesn't have supplier_code
                        date: m.purchase_date,
                        status: m.status,
                        items: vec![line_item], // ponytail: server model is single-line, show as one item
                        subtotal: m.total_cost,
                        discount_total: 0.0,
                        tax_total: 0.0,
                        grand_total: m.total_cost,
                        notes: m.notes.unwrap_or_default(),
                    })
                }
                Err(_) => None,
            }
        }
    });

    let loading = resource.read().is_none();
    let detail_opt = resource.read().as_ref().and_then(|d| d.clone());
    let mut show_delete_modal = use_signal(|| false);

    // Extract detail fields for use in RSX
    let (d, detail_ready) = if let Some(ref d) = detail_opt {
        (Some(d.clone()), true)
    } else {
        (None, false)
  };

    let on_back = move |_| { navigator.push("/purchases/direct"); };
    let on_edit = { let mut t = toast.clone(); move |_| t.info("Edit Mode", "Direct purchase editing coming soon.") };
    let on_receipt = { let mut t = toast.clone(); move |_| t.info("Goods Receipt", "Record receipt coming soon.") };
    let on_print = { let mut t = toast.clone(); move |_| t.info("Print", "Print view coming soon.") };
    let on_delete_prompt = { let mut m = show_delete_modal.clone(); move |_| m.set(true) };
    let confirm_delete = {
        let mut m = show_delete_modal.clone();
        let mut t = toast.clone();
        let nav = navigator.clone();
        let api = api.clone();
        let pid = id_display.clone();
        move |_| {
            m.set(false);
            let mut t = t.clone();
            let nav = nav.clone();
            let api = api.clone();
            let pid = pid.clone();
            spawn(async move {
                let parsed = pid.parse::<i64>().unwrap_or(0);
                let client = api.with(|c| c.clone());
                match client.delete_direct_purchase(parsed).await {
                    Ok(_) => {
                        t.success("Deleted", "Direct purchase deleted.");
                        nav.push("/purchases/direct");
                    }
                    Err(e) => {
                        t.error("Error", &format!("Failed to delete: {}", e));
                    }
                }
            });
        }
    };


    if loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page dp-detail-page",
                div { class: "dp-loading", div { class: "loading-spinner" } span { "Loading purchase details…" } }
            }
        };
    }
    if !detail_ready {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page dp-detail-page",
                div { class: "dp-loading",
                    div { style: "font-size: 40px;", "📥" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Purchase Not Found" }
                    p { "No direct purchase with ID \"{id_display}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: on_back, "← Back" }
                }
            }
        };
    }
    let detail_data = detail_opt.as_ref().cloned().unwrap();
    let sc = status_class(&detail_data.status);
    render_detail(detail_data, sc, on_back, on_edit, on_receipt, on_print, on_delete_prompt, confirm_delete, show_delete_modal)

}

fn render_detail(
    d: PurchaseDetail,
    sc: &'static str,
    mut on_back: impl FnMut(Event<MouseData>) + 'static,
    mut on_edit: impl FnMut(Event<MouseData>) + 'static,
    mut on_receipt: impl FnMut(Event<MouseData>) + 'static,
    mut on_print: impl FnMut(Event<MouseData>) + 'static,
    mut on_delete_prompt: impl FnMut(Event<MouseData>) + 'static,
    mut confirm_delete: impl FnMut(Event<MouseData>) + 'static,
    mut show_delete_modal: Signal<bool>,
) -> Element {
    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page dp-detail-page",
        div { class: "dp-detail-header",
            div { class: "dp-detail-title-group",
                Button { class: Some("dp-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: on_back, "← Back to Direct Purchases" }
                div { class: "dp-detail-title-row",
                    h1 { "Direct Purchase {d.dp_no}" }
                    span { class: "dp-detail-code", "{d.supplier_code}" }
                    span { class: "dp-status-badge {sc}", "{d.status}" }
                }
            }
        }

        div { class: "dp-detail-kpis",
            StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.0}", d.subtotal), variant: StatCardVariant::Default }
            StatCard { title: "Discount".to_string(), value: format!("PKR {:.0}", d.discount_total), variant: if d.discount_total > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
            StatCard { title: "Tax".to_string(), value: format!("PKR {:.0}", d.tax_total), variant: StatCardVariant::Default }
            StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.0}", d.grand_total), variant: StatCardVariant::Primary }
        }

        div { class: "dp-section",
            div { class: "dp-section-header", h2 { "Purchase Information" } }
            div { class: "dp-info-grid",
                div { class: "dp-field", span { class: "dp-field-label", "Supplier" } span { class: "dp-field-value", "{d.supplier_name}" } }
                div { class: "dp-field", span { class: "dp-field-label", "Supplier Code" } span { class: "dp-field-value monospace", "{d.supplier_code}" } }
                div { class: "dp-field", span { class: "dp-field-label", "Purchase Date" } span { class: "dp-field-value", "{d.date}" } }
                div { class: "dp-field", span { class: "dp-field-label", "Status" } span { class: "dp-field-value", "{d.status}" } }
            }
        }

        div { class: "dp-section",
            div { class: "dp-section-header", h2 { "Line Items" } }
            table { class: "dp-table",
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
            div { class: "dp-section",
                div { class: "dp-section-header", h2 { "Notes" } }
                p { style: "font-size: 13px; color: var(--text-secondary); line-height: 1.6; margin: 0;", "{d.notes}" }
            }
        }

        div { class: "dp-actions",
            div { class: "dp-actions-left",
                Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit" }
                Button { variant: ButtonVariant::Secondary, onclick: on_receipt, icon: Some("📦".to_string()), "Record Receipt" }
            }
            div { class: "dp-actions-right",
                Button { variant: ButtonVariant::Ghost, onclick: on_print, icon: Some("🖨".to_string()), "Print" }
                Button { variant: ButtonVariant::Ghost, onclick: on_delete_prompt, icon: Some("🗑".to_string()), "Delete" }
            }
        }

        Modal { is_open: show_delete_modal, title: Some("Delete Purchase".to_string()), size: ModalSize::Sm, close_on_backdrop: true, close_on_escape: true,
            footer: rsx! { Button { variant: ButtonVariant::Secondary, onclick: move |_| show_delete_modal.set(false), "Cancel" } Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete" } },
            p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "This action cannot be undone. Delete {d.dp_no}?" }
        }
        }
    }
}
