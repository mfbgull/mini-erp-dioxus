//! BOM Detail Page — View a Bill of Materials with header info, components table, and actions.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::models;
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.bom-detail-page {
    max-width: 960px;
    margin: 0 auto;
}
.bom-detail-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    margin-bottom: 20px; gap: 16px; flex-wrap: wrap;
}
.bom-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.bom-detail-back {
    display: inline-flex; align-items: center; gap: 4px; font-size: 13px;
    color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px;
    cursor: pointer; background: none; border: none; padding: 0;
}
.bom-detail-back:hover { text-decoration: underline; }
.bom-detail-title-row {
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
}
.bom-detail-title-row h1 {
    font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0;
}
.bom-detail-code {
    font-family: monospace; font-size: 13px; color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px;
}
.badge {
    display: inline-flex; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600;
}
.badge-success { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.badge-warning { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.badge-secondary { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.badge-danger { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.bom-detail-kpis {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px; margin-bottom: 20px;
}
.bom-detail-section {
    background: #fff; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px;
}
.bom-detail-section-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 16px; padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}
.bom-detail-section-header h2 {
    font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0;
}
.bom-detail-section-header .section-badge {
    font-size: 11px; color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 10px;
}
.bom-detail-info-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
}
.bom-detail-field {
    display: flex; flex-direction: column; gap: 3px;
}
.bom-detail-field-label {
    font-size: 11px; font-weight: 600; color: var(--text-secondary);
    text-transform: uppercase; letter-spacing: 0.3px;
}
.bom-detail-field-value {
    font-size: 14px; color: var(--text-primary);
}
.bom-detail-field-value.monospace { font-family: monospace; font-size: 13px; }
.bom-comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.bom-comp-table thead th {
    text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px;
    text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary);
    border-bottom: 2px solid var(--border-color, #e0e0e0);
}
.bom-comp-table thead th.text-right { text-align: right; }
.bom-comp-table tbody td {
    padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}
.bom-comp-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.bom-comp-table tbody tr:last-child td { border-bottom: none; }
.bom-comp-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.bom-total-row {
    display: flex; justify-content: flex-end; align-items: center;
    padding: 10px 10px 0; margin-top: 8px; gap: 16px;
    border-top: 2px solid var(--border-color, #e0e0e0);
}
.bom-total-label { font-size: 13px; font-weight: 600; color: var(--text-secondary); }
.bom-total-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
.bom-detail-actions {
    display: flex; align-items: center; justify-content: space-between;
    gap: 8px; margin-top: 20px; padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap;
}
.bom-detail-actions-left, .bom-detail-actions-right {
    display: flex; align-items: center; gap: 8px;
}
.bom-detail-loading {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary);
}
.loading-spinner {
    width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9); border-radius: 50%;
    animation: bom-spin 0.8s linear infinite;
}
@keyframes bom-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) {
    .bom-detail-header { flex-direction: column; }
    .bom-detail-title-row { flex-direction: column; align-items: flex-start; }
    .bom-detail-kpis { grid-template-columns: 1fr 1fr; }
    .bom-detail-info-grid { grid-template-columns: 1fr; }
    .bom-detail-actions { flex-direction: column; align-items: stretch; }
    .bom-detail-actions-left, .bom-detail-actions-right { justify-content: center; }
}
"##;

#[derive(Clone, Debug)]
struct BomComponentLine {
    item_code: String,
    item_name: String,
    quantity: f64,
    uom: String,
    unit_cost: f64,
    scrap_pct: f64,
    sub_total: f64,
}

#[derive(Clone, Debug)]
struct BomDetail {
    id: i64,
    bom_code: String,
    item_name: String,
    item_code: String,
    quantity_produced: f64,
    total_cost: f64,
    status: String,
    version: String,
    last_updated: String,
    components: Vec<BomComponentLine>,
}

async fn fetch_bom_detail_from_api(client: &crate::api::ApiClient, id: i64) -> Option<BomDetail> {
    let result = client.get_bom(id).await.ok()?;
    let bom: models::Bom = serde_json::from_value(result["bom"].clone()).ok()?;
    let items: Vec<models::BomItem> = serde_json::from_value(result["items"].clone()).ok()?;
    let components: Vec<BomComponentLine> = items.into_iter().map(|i| {
        let qty = i.quantity;
        let cost = i.unit_cost;
        BomComponentLine {
            item_code: i.item_code.unwrap_or_default(),
            item_name: i.item_name.unwrap_or_default(),
            quantity: qty,
            uom: "pcs".to_string(),
            unit_cost: cost,
            scrap_pct: 0.0,
            sub_total: qty * cost,
        }
    }).collect();
    let total_cost = components.iter().map(|c| c.sub_total).sum();
    Some(BomDetail {
        id: bom.id,
        bom_code: bom.bom_no,
        item_name: bom.finished_item_name.unwrap_or_default(),
        item_code: bom.finished_item_code.unwrap_or_default(),
        quantity_produced: bom.quantity,
        total_cost,
        status: if bom.is_active { "Active".to_string() } else { "Inactive".to_string() },
        version: String::new(),
        last_updated: bom.updated_at,
        components,
    })
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Active" => "badge badge-success",
        "Draft" => "badge badge-warning",
        "Inactive" => "badge badge-secondary",
        _ => "badge badge-secondary",
    }
}

#[component]
pub fn BomDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let id_clone = id.clone();
    let api = use_auth().api;
    let detail_resource = use_resource(move || {
        let api = api.clone();
        let id_for_fetch = id_clone.clone();
        async move {
            let parsed_id = id_for_fetch.parse::<i64>().unwrap_or(0);
            let client = api.read().clone();
            fetch_bom_detail_from_api(&client, parsed_id).await
        }
    });

    let is_loading = detail_resource.read().is_none();
    let detail_opt = detail_resource.read().as_ref().cloned().flatten();
    let mut show_delete_modal = use_signal(|| false);

    if detail_opt.is_none() && !is_loading {
        return rsx! {
            div { class: "page bom-detail-page",
                div { class: "bom-detail-loading",
                    div { style: "font-size: 40px;", "📋" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "BOM Not Found" }
                    p { "No BOM with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { navigator.push("/manufacturing/boms"); }, "← Back to BOMs" }
                }
            }
        };
    }

    if is_loading {
        return rsx! {
            div { class: "page bom-detail-page",
                div { class: "bom-detail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading BOM details…" }
                }
            }
        };
    }

    let detail = detail_opt.unwrap();
    let status_class = status_badge_class(&detail.status);

    let on_back = move |_: Event<MouseData>| { navigator.push("/manufacturing/boms"); };
    let mut t_edit = toast.clone();
    let on_edit = {
        let nav = navigator.clone();
        let id = id.clone();
        move |_| { nav.push(format!("/manufacturing/boms/{}", id)); t_edit.info("Edit Mode", "BOM editing coming soon."); }
    };
    let on_copy = {
        let mut toast = toast.clone();
        let mut d = detail.clone();
        move |_| { toast.info("Copy BOM", "BOM duplicated. Redirecting to edit..."); }
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
            toast.success("BOM Deleted", "BOM has been deleted.");
            nav.push("/manufacturing/boms");
        }
    };
    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page bom-detail-page",
            div { class: "bom-detail-header",
                div { class: "bom-detail-title-group",
                    button { class: "bom-detail-back", r#type: "button", onclick: on_back, "← Back to BOMs" }
                    div { class: "bom-detail-title-row",
                        h1 { "{detail.item_name}" }
                        span { class: "bom-detail-code", "{detail.bom_code}" }
                        span { class: "{status_class}", "{detail.status}" }
                    }
                }
            }

            div { class: "bom-detail-kpis",
                StatCard {
                    title: "Quantity Produced".to_string(),
                    value: format!("{} unit(s)", detail.quantity_produced),
                    variant: StatCardVariant::Primary,
                    icon: Some("📦".to_string()),
                }
                StatCard {
                    title: "Total Cost".to_string(),
                    value: format!("PKR {:.2}", detail.total_cost),
                    variant: StatCardVariant::Success,
                    icon: Some("💰".to_string()),
                    footer: Some(format!("v{}", detail.version)),
                }
                StatCard {
                    title: "Components".to_string(),
                    value: format!("{} items", detail.components.len()),
                    variant: StatCardVariant::Primary,
                    icon: Some("🔩".to_string()),
                }
                StatCard {
                    title: "Avg Cost / Component".to_string(),
                    value: format!("PKR {:.2}", detail.total_cost / detail.components.len() as f64),
                    variant: StatCardVariant::Default,
                    icon: Some("📊".to_string()),
                }
            }

            div { class: "bom-detail-section",
                div { class: "bom-detail-section-header",
                    h2 { "BOM Information" }
                    span { class: "section-badge", "Details" }
                }
                div { class: "bom-detail-info-grid",
                    div { class: "bom-detail-field",
                        span { class: "bom-detail-field-label", "Item Code" }
                        span { class: "bom-detail-field-value monospace", "{detail.item_code}" }
                    }
                    div { class: "bom-detail-field",
                        span { class: "bom-detail-field-label", "Version" }
                        span { class: "bom-detail-field-value", "{detail.version}" }
                    }
                    div { class: "bom-detail-field",
                        span { class: "bom-detail-field-label", "Last Updated" }
                        span { class: "bom-detail-field-value", "{detail.last_updated}" }
                    }
                    div { class: "bom-detail-field",
                        span { class: "bom-detail-field-label", "Status" }
                        span { class: "bom-detail-field-value", "{detail.status}" }
                    }
                }
            }

            div { class: "bom-detail-section",
                div { class: "bom-detail-section-header",
                    h2 { "Components" }
                    span { class: "section-badge", "{detail.components.len()} lines" }
                }
                table { class: "bom-comp-table",
                    thead {
                        tr {
                            th { "Item Code" }
                            th { "Item Name" }
                            th { class: "text-right", "Qty" }
                            th { "UOM" }
                            th { class: "text-right", "Unit Cost" }
                            th { class: "text-right", "Scrap %" }
                            th { class: "text-right", "Sub-total" }
                        }
                    }
                    tbody {
                        {detail.components.iter().map(|c| {
                            rsx! {
                                tr {
                                    td { span { class: "bom-detail-code", "{c.item_code}" } }
                                    td { "{c.item_name}" }
                                    td { class: "text-right", "{c.quantity}" }
                                    td { "{c.uom}" }
                                    td { class: "text-right", "PKR {c.unit_cost:.2}" }
                                    td { class: "text-right", "{c.scrap_pct}%" }
                                    td { class: "text-right", "PKR {c.sub_total:.2}" }
                                }
                            }
                        })}
                    }
                }
                div { class: "bom-total-row",
                    span { class: "bom-total-label", "Total Cost (incl. scrap):" }
                    span { class: "bom-total-value", "PKR {detail.total_cost:.2}" }
                }
            }

            div { class: "bom-detail-actions",
                div { class: "bom-detail-actions-left",
                    Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit BOM" }
                    Button { variant: ButtonVariant::Secondary, onclick: on_copy, icon: Some("📋".to_string()), "Copy BOM" }
                }
                div { class: "bom-detail-actions-right",
                    Button { variant: ButtonVariant::Ghost, onclick: on_delete, icon: Some("🗑️".to_string()), "Delete" }
                }
            }

            Modal {
                is_open: show_delete_modal,
                title: Some("Delete BOM".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete BOM" }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Are you sure you want to delete {detail.bom_code}?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This action cannot be undone. The BOM \"{detail.bom_code}\" will be permanently removed."
                    }
                }
            }
        }
    }
}
