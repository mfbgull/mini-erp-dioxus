//! Production Order Detail Page — View a production order with status, kpis, materials, and actions.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant,
    use_toast,
};
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.prd-detail-page {
    max-width: 960px;
    margin: 0 auto;
}
.prd-detail-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    margin-bottom: 20px; gap: 16px; flex-wrap: wrap;
}
.prd-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.prd-detail-back {
    display: inline-flex; align-items: center; gap: 4px; font-size: 13px;
    color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px;
    cursor: pointer; background: none; border: none; padding: 0;
}
.prd-detail-back:hover { text-decoration: underline; }
.prd-detail-title-row {
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
}
.prd-detail-title-row h1 {
    font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0;
}
.prd-detail-code {
    font-family: monospace; font-size: 13px; color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px;
}
.badge {
    display: inline-flex; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600;
}
.badge-success { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.badge-primary { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.badge-warning { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.badge-danger { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.badge-secondary { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.prd-detail-kpis {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px; margin-bottom: 20px;
}
.prd-detail-section {
    background: #fff; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px;
}
.prd-detail-section-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 16px; padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}
.prd-detail-section-header h2 {
    font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0;
}
.prd-detail-section-header .section-badge {
    font-size: 11px; color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 10px;
}
.prd-detail-info-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
}
.prd-detail-field {
    display: flex; flex-direction: column; gap: 3px;
}
.prd-detail-field-label {
    font-size: 11px; font-weight: 600; color: var(--text-secondary);
    text-transform: uppercase; letter-spacing: 0.3px;
}
.prd-detail-field-value { font-size: 14px; color: var(--text-primary); }
.prd-detail-field-value.monospace { font-family: monospace; font-size: 13px; }
.prd-comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.prd-comp-table thead th {
    text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px;
    text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary);
    border-bottom: 2px solid var(--border-color, #e0e0e0);
}
.prd-comp-table thead th.text-right { text-align: right; }
.prd-comp-table tbody td {
    padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}
.prd-comp-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.prd-comp-table tbody tr:last-child td { border-bottom: none; }
.prd-comp-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.prd-detail-notes {
    font-size: 13px; color: var(--text-secondary); line-height: 1.6;
    padding: 12px; background: var(--bg-muted, #f5f5f5);
    border-radius: 6px; margin: 0;
}
.prd-detail-actions {
    display: flex; align-items: center; justify-content: space-between;
    gap: 8px; margin-top: 20px; padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap;
}
.prd-detail-actions-left, .prd-detail-actions-right {
    display: flex; align-items: center; gap: 8px;
}
.prd-detail-loading {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary);
}
.loading-spinner {
    width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9); border-radius: 50%;
    animation: prd-spin 0.8s linear infinite;
}
@keyframes prd-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) {
    .prd-detail-header { flex-direction: column; }
    .prd-detail-title-row { flex-direction: column; align-items: flex-start; }
    .prd-detail-kpis { grid-template-columns: 1fr 1fr; }
    .prd-detail-info-grid { grid-template-columns: 1fr; }
    .prd-detail-actions { flex-direction: column; align-items: stretch; }
    .prd-detail-actions-left, .prd-detail-actions-right { justify-content: center; }
}
"##;

#[derive(Clone, Debug)]
struct MatConsumed {
    item_code: String,
    item_name: String,
    required_qty: f64,
    uom: String,
    issued_qty: f64,
}

#[derive(Clone, Debug)]
struct ProductionDetail {
    id: i64,
    prd_no: String,
    item_name: String,
    item_code: String,
    bom_code: String,
    planned_qty: i32,
    completed_qty: i32,
    scrap_qty: i32,
    start_date: String,
    end_date: String,
    status: String,
    notes: String,
    materials: Vec<MatConsumed>,
}

fn fetch_production_detail(id: &str) -> Option<ProductionDetail> {
    let parsed = id.parse::<i64>().unwrap_or(0);
    match parsed {
        3 => Some(ProductionDetail {
            id: 3,
            prd_no: "PRD-2026-0008".to_string(),
            item_name: "Rubber Gasket Set".to_string(),
            item_code: "ITM-0005".to_string(),
            bom_code: "BOM-0003".to_string(),
            planned_qty: 1000,
            completed_qty: 620,
            scrap_qty: 15,
            start_date: "2026-06-15".to_string(),
            end_date: "2026-06-30".to_string(),
            status: "In Progress".to_string(),
            notes: "High-priority order for Beta Industries. Quality check at 500 units.".to_string(),
            materials: vec![
                MatConsumed { item_code: "ITM-0023".to_string(), item_name: "Rubber Sheet 5mm".to_string(), required_qty: 100.0, uom: "sheets".to_string(), issued_qty: 70.0 },
                MatConsumed { item_code: "ITM-0025".to_string(), item_name: "Bolt M8 x 30mm".to_string(), required_qty: 4000.0, uom: "pcs".to_string(), issued_qty: 2500.0 },
                MatConsumed { item_code: "ITM-0026".to_string(), item_name: "Nut M8".to_string(), required_qty: 4000.0, uom: "pcs".to_string(), issued_qty: 2500.0 },
            ],
        }),
        1 => Some(ProductionDetail {
            id: 1,
            prd_no: "PRD-2026-0007".to_string(),
            item_name: "Premium Widget Alpha".to_string(),
            item_code: "ITM-0001".to_string(),
            bom_code: "BOM-0001".to_string(),
            planned_qty: 500,
            completed_qty: 500,
            scrap_qty: 8,
            start_date: "2026-06-10".to_string(),
            end_date: "2026-06-20".to_string(),
            status: "Completed".to_string(),
            notes: "Completed ahead of schedule. Yield rate: 98.4%.".to_string(),
            materials: vec![
                MatConsumed { item_code: "ITM-0020".to_string(), item_name: "Steel Plate 6mm".to_string(), required_qty: 1000.0, uom: "sheets".to_string(), issued_qty: 1000.0 },
                MatConsumed { item_code: "ITM-0021".to_string(), item_name: "Aluminum Rod 20mm".to_string(), required_qty: 750.0, uom: "meters".to_string(), issued_qty: 750.0 },
                MatConsumed { item_code: "ITM-0025".to_string(), item_name: "Bolt M8 x 30mm".to_string(), required_qty: 4000.0, uom: "pcs".to_string(), issued_qty: 4040.0 },
                MatConsumed { item_code: "ITM-0026".to_string(), item_name: "Nut M8".to_string(), required_qty: 4000.0, uom: "pcs".to_string(), issued_qty: 4040.0 },
                MatConsumed { item_code: "ITM-0027".to_string(), item_name: "Washer M8".to_string(), required_qty: 8000.0, uom: "pcs".to_string(), issued_qty: 8080.0 },
                MatConsumed { item_code: "ITM-0024".to_string(), item_name: "Brass Fitting Set".to_string(), required_qty: 500.0, uom: "pcs".to_string(), issued_qty: 505.0 },
            ],
        }),
        4 => Some(ProductionDetail {
            id: 4,
            prd_no: "PRD-2026-0009".to_string(),
            item_name: "Assembly Kit Type-B".to_string(),
            item_code: "ITM-0008".to_string(),
            bom_code: "BOM-0004".to_string(),
            planned_qty: 300,
            completed_qty: 0,
            scrap_qty: 0,
            start_date: "2026-06-28".to_string(),
            end_date: "2026-07-10".to_string(),
            status: "Planned".to_string(),
            notes: "New product launch. First production run.".to_string(),
            materials: vec![
                MatConsumed { item_code: "ITM-0020".to_string(), item_name: "Steel Plate 6mm".to_string(), required_qty: 300.0, uom: "sheets".to_string(), issued_qty: 0.0 },
                MatConsumed { item_code: "ITM-0025".to_string(), item_name: "Bolt M8 x 30mm".to_string(), required_qty: 1200.0, uom: "pcs".to_string(), issued_qty: 0.0 },
            ],
        }),
        _ => None,
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Completed" => "badge badge-success",
        "In Progress" => "badge badge-primary",
        "Planned" => "badge badge-warning",
        "Cancelled" => "badge badge-danger",
        _ => "badge badge-secondary",
    }
}

fn efficiency(completed: i32, scrap: i32, planned: i32) -> f64 {
    if planned > 0 {
        ((completed - scrap) as f64 / planned as f64) * 100.0
    } else { 0.0 }
}

#[component]
pub fn ProductionDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let id_clone = id.clone();
    let detail_resource = use_resource(move || {
        let id_for_fetch = id_clone.clone();
        async move {
            crate::utils::sleep(std::time::Duration::from_millis(500)).await;
            fetch_production_detail(&id_for_fetch)
        }
    });

    let is_loading = detail_resource.read().is_none();
    let detail_opt = detail_resource.read().as_ref().cloned().flatten();
    let mut show_delete_modal = use_signal(|| false);
    let show_complete_modal = use_signal(|| false);
    let show_cancel_modal = use_signal(|| false);

    if detail_opt.is_none() && !is_loading {
        return rsx! {
            div { class: "page prd-detail-page",
                div { class: "prd-detail-loading",
                    div { style: "font-size: 40px;", "⚙" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Production Order Not Found" }
                    p { "No production order with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { navigator.push("/manufacturing/production"); }, "← Back to Orders" }
                }
            }
        };
    }

    if is_loading {
        return rsx! {
            div { class: "page prd-detail-page",
                div { class: "prd-detail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading production order details…" }
                }
            }
        };
    }

    let detail = detail_opt.unwrap();
    let status_class = status_badge_class(&detail.status);
    let eff = efficiency(detail.completed_qty, detail.scrap_qty, detail.planned_qty);

    let on_back = move |_: Event<MouseData>| { navigator.push("/manufacturing/production"); };

    let mut t_edit = toast.clone();
    let on_edit = {
        let nav = navigator.clone();
        let id = id.clone();
        move |_| { nav.push(format!("/manufacturing/production/{}", id)); t_edit.info("Edit Mode", "Editing coming soon."); }
    };

    let on_update_progress = {
        let mut toast = toast.clone();
        let mut d = detail.clone();
        move |_| { toast.info("Update Progress", "Update progress feature coming soon."); }
    };

    let on_complete = {
        let mut modal = show_complete_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_complete = {
        let mut toast = toast.clone();
        let mut modal = show_complete_modal.clone();
        let pn = detail.prd_no.clone();
        move |_| {
            modal.set(false);
            toast.success("Order Completed", &format!("{} has been marked as complete.", pn));
        }
    };

    let cancel_complete = {
        let mut modal = show_complete_modal.clone();
        move |_| { modal.set(false); }
    };

    let on_cancel = {
        let mut modal = show_cancel_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_cancel = {
        let mut toast = toast.clone();
        let mut modal = show_cancel_modal.clone();
        let pn = detail.prd_no.clone();
        move |_| {
            modal.set(false);
            toast.warning("Order Cancelled", &format!("{} has been cancelled.", pn));
        }
    };

    let cancel_cancel = {
        let mut modal = show_cancel_modal.clone();
        move |_| { modal.set(false); }
    };

    let on_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_delete = {
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut modal = show_delete_modal.clone();
        move |_| {
            modal.set(false);
            toast.success("Order Deleted", "Production order has been deleted.");
            nav.push("/manufacturing/production");
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page prd-detail-page",
            div { class: "prd-detail-header",
                div { class: "prd-detail-title-group",
                    button { class: "prd-detail-back", r#type: "button", onclick: on_back, "← Back to Orders" }
                    div { class: "prd-detail-title-row",
                        h1 { "{detail.item_name}" }
                        span { class: "prd-detail-code", "{detail.prd_no}" }
                        span { class: "{status_class}", "{detail.status}" }
                    }
                }
            }

            div { class: "prd-detail-kpis",
                StatCard {
                    title: "Planned Quantity".to_string(),
                    value: format!("{} units", detail.planned_qty),
                    variant: StatCardVariant::Primary,
                    icon: Some("📋".to_string()),
                }
                StatCard {
                    title: "Completed".to_string(),
                    value: format!("{} units", detail.completed_qty),
                    variant: if detail.completed_qty >= detail.planned_qty { StatCardVariant::Success } else { StatCardVariant::Primary },
                    icon: Some("✅".to_string()),
                }
                StatCard {
                    title: "Scrap".to_string(),
                    value: format!("{} units", detail.scrap_qty),
                    variant: if detail.scrap_qty > 0 { StatCardVariant::Warning } else { StatCardVariant::Default },
                    icon: Some("🔧".to_string()),
                }
                StatCard {
                    title: "Efficiency".to_string(),
                    value: format!("{:.1}%", eff),
                    variant: if eff >= 98.0 { StatCardVariant::Success }
                             else if eff >= 90.0 { StatCardVariant::Primary }
                             else { StatCardVariant::Warning },
                    icon: Some("🎯".to_string()),
                    footer: Some(format!("Target: ≥ 95%")),
                }
            }

            div { class: "prd-detail-section",
                div { class: "prd-detail-section-header",
                    h2 { "Order Information" }
                    span { class: "section-badge", "Details" }
                }
                div { class: "prd-detail-info-grid",
                    div { class: "prd-detail-field",
                        span { class: "prd-detail-field-label", "Item Code" }
                        span { class: "prd-detail-field-value monospace", "{detail.item_code}" }
                    }
                    div { class: "prd-detail-field",
                        span { class: "prd-detail-field-label", "BOM Reference" }
                        span { class: "prd-detail-field-value monospace", "{detail.bom_code}" }
                    }
                    div { class: "prd-detail-field",
                        span { class: "prd-detail-field-label", "Start Date" }
                        span { class: "prd-detail-field-value", "{detail.start_date}" }
                    }
                    div { class: "prd-detail-field",
                        span { class: "prd-detail-field-label", "End Date" }
                        span { class: "prd-detail-field-value", "{detail.end_date}" }
                    }
                    div { class: "prd-detail-field",
                        span { class: "prd-detail-field-label", "Status" }
                        span { class: "prd-detail-field-value", "{detail.status}" }
                    }
                }
            }

            div { class: "prd-detail-section",
                div { class: "prd-detail-section-header",
                    h2 { "Materials Consumed" }
                    span { class: "section-badge", "{detail.materials.len()} items" }
                }
                table { class: "prd-comp-table",
                    thead {
                        tr {
                            th { "Item Code" }
                            th { "Item Name" }
                            th { class: "text-right", "Required Qty" }
                            th { "UOM" }
                            th { class: "text-right", "Issued Qty" }
                            th { class: "text-right", "Status" }
                        }
                    }
                    tbody {
                        {detail.materials.iter().map(|m| {
                            let issued_pct = if m.required_qty > 0.0 {
                                (m.issued_qty / m.required_qty) * 100.0
                            } else { 0.0 };
                            let status_text = if issued_pct >= 100.0 { "✅ Full" }
                                              else if issued_pct > 0.0 { "⚠ Partial" }
                                              else { "⏳ Pending" };
                            rsx! {
                                tr {
                                    td { span { class: "prd-detail-code", "{m.item_code}" } }
                                    td { "{m.item_name}" }
                                    td { class: "text-right", "{m.required_qty}" }
                                    td { "{m.uom}" }
                                    td { class: "text-right", "{m.issued_qty}" }
                                    td { class: "text-right", "{status_text}" }
                                }
                            }
                        })}
                    }
                }
            }

            if !detail.notes.is_empty() {
                div { class: "prd-detail-section",
                    div { class: "prd-detail-section-header",
                        h2 { "Notes" }
                    }
                    p { class: "prd-detail-notes", "{detail.notes}" }
                }
            }

            div { class: "prd-detail-actions",
                div { class: "prd-detail-actions-left",
                    Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit" }
                    Button { variant: ButtonVariant::Secondary, onclick: on_update_progress, icon: Some("📊".to_string()), "Update Progress" }
                    if detail.status != "Completed" && detail.status != "Cancelled" {
                        Button { variant: ButtonVariant::Success, onclick: on_complete, icon: Some("✅".to_string()), "Complete" }
                    }
                    if detail.status != "Cancelled" && detail.status != "Completed" {
                        Button { variant: ButtonVariant::Warning, onclick: on_cancel, icon: Some("⛔".to_string()), "Cancel" }
                    }
                }
                div { class: "prd-detail-actions-right",
                    Button { variant: ButtonVariant::Ghost, onclick: on_delete, icon: Some("🗑️".to_string()), "Delete" }
                }
            }

            Modal {
                is_open: show_complete_modal,
                title: Some("Complete Order".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_complete, "Cancel" }
                    Button { variant: ButtonVariant::Success, onclick: confirm_complete, "Complete Order" }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Mark {detail.prd_no} as complete?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This will set the order status to Completed. Completed quantity: {detail.completed_qty}/{detail.planned_qty}."
                    }
                }
            }

            Modal {
                is_open: show_cancel_modal,
                title: Some("Cancel Order".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_cancel, "Keep Order" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_cancel, "Cancel Order" }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Cancel {detail.prd_no}?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This will cancel the production order. Any materials already issued will remain allocated."
                    }
                }
            }

            Modal {
                is_open: show_delete_modal,
                title: Some("Delete Order".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete" }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Are you sure you want to delete {detail.prd_no}?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This action cannot be undone."
                    }
                }
            }
        }
    }
}
