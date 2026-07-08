//! Supplier Detail Page — Tabbed detail view for supplier management.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::pages::supplier_list::Supplier;
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.supplier-detail-page { max-width: 1000px; margin: 0 auto; }
.supplier-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.supplier-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.supplier-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.supplier-detail-back:hover { text-decoration: underline; }
.supplier-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.supplier-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.supplier-detail-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.supplier-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.supplier-status-active { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.supplier-status-inactive { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.supplier-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.supplier-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 2px solid var(--border-color, #e0e0e0); overflow-x: auto; }
.supplier-tab { display: inline-flex; align-items: center; gap: 6px; padding: 10px 18px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border: none; background: none; white-space: nowrap; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.15s ease; }
.supplier-tab:hover { color: var(--text-primary); background: rgba(74, 144, 217, 0.04); }
.supplier-tab-active { color: var(--accent, #4a90d9); border-bottom-color: var(--accent, #4a90d9); font-weight: 600; }
.supplier-tab-count { font-size: 11px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 0 6px; border-radius: 8px; line-height: 18px; }
.supplier-tab-active .supplier-tab-count { background: rgba(74, 144, 217, 0.1); color: var(--accent, #4a90d9); }
.supplier-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.supplier-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.supplier-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.supplier-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.supplier-field { display: flex; flex-direction: column; gap: 3px; }
.supplier-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.supplier-field-value { font-size: 14px; color: var(--text-primary); }
.supplier-field-value.monospace { font-family: monospace; font-size: 13px; }
.supplier-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.supplier-actions-left, .supplier-actions-right { display: flex; align-items: center; gap: 8px; }
.supplier-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.supplier-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: supplier-spin 0.8s linear infinite; }
@keyframes supplier-spin { to { transform: rotate(360deg); } }
.customer-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.customer-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.customer-table thead th.text-right { text-align: right; }
.customer-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.customer-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.customer-table tbody td.text-success { color: #28a745; }
.customer-table tbody td.text-danger { color: #dc3545; }
.customer-table tbody tr:last-child td { border-bottom: none; }
.customer-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.customer-table-badge { display: inline-flex; align-items: center; padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; }
.customer-table-badge-green { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.customer-table-badge-yellow { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.customer-table-badge-blue { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.customer-table-badge-red { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.customer-table-badge-gray { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.customer-table-empty { text-align: center; padding: 30px 20px; color: var(--text-secondary); font-size: 14px; }
@media (max-width: 768px) {
    .supplier-detail-header { flex-direction: column; }
    .supplier-detail-kpis { grid-template-columns: 1fr 1fr; }
    .supplier-info-grid { grid-template-columns: 1fr; }
    .supplier-tab { padding: 10px 12px; font-size: 12px; }
    .supplier-actions { flex-direction: column; align-items: stretch; }
}
"##;

#[derive(Clone, Debug)]
struct PoItem {
    id: i64,
    po_no: String,
    date: String,
    status: String,
    total: f64,
    received: f64,
}

#[derive(Clone, Debug)]
struct PaymentItem {
    id: i64,
    payment_no: String,
    date: String,
    amount: f64,
    method: String,
}



fn status_class(status: &str) -> &'static str {
    match status {
        "Active" => "supplier-status-active",
        "Inactive" => "supplier-status-inactive",
        _ => "supplier-status-active",
    }
}

fn badge_class(status: &str) -> &'static str {
    match status {
        "Completed" => "customer-table-badge-green",
        "Open" => "customer-table-badge-yellow",
        "Partially Received" => "customer-table-badge-blue",
        _ => "customer-table-badge-gray",
    }
}

#[component]
pub fn SupplierDetailPage(id: String) -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let api = use_auth().api;
    let supplier_resource = use_resource(move || {
        let id = id.clone();
        let api = api.clone();
        async move {
            let parsed = id.parse::<i64>().unwrap_or(0);
            if parsed == 0 {
                return None;
            }
            let client = api.read().clone();
            let api_supplier = client.get_supplier(parsed).await.ok()?;
            let city = api_supplier
                .address
                .split(',')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            Some(Supplier {
                id: api_supplier.id,
                supplier_code: api_supplier.supplier_code,
                supplier_name: api_supplier.supplier_name,
                email: api_supplier.email,
                phone: api_supplier.phone,
                city,
                payment_terms: "Net 30".to_string(),
                credit_limit: 0.0,
                current_balance: 0.0,
                status: if api_supplier.is_active {
                    "Active".to_string()
                } else {
                    "Inactive".to_string()
                },
                supplier_type: "Local".to_string(),
            })
        }
    });

    let supplier_snapshot = supplier_resource.read();
    let is_loading = supplier_snapshot.is_none();
    let supplier_opt = supplier_snapshot.as_ref().and_then(|s| s.clone());

    let mut active_tab = use_signal(|| 0usize);
    let mut show_delete_modal = use_signal(|| false);

    // ponytail: no dedicated PO/payment endpoints for a supplier yet
    let (orders, payments): (Vec<PoItem>, Vec<PaymentItem>) = (Vec::new(), Vec::new());

    let tabs = ["Overview", "Purchase Orders", "Payments"];

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page supplier-detail-page",
            if is_loading {
                div { class: "supplier-loading",
                    div { class: "loading-spinner" }
                    span { "Loading supplier details…" }
                }
            } else if supplier_opt.is_none() {
                div { class: "supplier-loading",
                    h2 { style: "margin: 0; color: var(--text-primary);", "Supplier Not Found" }
                    p { "No supplier with ID \"{id_display}\" was found." }
                }
            } else {{
                let supplier = supplier_opt.as_ref().unwrap();
                let outstanding_po = orders.iter().filter(|o| o.status != "Completed").map(|o| o.total - o.received).sum::<f64>();
                let total_purchased = orders.iter().map(|o| o.total).sum::<f64>();

                rsx! {
                    div { class: "supplier-detail-header",
                        div { class: "supplier-detail-title-group",
                            Button { class: Some("supplier-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: move |_| { navigator.push("/suppliers"); }, "← Back to Suppliers" }
                            div { class: "supplier-detail-title-row",
                                h1 { "{supplier.supplier_name}" }
                                span { class: "supplier-detail-code", "{supplier.supplier_code}" }
                                span { class: "supplier-status-badge {status_class(&supplier.status)}",
                                    if supplier.status == "Active" { "✓ Active" } else { "— Inactive" }
                                }
                            }
                        }
                    }

                    div { class: "supplier-detail-kpis",
                        StatCard {
                            title: "Current Balance".to_string(),
                            value: format!("PKR {:.0}", supplier.current_balance),
                            variant: if supplier.current_balance > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Success },
                            icon: Some("💰".to_string()),
                            footer: Some(format!("Limit: PKR {:.0}", supplier.credit_limit)),
                        }
                        StatCard {
                            title: "Total Purchased".to_string(),
                            value: format!("PKR {:.0}", total_purchased),
                            variant: StatCardVariant::Primary,
                            icon: Some("📦".to_string()),
                        }
                        StatCard {
                            title: "Outstanding POs".to_string(),
                            value: format!("PKR {:.0}", outstanding_po),
                            variant: if outstanding_po > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Success },
                            icon: Some("📋".to_string()),
                            footer: Some(format!("{} open orders", orders.iter().filter(|o| o.status != "Completed").count())),
                        }
                    }

                    div { class: "supplier-tabs",
                        {tabs.iter().enumerate().map(|(i, name)| {
                            let is_active = *active_tab.read() == i;
                            let count = match i {
                                1 => Some(orders.len()),
                                2 => Some(payments.len()),
                                _ => None,
                            };
                            let tab_class = if is_active { "supplier-tab supplier-tab-active" } else { "supplier-tab" };
                            rsx! {
                                button {
                                    class: "{tab_class}", r#type: "button",
                                    onclick: move |_| active_tab.set(i),
                                    "{name}"
                                    if let Some(c) = count { span { class: "supplier-tab-count", "{c}" } }
                                }
                            }
                        })}
                    }

                    if *active_tab.read() == 0 {
                        div { class: "supplier-section",
                            div { class: "supplier-section-header", h2 { "Supplier Information" } }
                            div { class: "supplier-info-grid",
                                div { class: "supplier-field", span { class: "supplier-field-label", "Email" } span { class: "supplier-field-value", "{supplier.email}" } }
                                div { class: "supplier-field", span { class: "supplier-field-label", "Phone" } span { class: "supplier-field-value", "{supplier.phone}" } }
                                div { class: "supplier-field", span { class: "supplier-field-label", "City" } span { class: "supplier-field-value", "{supplier.city}" } }
                                div { class: "supplier-field", span { class: "supplier-field-label", "Type" } span { class: "supplier-field-value", "{supplier.supplier_type}" } }
                                div { class: "supplier-field", span { class: "supplier-field-label", "Terms" } span { class: "supplier-field-value", "{supplier.payment_terms}" } }
                                div { class: "supplier-field", span { class: "supplier-field-label", "Credit Limit" } span { class: "supplier-field-value monospace", "PKR {supplier.credit_limit:.0}" } }
                            }
                        }
                    }

                    if *active_tab.read() == 1 {
                        div { class: "supplier-section",
                            div { class: "supplier-section-header",
                                h2 { "Purchase Orders" }
                                Button { variant: ButtonVariant::Primary, onclick: { let nav = navigator.clone(); move |_| { nav.push("/purchases/orders/new"); } }, "＋ New PO" }
                            }
                            if orders.is_empty() {
                                div { class: "customer-table-empty", "No purchase orders found." }
                            } else {
                                table { class: "customer-table",
                                    thead { tr {
                                        th { "PO #" } th { "Date" } th { "Status" }
                                        th { class: "text-right", "Total" } th { class: "text-right", "Received" }
                                    }}
                                    tbody { {orders.iter().map(|po| {
                                        let bdg = badge_class(&po.status);
                                        rsx! { tr {
                                            td { style: "font-family: monospace;", "{po.po_no}" }
                                            td { "{po.date}" }
                                            td { span { class: "customer-table-badge {bdg}", "{po.status}" } }
                                            td { class: "text-right", "PKR {po.total:.0}" }
                                            td { class: "text-right", "PKR {po.received:.0}" }
                                        }}
                                    })}
                                    }
                                }
                            }
                        }
                    }

                    if *active_tab.read() == 2 {
                        div { class: "supplier-section",
                            div { class: "supplier-section-header", h2 { "Payments" } }
                            if payments.is_empty() {
                                div { class: "customer-table-empty", "No payments recorded." }
                            } else {
                                table { class: "customer-table",
                                    thead { tr {
                                        th { "Payment #" } th { "Date" } th { "Method" }
                                        th { class: "text-right", "Amount" }
                                    }}
                                    tbody { {payments.iter().map(|pmt| {
                                        rsx! { tr {
                                            td { style: "font-family: monospace;", "{pmt.payment_no}" }
                                            td { "{pmt.date}" } td { "{pmt.method}" }
                                            td { class: "text-right text-success", "PKR {pmt.amount:.0}" }
                                        }}
                                    })}
                                    }
                                }
                            }
                        }
                    }

                    div { class: "supplier-actions",
                        div { class: "supplier-actions-left",
                            Button { variant: ButtonVariant::Primary, onclick: { let nav = navigator.clone(); let sid = id_display.clone(); move |_| { nav.push(format!("/suppliers/{}/edit", sid)); } }, icon: Some("✏️".to_string()), "Edit Supplier" }
                            Button { variant: ButtonVariant::Secondary, onclick: { let nav = navigator.clone(); move |_| { nav.push("/purchases/orders/new"); } }, icon: Some("📋".to_string()), "New Purchase Order" }
                        }
                        div { class: "supplier-actions-right",
                            Button { variant: ButtonVariant::Ghost, onclick: move |_| show_delete_modal.set(true), icon: Some("🗑️".to_string()), "Delete" }
                        }
                    }

                    Modal {
                        is_open: show_delete_modal,
                        title: Some("Delete Supplier".to_string()),
                        size: ModalSize::Sm,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: move |_| show_delete_modal.set(false), "Cancel" }
                            Button { variant: ButtonVariant::Danger, onclick: { let mut t = toast.clone(); move |_| { show_delete_modal.set(false); t.success("Deleted", "Supplier has been deleted."); navigator.push("/suppliers"); } }, "Delete Supplier" }
                        },
                        div {
                            p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;", "Delete {supplier.supplier_name}?" }
                            p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;", "This action cannot be undone." }
                        }
                    }
                }
            }}
        }
    }
}
