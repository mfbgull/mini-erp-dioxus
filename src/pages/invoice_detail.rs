//! Invoice Detail Page — View a single invoice with header info, KPI cards,
//! line items table, and action bar with delete confirmation.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::components::invoice::{InvoicePaymentPanel, PaymentInfo};
use crate::models as models;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.invoice-detail-page { max-width: 960px; margin: 0 auto; }

.invoice-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.invoice-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.invoice-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.invoice-detail-back:hover { text-decoration: underline; }
.invoice-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.invoice-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }

.invoice-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.invoice-status-paid { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.invoice-status-unpaid { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.invoice-status-partial { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.invoice-status-overdue { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.invoice-status-cancelled { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.invoice-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.invoice-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.invoice-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.invoice-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.invoice-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.invoice-field { display: flex; flex-direction: column; gap: 3px; }
.invoice-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.invoice-field-value { font-size: 14px; color: var(--text-primary); }
.invoice-field-value.monospace { font-family: monospace; font-size: 13px; }

.invoice-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }

.invoice-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.invoice-items-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.invoice-items-table thead th.text-right { text-align: right; }
.invoice-items-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.invoice-items-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.invoice-items-table tbody tr:last-child td { border-bottom: none; }
.invoice-items-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.invoice-items-table tfoot td { padding: 8px 10px; font-weight: 600; font-size: 13px; border-top: 2px solid var(--border-color, #e0e0e0); }
.invoice-items-table tfoot td.text-right { text-align: right; font-family: monospace; font-size: 12px; }

.invoice-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.invoice-actions-left, .invoice-actions-right { display: flex; align-items: center; gap: 8px; }

.invoice-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.invoice-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: inv-spin 0.8s linear infinite; }
@keyframes inv-spin { to { transform: rotate(360deg); } }

@media (max-width: 768px) {
    .invoice-detail-header { flex-direction: column; }
    .invoice-detail-title-row { flex-direction: column; align-items: flex-start; }
    .invoice-detail-kpis { grid-template-columns: 1fr 1fr; }
    .invoice-info-grid { grid-template-columns: 1fr; }
    .invoice-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct InvoiceLineItem {
    line_no: i32,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    discount: f64,
    tax_rate: f64,
    net_amount: f64,
    item_id: Option<i64>,
}

#[derive(Clone, Debug)]
struct ReturnItemInput {
    item_id: i64,
    item_name: String,
    max_quantity: f64,
    return_quantity: String,
}

#[derive(Clone, Debug)]
struct InvoiceDetail {
    id: i64,
    invoice_no: String,
    customer_id: i64,
    customer_name: String,
    customer_code: String,
    invoice_date: String,
    due_date: String,
    status: String,
    total_amount: f64,
    paid_amount: f64,
    balance_amount: f64,
    discount_percent: f64,
    tax_rate: f64,
    notes: String,
    source_type: String,
    items: Vec<InvoiceLineItem>,
}


fn status_class(status: &str) -> &'static str {
    match status {
        "Paid" => "invoice-status-paid",
        "Unpaid" => "invoice-status-unpaid",
        "Partially Paid" => "invoice-status-partial",
        "Overdue" => "invoice-status-overdue",
        "Cancelled" => "invoice-status-cancelled",
        _ => "invoice-status-unpaid",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn InvoiceDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let api = use_auth().api;
    let invoice_resource = use_resource(move || {
        let api = api.clone();
        let id = id.clone();
        async move {
            let parsed = id.parse::<i64>().ok()?;
            let client = api.with(|c| c.clone());
            let result = client.get_invoice(parsed).await.ok()?;
            let data = result.get("data")?;
            let inv: models::Invoice = serde_json::from_value(data.get("invoice")?.clone()).ok()?;
            let items: Vec<models::InvoiceItem> = serde_json::from_value(data.get("items")?.clone()).ok()?;
            Some(InvoiceDetail {
                id: inv.id,
                invoice_no: inv.invoice_no,
                customer_id: inv.customer_id,
                customer_name: inv.customer_name.clone().unwrap_or_default(),
                customer_code: inv.customer_code.clone().unwrap_or_default(),
                invoice_date: inv.invoice_date.clone(),
                due_date: inv.due_date.clone(),
                status: inv.status.clone(),
                total_amount: inv.total_amount,
                paid_amount: inv.paid_amount,
                balance_amount: inv.balance_amount,
                discount_percent: inv.discount_value.unwrap_or(0.0),
                tax_rate: inv.tax_rate.unwrap_or(0.0),
                notes: inv.notes.clone().unwrap_or_default(),
                source_type: inv.source_type.clone(),
                items: items.into_iter().enumerate().map(|(i, ii)| InvoiceLineItem {
                    line_no: (i + 1) as i32,
                    item_code: ii.item_code.clone().unwrap_or_default(),
                    item_name: ii.item_name.clone().unwrap_or_default(),
                    quantity: ii.quantity,
                    unit_price: ii.unit_price,
                    discount: ii.discount_value.unwrap_or(0.0),
                    tax_rate: ii.tax_rate,
                    net_amount: ii.amount,
                    item_id: Some(ii.item_id),
                }).collect(),
            })
        }
    });

    let is_loading = invoice_resource.read().is_none();
    let inv_opt = invoice_resource.read().as_ref().cloned().flatten();
    let mut show_delete_modal = use_signal(|| false);
    let mut show_payment_modal = use_signal(|| false);
    let mut show_return_modal = use_signal(|| false);
    let mut return_items = use_signal(|| Vec::<ReturnItemInput>::new());
    let mut payment_info = use_signal(|| PaymentInfo::default());
    let is_saving = use_signal(|| false);
    let counter = use_signal(|| 0u32);

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page invoice-detail-page",
                div { class: "invoice-loading",
                    div { class: "loading-spinner" }
                    span { "Loading invoice details…" }
                }
            }
        };
    }
    if inv_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page invoice-detail-page",
                div { class: "invoice-loading",
                    div { style: "font-size: 40px;", "🧾" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Invoice Not Found" }
                    p { "No invoice with ID \"{id_display}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { navigator.push("/sales/invoices"); }, "\u{2190} Back to Invoices" }
                }
            }
        };
    }
    let inv = inv_opt.as_ref().unwrap();
    let status_cls = status_class(&inv.status);
    let on_back = { let nav = navigator.clone(); move |_| { nav.push("/sales/invoices"); } };
    let on_print = { let nav = navigator.clone(); let i = inv.id; move |_| { nav.push(format!("/sales/invoices/{}/print", i)); } };
    let on_edit = { let nav = navigator.clone(); let i = inv.id; move |_| { nav.push(format!("/sales/invoices/{}/edit", i)); } };
    let on_payment = { let mut modal = show_payment_modal.clone(); let mut pay = payment_info.clone(); let total = inv.balance_amount; move |_| { pay.set(PaymentInfo { record_payment: true, amount: total, ..PaymentInfo::default() }); modal.set(true); } };
    let on_delete = { let mut m = show_delete_modal.clone(); move |_| m.set(true) };
    let confirm_delete = { let nav = navigator.clone(); let mut m = show_delete_modal.clone(); let mut t = toast.clone(); move |_| { m.set(false); t.success("Deleted", "Invoice has been deleted."); nav.push("/sales/invoices"); } };
    let cancel_delete = { let mut m = show_delete_modal.clone(); move |_| m.set(false) };
    let cancel_payment = { let mut m = show_payment_modal.clone(); move |_| m.set(false) };
    let submit_payment = {
        let api = api.clone();
        let mut toast = toast.clone();
        let mut modal = show_payment_modal.clone();
        let inv_id = inv.id;
        let cust_id = inv.customer_id;
        let inv_no = inv.invoice_no.clone();
        let mut pay_sig = payment_info.clone();
        let mut saving_sig = is_saving.clone();
        let mut refresh = invoice_resource.clone();
        move |_| {
            let pay = pay_sig.read().clone();
            if !pay.record_payment || pay.amount <= 0.0 {
                toast.error("Validation Error", "Please enter a valid payment amount.");
                return;
            }
            saving_sig.set(true);
            let client = api.with(|c| c.clone());
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let body = serde_json::json!({
                "customer_id": cust_id,
                "invoice_id": inv_id,
                "payment_date": today,
                "amount": pay.amount,
                "payment_method": pay.method,
                "reference": pay.reference,
                "notes": pay.notes,
            });
            let mut toast2 = toast.clone();
            let inv_no2 = inv_no.clone();
            spawn(async move {
                match client.create_payment(&body).await {
                    Ok(_) => {
                        toast2.success("Payment Recorded", &format!("Payment for {} recorded successfully.", inv_no2));
                        modal.set(false);
                        pay_sig.set(PaymentInfo::default());
                        saving_sig.set(false);
                        refresh.restart();
                    }
                    Err(e) => {
                        toast2.error("Payment Failed", &e);
                        saving_sig.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page invoice-detail-page",
    // ── Header ──
                    div { class: "invoice-detail-header",
                        div { class: "invoice-detail-title-group",
                            Button { class: Some("invoice-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: on_back, "← Back to Invoices" }
                            div { class: "invoice-detail-title-row",
                                h1 { "Invoice {inv.invoice_no}" }
                                span { class: "invoice-status-badge {status_cls}", "{inv.status}" }
                            }
                        }
                    }

                    // ── KPI Cards ──
                    div { class: "invoice-detail-kpis",
                        StatCard {
                            title: "Total Amount".to_string(),
                            value: format!("PKR {:.0}", inv.total_amount),
                            variant: StatCardVariant::Primary,
                            icon: Some("💰".to_string()),
                        }
                        StatCard {
                            title: "Paid".to_string(),
                            value: format!("PKR {:.0}", inv.paid_amount),
                            variant: if inv.paid_amount == inv.total_amount { StatCardVariant::Success } else { StatCardVariant::Warning },
                            icon: Some("✅".to_string()),
                        }
                        StatCard {
                            title: "Balance".to_string(),
                            value: format!("PKR {:.0}", inv.balance_amount),
                            variant: if inv.balance_amount > 0.0 { StatCardVariant::Danger } else { StatCardVariant::Success },
                            icon: Some("⚠".to_string()),
                        }
                        StatCard {
                            title: format!("Tax ({:.0}%)", inv.tax_rate),
                            value: format!("PKR {:.0}", inv.total_amount * inv.tax_rate / 100.0),
                            variant: StatCardVariant::Default,
                            icon: Some("📊".to_string()),
                        }
                    }

                    // ── Section: Invoice Details ──
                    div { class: "invoice-section",
                        div { class: "invoice-section-header",
                            h2 { "Invoice Details" }
                        }
                        div { class: "invoice-info-grid",
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Customer" }
                                span { class: "invoice-field-value", "{inv.customer_name} ({inv.customer_code})" }
                            }
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Invoice Date" }
                                span { class: "invoice-field-value", "{inv.invoice_date}" }
                            }
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Due Date" }
                                span { class: "invoice-field-value", "{inv.due_date}" }
                            }
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Source" }
                                span { class: "invoice-field-value", "{inv.source_type}" }
                            }
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Discount" }
                                span { class: "invoice-field-value monospace", "{inv.discount_percent:.0}%" }
                            }
                            div { class: "invoice-field",
                                span { class: "invoice-field-label", "Tax Rate" }
                                span { class: "invoice-field-value monospace", "{inv.tax_rate:.0}%" }
                            }
                        }
                    }

                    // ── Section: Line Items ──
                    div { class: "invoice-section",
                        div { class: "invoice-section-header",
                            h2 { "Line Items" }
                            span { style: "font-size: 12px; color: var(--text-secondary);", "{inv.items.len()} item(s)" }
                        }
                        table { class: "invoice-items-table",
                            thead { tr {
                                th { "#" }
                                th { "Item Code" }
                                th { "Item Name" }
                                th { class: "text-right", "Qty" }
                                th { class: "text-right", "Rate" }
                                th { class: "text-right", "Disc" }
                                th { class: "text-right", "Tax" }
                                th { class: "text-right", "Amount" }
                            } }
                            tbody {
                                {inv.items.iter().map(|li| {
                                    rsx! {
                                        tr {
                                            td { "{li.line_no}" }
                                            td { style: "font-family: monospace;", "{li.item_code}" }
                                            td { "{li.item_name}" }
                                            td { class: "text-right", "{li.quantity:.0}" }
                                            td { class: "text-right", "PKR {li.unit_price:.2}" }
                                            td { class: "text-right", "{li.discount:.0}%" }
                                            td { class: "text-right", "{li.tax_rate:.0}%" }
                                            td { class: "text-right", "PKR {li.net_amount:.2}" }
                                        }
                                    }
                                })}
                            }
                            tfoot {
                                tr {
                                    td { colspan: "7", style: "text-align: right; padding-right: 10px;", "Total" }
                                    td { class: "text-right", "PKR {inv.total_amount:.2}" }
                                }
                                if inv.discount_percent > 0.0 {
                                    tr {
                                        td { colspan: "7", style: "text-align: right; padding-right: 10px;", "Discount ({inv.discount_percent:.0}%)" }
                                        td { class: "text-right", "-PKR {inv.total_amount * inv.discount_percent / 100.0:.2}" }
                                    }
                                }
                                tr {
                                    td { colspan: "7", style: "text-align: right; padding-right: 10px; font-weight: 700;", "Grand Total" }
                                    td { class: "text-right", style: "font-size: 14px; font-weight: 700;", "PKR {inv.total_amount:.0}" }
                                }
                            }
                        }
                    }

                    // ── Section: Notes ──
                    if !inv.notes.is_empty() {
                        div { class: "invoice-section",
                            div { class: "invoice-section-header",
                                h2 { "Notes" }
                            }
                            p { class: "invoice-notes", "{inv.notes}" }
                        }
                    }

                    // ── Action Bar ──
                    div { class: "invoice-actions",
                        div { class: "invoice-actions-left",
                            Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit" }
                            Button { variant: ButtonVariant::Secondary, onclick: on_print, icon: Some("🖨".to_string()), "Print" }
                            Button { variant: ButtonVariant::Secondary, onclick: on_payment, icon: Some("💰".to_string()), "Record Payment" }
                            if inv.status != "Cancelled" {
                                Button { variant: ButtonVariant::Warning, onclick: { let mut show = show_return_modal.clone(); let inv_clone = inv.clone(); let items_clone = inv.items.clone(); move |_| {
                                    // Initialize return items
                                    let returnInputs: Vec<ReturnItemInput> = items_clone.iter().map(|item| ReturnItemInput {
                                        item_id: item.item_id.unwrap_or(0),
                                        item_name: item.item_name.clone(),
                                        max_quantity: item.quantity,
                                        return_quantity: String::new(),
                                    }).collect();
                                    return_items.set(returnInputs);
                                    show.set(true);
                                }}, icon: Some("↩".to_string()), "Return" }
                            }
                        }
                        div { class: "invoice-actions-right",
                            Button { variant: ButtonVariant::Ghost, onclick: on_delete, icon: Some("🗑️".to_string()), "Delete" }
                        }
                    }

                    // ── Delete Confirmation Modal ──
                    Modal {
                        is_open: show_delete_modal,
                        title: Some("Delete Invoice".to_string()),
                        size: ModalSize::Sm,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                            Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete Invoice" }
                        },
                        div {
                            p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;", "Delete {inv.invoice_no}?" }
                            p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;", "This action cannot be undone." }
                        }
                    }

                    // ── Payment Modal ──
                    Modal {
                        is_open: show_payment_modal,
                        title: Some(format!("Record Payment — {}", inv.invoice_no)),
                        size: ModalSize::Md,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: cancel_payment, "Cancel" }
                            Button { variant: ButtonVariant::Primary, loading: *is_saving.read(), onclick: submit_payment, "Record Payment" }
                        },
                        InvoicePaymentPanel {
                            total: inv.balance_amount,
                            payment: payment_info.read().clone(),
                            on_change: move |p| { payment_info.set(p); },
                        }
                    }

                    // ── Return Modal ──
                    Modal {
                        is_open: show_return_modal,
                        title: Some(format!("Return Items — {}", inv.invoice_no)),
                        size: ModalSize::Md,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: move |_| show_return_modal.set(false), "Cancel" }
                            Button {
                                variant: ButtonVariant::Warning,
                                onclick: { let api = use_auth().api; let mut toast = toast.clone(); let mut show = show_return_modal.clone(); let items = return_items.clone(); let invoice_id = inv.id.clone(); let mut counter = counter.clone(); move |_| {
                                    let returnData: Vec<serde_json::Value> = items.read().iter().filter_map(|item| {
                                        let qty: f64 = item.return_quantity.parse().ok().unwrap_or(0.0);
                                        if qty > 0.0 && qty <= item.max_quantity {
                                            Some(serde_json::json!({ "item_id": item.item_id, "quantity": qty }))
                                        } else {
                                            None
                                        }
                                    }).collect();
                                    if returnData.is_empty() {
                                        toast.error("Error", "No items selected for return.");
                                        return;
                                    }
                                    let api = api.clone();
                                    let mut toast = toast.clone();
                                    let mut show = show.clone();
                                    let mut counter = counter.clone();
                                    let body = serde_json::json!({ "items": returnData });
                                    spawn(async move {
                                        let client = api.read().clone();
                                        match client.return_invoice(invoice_id, &body).await {
                                            Ok(_) => {
                                                toast.success("Return Recorded", "Items have been returned and stock updated.");
                                                show.set(false);
                                                let current = *counter.read();
                                                counter.set(current + 1);
                                            }
                                            Err(e) => toast.error("Error", &e),
                                        }
                                    });
                                }},
                                "Process Return"
                            }
                        },
                        div { style: "display: flex; flex-direction: column; gap: 12px;",
                            p { style: "margin: 0; font-size: 13px; color: var(--text-secondary);", "Enter the quantity to return for each item. Leave blank or 0 to skip." }
                            for (idx, item) in return_items.read().iter().enumerate() {{
                                let idx = idx;
                                rsx! {
                                    div { style: "display: flex; align-items: center; gap: 12px; padding: 8px; background: var(--bg-muted, #f8f9fa); border-radius: 6px;",
                                        div { style: "flex: 1;",
                                            div { style: "font-weight: 500; font-size: 13px;", "{item.item_name}" }
                                            div { style: "font-size: 11px; color: var(--text-secondary);", "Max: {item.max_quantity:.0}" }
                                        }
                                        input {
                                            r#type: "number",
                                            step: "1",
                                            min: "0",
                                            max: "{item.max_quantity}",
                                            placeholder: "0",
                                            value: "{item.return_quantity}",
                                            style: "width: 80px; padding: 6px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; text-align: right; font-family: monospace;",
                                            onchange: { let mut items = return_items.clone(); move |e| {
                                                items.write()[idx].return_quantity = e.value();
                                            }}
                                        }
                                    }
                                }
                            }}
                        }
                    }

        }
    }
}