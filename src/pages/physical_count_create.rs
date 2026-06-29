//! Physical Count Create Page — Form to create a new physical inventory count.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, DateRangePicker, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::auth::use_auth;
use chrono::NaiveDate;
use dioxus::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.pc-create-page {
    max-width: 800px;
    margin: 0 auto;
}

.pc-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.pc-create-header h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
}

.pc-back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
    margin-bottom: 16px;
}

.pc-back-link:hover { text-decoration: underline; }

.pc-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.pc-section h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.pc-form-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.pc-form-row > * {
    flex: 1;
    min-width: 180px;
}

.pc-action-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}

@media (max-width: 768px) {
    .pc-form-row { flex-direction: column; }
    .pc-form-row > * { min-width: 100%; }
    .pc-action-bar { flex-direction: column; }
}
"##;

// ============================================================================
// Helpers
// ============================================================================

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn PhysicalCountCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    // ── API ──
    let api = use_auth().api;

    // ── Warehouse options (fetched from API on mount) ──
    let warehouse_list = use_signal(|| Vec::<crate::models::Warehouse>::new());
    {
        let api = api.clone();
        let list = warehouse_list.clone();
        use_effect(move || {
            let api = api.clone();
            let mut list = list.clone();
            spawn(async move {
                let client = api.read().clone();
                if let Ok(whs) = client.list_warehouses().await {
                    list.set(whs);
                }
            });
        });
    }
    let warehouse_options = use_memo(move || {
        warehouse_list.read().iter().map(|w| SelectOption {
            value: w.id.to_string(),
            label: w.warehouse_name.clone(),
        }).collect::<Vec<_>>()
    });

    // ── Form State ──
    let count_date = use_signal(|| Some(chrono::Local::now().date_naive()));
    let warehouse = use_signal(|| String::new());
    let notes = use_signal(String::new);

    // UI state
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    // ── Validation ──
    let validate = {
        let wh = warehouse.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&str, String>::new();
            if wh.read().is_empty() {
                errs.insert("warehouse", "Warehouse is required.".to_string());
            }
            let is_valid = errs.is_empty();
            if !is_valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            is_valid
        }
    };

    // ── Handlers ──

    let on_warehouse_change = {
        let mut wh = warehouse.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { wh.set(v); dirty.set(true); }
    };

    let on_date_change = {
        let mut cd = count_date.clone();
        let mut dirty = is_dirty.clone();
        move |(start, _end): (Option<NaiveDate>, Option<NaiveDate>)| {
            cd.set(start);
            dirty.set(true);
        }
    };

    let on_notes_change = {
        let mut n = notes.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { n.set(v); dirty.set(true); }
    };

    // Save
    let save_count = {
        let api = api.clone();
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let wh = warehouse.clone();
        let nts = notes.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let wh = wh.read().parse::<i64>().unwrap_or(0);
            let nts = nts.read().clone();

            spawn(async move {
                let form = crate::models::PhysicalCountForm {
                    warehouse_id: wh,
                    notes: if nts.is_empty() { None } else { Some(nts) },
                };
                let client = api.read().clone();
                match client.create_physical_count(&form).await {
                    Ok(count) => {
                        toast.success("Count Created", &format!("Physical count {} has been created.", count.count_no));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/inventory/physical-counts");
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Save & New
    let save_and_new = {
        let api = api.clone();
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut wh = warehouse.clone();
        let mut nts = notes.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let api = api.clone();
            let mut toast = toast.clone();
            let wh_val = wh.read().parse::<i64>().unwrap_or(0);
            let nts_val = nts.read().clone();

            spawn(async move {
                let form = crate::models::PhysicalCountForm {
                    warehouse_id: wh_val,
                    notes: if nts_val.is_empty() { None } else { Some(nts_val) },
                };
                let client = api.read().clone();
                match client.create_physical_count(&form).await {
                    Ok(count) => {
                        toast.success("Count Created", &format!("{}. Creating another…", count.count_no));
                        // Reset form
                        wh.set(String::new());
                        nts.set(String::new());
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &e);
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
            else { nav.push("/inventory/physical-counts"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/inventory/physical-counts"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    // ── Derived ──
    let wh_err = errors.read().get("warehouse").cloned();

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page pc-create-page",

            // ── Header ──
            div { class: "pc-create-header",
                div {
                    a {
                        class: "pc-back-link",
                        href: "/inventory/physical-counts",
                        "← Back to Physical Counts"
                    }
                    h1 { "New Physical Count" }
                }
                if *is_dirty.read() {
                    span {
                        style: "font-size: 12px; color: var(--warning); font-weight: 500;",
                        "⚠ Unsaved changes"
                    }
                }
            }

            // ── Section: Count Details ──
            div { class: "pc-section",
                h2 { "Count Details" }
                div { class: "pc-form-row",
                    FormInput {
                        label: Some("Count No".to_string()),
                        value: "—".to_string(),
                        oninput: move |_| {},
                        r#type: InputType::Text,
                        disabled: true,
                        hint: Some("Assigned on save".to_string()),
                    }
                    DateRangePicker {
                        start: *count_date.read(),
                        end: None,
                        on_change: on_date_change,
                        start_label: "Count Date".to_string(),
                        end_label: "".to_string(),
                    }
                }
                div { class: "pc-form-row", style: "margin-top: 12px;",
                    div {
                        SearchableSelect {
                            options: warehouse_options(),
                            selected_value: Some(warehouse.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_warehouse_change,
                            placeholder: "Select warehouse…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                        if let Some(e) = &wh_err {
                            span { style: "color: var(--danger); font-size: 12px;", "{e}" }
                        }
                    }
                }
            }

            // ── Section: Notes ──
            div { class: "pc-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes_change,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes about this physical count…".to_string()),
                    hint: Some("Describe the scope or reason for this count.".to_string()),
                }
            }

            // ── Action Bar ──
            div { class: "pc-action-bar",
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
                    onclick: save_count,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Count"
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
                    "You have unsaved changes. Are you sure you want to discard this physical count?"
                }
            }
        }
    }
}
