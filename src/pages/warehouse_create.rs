//! Warehouse Create Page — Form to create a new warehouse location using the
//! common UI component library (FormInput, Button, Toast, Modal).

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, Modal, ModalSize, use_toast,
};
use crate::models::WarehouseForm;
use dioxus::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.warehouse-create-page {
    max-width: 800px;
    margin: 0 auto;
}

.warehouse-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.warehouse-create-header h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
}

.warehouse-back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
    margin-bottom: 16px;
}

.warehouse-back-link:hover { text-decoration: underline; }

.warehouse-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.warehouse-section h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.warehouse-form-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.warehouse-form-row > * {
    flex: 1;
    min-width: 180px;
}

.warehouse-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border: 1.5px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: 13px;
    color: var(--text-primary);
    background: #fff;
    user-select: none;
}

.warehouse-toggle-btn:hover {
    border-color: var(--accent, #4a90d9);
    background: rgba(74, 144, 217, 0.04);
}

.warehouse-toggle-btn-active {
    border-color: var(--accent, #4a90d9);
    background: rgba(74, 144, 217, 0.08);
    color: var(--accent, #4a90d9);
    font-weight: 600;
}

.warehouse-action-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}

@media (max-width: 768px) {
    .warehouse-form-row { flex-direction: column; }
    .warehouse-form-row > * { min-width: 100%; }
    .warehouse-action-bar { flex-direction: column; }
}
"##;

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn WarehouseCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    // ── Form State ──
    let mut warehouse_code = use_signal(String::new);
    let warehouse_name = use_signal(String::new);
    let location = use_signal(String::new);
    let mut is_active = use_signal(|| true);

    // UI state
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    // ── Validation ──
    let validate = {
        let name = warehouse_name.clone();
        let loc = location.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if name.read().trim().is_empty() {
                errs.insert("name", "Warehouse name is required.".to_string());
            }
            let is_valid = errs.is_empty();
            if !is_valid {
                toast.warning("Validation Error", "Please fix the highlighted fields.");
            }
            is_valid
        }
    };

    // ── Handlers ──

    let on_name_change = {
        let mut name = warehouse_name.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { name.set(v); dirty.set(true); }
    };

    let on_location_change = {
        let mut loc = location.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { loc.set(v); dirty.set(true); }
    };

    // Save
    let save_warehouse = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let api = api.clone();
        let nav = navigator.clone();
        let name = warehouse_name.clone();
        let code = warehouse_code.clone();
        let loc = location.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let n = name.read().clone();
            let c = code.read().clone();
            let l = loc.read().clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let api = api.clone();

            spawn(async move {
                let client = api.with(|c| c.clone());
                let form = WarehouseForm {
                    warehouse_code: c,
                    warehouse_name: n.clone(),
                    location: if l.trim().is_empty() { None } else { Some(l) },
                };
                match client.create_warehouse(&form).await {
                    Ok(_wh) => {
                        toast.success("Warehouse Created", &format!("{} has been created.", n));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/inventory/warehouses");
                    }
                    Err(err) => {
                        toast.error("Error", &err);
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Save & New
    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let api = api.clone();
        let name = warehouse_name.clone();
        let code = warehouse_code.clone();
        let loc = location.clone();
        let mut validate = validate.clone();
        let mut w_name = warehouse_name.clone();
        let mut w_location = location.clone();
        let mut w_code = warehouse_code.clone();
        let mut w_active = is_active.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let n = name.read().clone();
            let c = code.read().clone();
            let l = loc.read().clone();
            let mut toast = toast.clone();
            let api = api.clone();

            spawn(async move {
                let client = api.with(|c| c.clone());
                let form = WarehouseForm {
                    warehouse_code: c,
                    warehouse_name: n.clone(),
                    location: if l.trim().is_empty() { None } else { Some(l) },
                };
                match client.create_warehouse(&form).await {
                    Ok(_) => {
                        toast.success("Warehouse Created", &format!("{} created. Creating another…", n));
                        w_code.set(String::new());
                        w_name.set(String::new());
                        w_location.set(String::new());
                        w_active.set(true);
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(err) => {
                        toast.error("Error", &err);
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Discard
    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let dirty = is_dirty.clone();
        let nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/inventory/warehouses"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/inventory/warehouses"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    // ── Derived ──
    let name_err = errors.read().get("name").cloned();
    let active_icon = if *is_active.read() { "✅" } else { "⛔" };
    let active_label = if *is_active.read() { "Active" } else { "Inactive" };
    let toggle_class = if *is_active.read() { "warehouse-toggle-btn warehouse-toggle-btn-active" } else { "warehouse-toggle-btn" };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page warehouse-create-page",

            // ── Header ──
            div { class: "warehouse-create-header",
                div {
                    a {
                        class: "warehouse-back-link",
                        href: "/inventory/warehouses",
                        "← Back to Warehouses"
                    }
                    h1 { "New Warehouse" }
                }
                if *is_dirty.read() {
                    span {
                        style: "font-size: 12px; color: var(--warning); font-weight: 500;",
                        "⚠ Unsaved changes"
                    }
                }
            }

            // ── Section: Basic Info ──
            div { class: "warehouse-section",
                h2 { "Basic Information" }
                div { class: "warehouse-form-row",
                    FormInput {
                        label: Some("Warehouse Code".to_string()),
                        value: warehouse_code.read().clone(),
                        oninput: move |v| { warehouse_code.set(v); is_dirty.set(true); },
                        r#type: InputType::Text,
                        placeholder: Some("e.g. WH-001".to_string()),
                        hint: Some("Unique warehouse identifier".to_string()),
                    }
                    FormInput {
                        label: Some("Warehouse Name".to_string()),
                        value: warehouse_name.read().clone(),
                        oninput: on_name_change,
                        r#type: InputType::Text,
                        placeholder: Some("Enter warehouse name".to_string()),
                        required: true,
                        error: name_err,
                    }
                }
                div { class: "warehouse-form-row", style: "margin-top: 12px;",
                    FormInput {
                        label: Some("Location".to_string()),
                        value: location.read().clone(),
                        oninput: on_location_change,
                        r#type: InputType::Text,
                        placeholder: Some("e.g. Building A, Floor 1".to_string()),
                        required: true,
                        error: None,
                    }
                }
            }

            // ── Section: Status ──
            div { class: "warehouse-section",
                h2 { "Status" }
                div { class: "warehouse-form-row",
                    div {
                        label { class: "cb-input-label", style: "margin-bottom: 6px; display: block;", "Active Status" }
                        button {
                            class: "{toggle_class}",
                            onclick: move |_| {
                                let val = !*is_active.read();
                                is_active.set(val);
                                is_dirty.set(true);
                            },
                            r#type: "button",
                            span { "{active_icon}" }
                            span { "{active_label}" }
                        }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "warehouse-action-bar",
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
                    onclick: save_warehouse,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Warehouse"
                }
            }

            // ── Discard Confirmation Modal ──
            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: cancel_discard,
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        onclick: confirm_discard,
                        "Discard"
                    }
                },
                p {
                    style: "margin: 0; color: var(--text-secondary); font-size: 14px;",
                    "You have unsaved changes. Are you sure you want to discard this warehouse?"
                }
            }
        }
    }
}
