//! Production Order Create Page — Form to create a new production order.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

const PAGE_CSS: &str = r##"
.prd-create-page {
    max-width: 800px;
    margin: 0 auto;
}
.prd-create-header {
    display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px;
}
.prd-create-header h1 {
    font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary);
}
.prd-back-link {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px;
}
.prd-back-link:hover { text-decoration: underline; }
.prd-section {
    background: #fff; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px;
}
.prd-section h2 {
    font-size: 15px; font-weight: 600; color: var(--text-primary);
    margin: 0 0 16px 0; padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}
.prd-form-row {
    display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap;
}
.prd-form-row > * { flex: 1; min-width: 180px; }
.prd-action-bar {
    display: flex; justify-content: flex-end; gap: 8px;
    margin-top: 20px; padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}
@media (max-width: 768px) {
    .prd-form-row { flex-direction: column; }
    .prd-form-row > * { min-width: 100%; }
    .prd-action-bar { flex-direction: column; }
}
"##;

static NEXT_PRD_ID: AtomicU32 = AtomicU32::new(8);

fn generate_prd_no() -> String {
    let seq = NEXT_PRD_ID.fetch_add(1, Ordering::Relaxed);
    format!("PRD-2026-{:04}", seq)
}

fn finished_item_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "ITM-0001".to_string(), label: "Premium Widget Alpha".to_string() },
        SelectOption { value: "ITM-0004".to_string(), label: "Steel Bracket XR-200".to_string() },
        SelectOption { value: "ITM-0005".to_string(), label: "Rubber Gasket Set".to_string() },
        SelectOption { value: "ITM-0008".to_string(), label: "Assembly Kit Type-B".to_string() },
        SelectOption { value: "ITM-0012".to_string(), label: "Control Panel CX-12".to_string() },
        SelectOption { value: "ITM-0015".to_string(), label: "Hydraulic Pump HP-45".to_string() },
    ]
}

fn bom_options_by_item(item_code: &str) -> Vec<SelectOption> {
    match item_code {
        "ITM-0001" => vec![
            SelectOption { value: "BOM-0001".to_string(), label: "BOM-0001 (v1.2) - Premium Widget Alpha".to_string() },
        ],
        "ITM-0004" => vec![
            SelectOption { value: "BOM-0002".to_string(), label: "BOM-0002 (v1.0) - Steel Bracket XR-200".to_string() },
        ],
        "ITM-0005" => vec![
            SelectOption { value: "BOM-0003".to_string(), label: "BOM-0003 (v0.9) - Rubber Gasket Set".to_string() },
        ],
        "ITM-0008" => vec![
            SelectOption { value: "BOM-0004".to_string(), label: "BOM-0004 (v2.1) - Assembly Kit Type-B".to_string() },
        ],
        "ITM-0012" => vec![
            SelectOption { value: "BOM-0005".to_string(), label: "BOM-0005 (v1.0) - Control Panel CX-12".to_string() },
        ],
        "ITM-0015" => vec![
            SelectOption { value: "BOM-0006".to_string(), label: "BOM-0006 (v1.3) - Hydraulic Pump HP-45".to_string() },
        ],
        _ => vec![],
    }
}

#[component]
pub fn ProductionCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let prd_no = use_signal(|| generate_prd_no());
    let item_to_produce = use_signal(String::new);
    let bom = use_signal(String::new);
    let planned_qty = use_signal(|| "100".to_string());
    let start_date = use_signal(String::new);
    let expected_end_date = use_signal(String::new);
    let notes = use_signal(String::new);

    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    let bom_opts = bom_options_by_item(&item_to_produce.read().clone());
    let bom_disabled = item_to_produce.read().is_empty();

    let validate = {
        let mut item = item_to_produce.clone();
        let mut bom = bom.clone();
        let mut qty = planned_qty.clone();
        let mut start = start_date.clone();
        let mut end = expected_end_date.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if item.read().is_empty() { errs.insert("item", "Item is required.".to_string()); }
            if bom.read().is_empty() { errs.insert("bom", "BOM is required.".to_string()); }
            if let Ok(q) = qty.read().parse::<i32>() {
                if q <= 0 { errs.insert("qty", "Must be > 0.".to_string()); }
            } else { errs.insert("qty", "Invalid number.".to_string()); }
            if start.read().is_empty() { errs.insert("start", "Start date is required.".to_string()); }
            if end.read().is_empty() { errs.insert("end", "End date is required.".to_string()); }
            let is_valid = errs.is_empty();
            if !is_valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            is_valid
        }
    };

    let on_item_change = {
        let mut item = item_to_produce.clone();
        let mut bom = bom.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| {
            item.set(v.clone());
            bom.set(String::new());
            dirty.set(true);
        }
    };

    let on_bom_change = {
        let mut b = bom.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { b.set(v); dirty.set(true); }
    };

    let on_qty_change = {
        let mut q = planned_qty.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { q.set(v); dirty.set(true); }
    };

    let on_start_change = {
        let mut d = start_date.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { d.set(v); dirty.set(true); }
    };

    let on_end_change = {
        let mut d = expected_end_date.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { d.set(v); dirty.set(true); }
    };

    let on_notes_change = {
        let mut n = notes.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { n.set(v); dirty.set(true); }
    };

    let save_prd = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut pn = prd_no.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let no = pn.read().clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("Production Order Created", &format!("{} has been created.", no));
                saving.set(false);
                dirty.set(false);
                nav.push("/manufacturing/production");
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut pn = prd_no.clone();
        let mut item = item_to_produce.clone();
        let mut bom = bom.clone();
        let mut qty = planned_qty.clone();
        let mut start = start_date.clone();
        let mut end = expected_end_date.clone();
        let mut notes = notes.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let no = pn.read().clone();
            let mut toast = toast.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("Production Order Created", &format!("{} created. Creating another...", no));
                pn.set(generate_prd_no());
                item.set(String::new());
                bom.set(String::new());
                qty.set("100".to_string());
                start.set(String::new());
                end.set(String::new());
                notes.set(String::new());
                saving.set(false);
                dirty.set(false);
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let mut dirty = is_dirty.clone();
        let mut nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/manufacturing/production"); }
        }
    };

    let confirm_discard = {
        let mut nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/manufacturing/production"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page prd-create-page",
            div { class: "prd-create-header",
                div {
                    a { class: "prd-back-link", href: "/manufacturing/production", "← Back to Production Orders" }
                    h1 { "New Production Order" }
                }
                if *is_dirty.read() {
                    span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" }
                }
            }

            div { class: "prd-section",
                h2 { "Order Information" }
                div { class: "prd-form-row",
                    FormInput {
                        label: Some("Production No".to_string()),
                        value: prd_no.read().clone(),
                        oninput: move |_| {},
                        r#type: InputType::Text,
                        disabled: true,
                        hint: Some("Auto-generated".to_string()),
                    }
                }
                div { class: "prd-form-row", style: "margin-top: 12px;",
                    SearchableSelect {
                        options: finished_item_options(),
                        selected_value: Some(item_to_produce.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_item_change,
                        placeholder: "Select item to produce...",
                        searchable: true,
                        class: Some("cb-input-group".to_string()),
                    }
                }
            }

            div { class: "prd-section",
                h2 { "BOM & Quantities" }
                div { class: "prd-form-row",
                    SearchableSelect {
                        options: bom_opts,
                        selected_value: Some(bom.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_bom_change,
                        placeholder: if bom_disabled { "Select item first..." } else { "Select BOM..." },
                        searchable: true,
                        disabled: bom_disabled,
                        class: Some("cb-input-group".to_string()),
                    }
                    FormInput {
                        label: Some("Planned Quantity".to_string()),
                        value: planned_qty.read().clone(),
                        oninput: on_qty_change,
                        r#type: InputType::Number,
                        placeholder: Some("100".to_string()),
                        min: Some(1.0),
                        step: Some(1.0),
                    }
                }
            }

            div { class: "prd-section",
                h2 { "Schedule" }
                div { class: "prd-form-row",
                    FormInput {
                        label: Some("Start Date".to_string()),
                        value: start_date.read().clone(),
                        oninput: on_start_change,
                        r#type: InputType::Date,
                    }
                    FormInput {
                        label: Some("Expected End Date".to_string()),
                        value: expected_end_date.read().clone(),
                        oninput: on_end_change,
                        r#type: InputType::Date,
                    }
                }
            }

            div { class: "prd-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes_change,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes about this production order...".to_string()),
                    hint: Some("Internal notes.".to_string()),
                }
            }

            div { class: "prd-action-bar",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: open_discard,
                    disabled: *is_saving.read(),
                    "Discard"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: save_and_new,
                    loading: *is_saving.read(),
                    icon: Some("💾".to_string()),
                    "Save & New"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: save_prd,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Production Order"
                }
            }

            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_discard, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_discard, "Discard" }
                },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;",
                    "You have unsaved changes. Are you sure you want to discard this production order?"
                }
            }
        }
    }
}
