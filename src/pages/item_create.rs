//! Item Create Page — Full-featured inventory item creation form using
//! the common UI component library (FormInput, SearchableSelect, Button,
//! Toast, Modal, StatCard).

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.item-create-page {
    max-width: 800px;
    margin: 0 auto;
}

.item-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.item-create-header h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
}

.item-back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
    margin-bottom: 16px;
}

.item-back-link:hover { text-decoration: underline; }

.item-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.item-section h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.item-form-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.item-form-row > * {
    flex: 1;
    min-width: 180px;
}

.item-type-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
}

.item-type-chip {
    display: flex;
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

.item-type-chip:hover {
    border-color: var(--accent, #4a90d9);
    background: rgba(74, 144, 217, 0.04);
}

.item-type-chip-active {
    border-color: var(--accent, #4a90d9);
    background: rgba(74, 144, 217, 0.08);
    color: var(--accent, #4a90d9);
    font-weight: 600;
}

.item-type-chip input {
    display: none;
}

.item-type-chip-icon {
    font-size: 16px;
}

.item-action-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}

@media (max-width: 768px) {
    .item-form-row { flex-direction: column; }
    .item-form-row > * { min-width: 100%; }
    .item-type-grid { grid-template-columns: 1fr 1fr; }
    .item-action-bar { flex-direction: column; }
}
"##;

// ============================================================================

fn category_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Widgets".to_string(), label: "Widgets".to_string() },
        SelectOption { value: "Fasteners".to_string(), label: "Fasteners".to_string() },
        SelectOption { value: "Raw Materials".to_string(), label: "Raw Materials".to_string() },
        SelectOption { value: "Equipment".to_string(), label: "Equipment".to_string() },
        SelectOption { value: "Consumables".to_string(), label: "Consumables".to_string() },
        SelectOption { value: "Electrical".to_string(), label: "Electrical".to_string() },
        SelectOption { value: "Packaging".to_string(), label: "Packaging".to_string() },
        SelectOption { value: "Safety".to_string(), label: "Safety".to_string() },
    ]
}

fn uom_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "pcs".to_string(), label: "Pieces (pcs)".to_string() },
        SelectOption { value: "kg".to_string(), label: "Kilograms (kg)".to_string() },
        SelectOption { value: "liters".to_string(), label: "Liters".to_string() },
        SelectOption { value: "rolls".to_string(), label: "Rolls".to_string() },
        SelectOption { value: "sheets".to_string(), label: "Sheets".to_string() },
        SelectOption { value: "packs".to_string(), label: "Packs".to_string() },
        SelectOption { value: "meters".to_string(), label: "Meters (m)".to_string() },
        SelectOption { value: "boxes".to_string(), label: "Boxes".to_string() },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ItemCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    // ── Form State ──
    let item_code = use_signal(String::new);
    let item_name = use_signal(String::new);
    let category = use_signal(String::new);
    let uom = use_signal(|| "pcs".to_string());
    let standard_cost = use_signal(|| String::new());
    let selling_price = use_signal(|| String::new());
    let reorder_level = use_signal(|| String::new());
    let warehouse = use_signal(|| "Main Warehouse".to_string());
    let notes = use_signal(String::new);
    let mut is_active = use_signal(|| true);

    // Item type toggles
    let is_raw_material = use_signal(|| false);
    let is_finished_good = use_signal(|| false);
    let is_purchased = use_signal(|| true);
    let is_manufactured = use_signal(|| false);

    // UI state
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    // ── Validation ──
    let validate = {
        let name = item_name.clone();
        let cat = category.clone();
        let cost = standard_cost.clone();
        let price = selling_price.clone();
        let reorder = reorder_level.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if name.read().trim().is_empty() {
                errs.insert("name", "Item name is required.".to_string());
            }
            if cat.read().is_empty() {
                errs.insert("category", "Category is required.".to_string());
            }
            if let Ok(c) = cost.read().parse::<f64>() {
                if c < 0.0 { errs.insert("cost", "Cost cannot be negative.".to_string()); }
            } else if !cost.read().is_empty() {
                errs.insert("cost", "Invalid number.".to_string());
            }
            if let Ok(p) = price.read().parse::<f64>() {
                if p < 0.0 { errs.insert("price", "Price cannot be negative.".to_string()); }
            } else if !price.read().is_empty() {
                errs.insert("price", "Invalid number.".to_string());
            }
            if let Ok(r) = reorder.read().parse::<i32>() {
                if r < 0 { errs.insert("reorder", "Cannot be negative.".to_string()); }
            } else if !reorder.read().is_empty() {
                errs.insert("reorder", "Invalid number.".to_string());
            }

            let is_valid = errs.is_empty();
            if !is_valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            is_valid
        }
    };

    // ── Type Toggle Helper ──
    let make_toggle = |signal: Signal<bool>, _key: &'static str| {
        let mut sig = signal.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            let val = !*sig.read();
            *sig.write() = val;
            dirty.set(true);
        }
    };

    // ── Handlers ──

    let on_name_change = {
        let mut name = item_name.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { name.set(v); dirty.set(true); }
    };

    let on_category_change = {
        let mut cat = category.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { cat.set(v); dirty.set(true); }
    };

    let on_uom_change = {
        let mut u = uom.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { u.set(v); dirty.set(true); }
    };

    #[allow(clippy::redundant_closure)]
    let on_cost_change = {
        let mut c = standard_cost.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { c.set(v); dirty.set(true); }
    };

    #[allow(clippy::redundant_closure)]
    let on_price_change = {
        let mut p = selling_price.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { p.set(v); dirty.set(true); }
    };

    #[allow(clippy::redundant_closure)]
    let on_reorder_change = {
        let mut r = reorder_level.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { r.set(v); dirty.set(true); }
    };

    let on_warehouse_change = {
        let mut w = warehouse.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { w.set(v); dirty.set(true); }
    };

    let on_code_change = {
        let mut code = item_code.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { code.set(v); dirty.set(true); }
    };

    let on_notes_change = {
        let mut n = notes.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { n.set(v); dirty.set(true); }
    };

    // Toggle handlers
    let toggle_raw = make_toggle(is_raw_material, "raw");
    let toggle_finished = make_toggle(is_finished_good, "finished");
    let toggle_purchased = make_toggle(is_purchased, "purchased");
    let toggle_manufactured = make_toggle(is_manufactured, "manufactured");

    // Save
    let save_item = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let name = item_name.clone();
        let code = item_code.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        let cat = category.clone();
        let u = uom.clone();
        let cost = standard_cost.clone();
        let price = selling_price.clone();
        let reorder = reorder_level.clone();
        let desc = notes.clone();
        let raw = is_raw_material.clone();
        let finished = is_finished_good.clone();
        let purchased = is_purchased.clone();
        let manufactured = is_manufactured.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);

            let form = crate::models::ItemForm {
                item_code: code.read().clone(),
                item_name: name.read().clone(),
                description: {
                    let d = desc.read();
                    if d.is_empty() { None } else { Some(d.clone()) }
                },
                category: {
                    let c = cat.read();
                    if c.is_empty() { None } else { Some(c.clone()) }
                },
                unit_of_measure: Some(u.read().clone()),
                reorder_level: reorder.read().parse::<f64>().ok(),
                standard_cost: cost.read().parse::<f64>().ok(),
                selling_price: price.read().parse::<f64>().ok(),
                is_raw_material: Some(*raw.read()),
                is_finished_good: Some(*finished.read()),
                is_purchased: Some(*purchased.read()),
                is_manufactured: Some(*manufactured.read()),
            };

            let n = name.read().clone();
            let c = code.read().clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();

            spawn(async move {
                let client = api.with(|a| a.clone());
                match client.create_item(&form).await {
                    Ok(_) => {
                        toast.success("Item Created", &format!("{} ({}) has been created.", n, c));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/inventory/items");
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
        let mut saving = is_saving.clone();
        let toast = toast.clone();
        let name = item_name.clone();
        let code = item_code.clone();
        let mut validate = validate.clone();
        let mut i_name = item_name.clone();
        let mut i_category = category.clone();
        let mut i_uom = uom.clone();
        let mut i_cost = standard_cost.clone();
        let mut i_price = selling_price.clone();
        let mut i_reorder = reorder_level.clone();
        let mut i_notes = notes.clone();
        let mut i_raw = is_raw_material.clone();
        let mut i_finished = is_finished_good.clone();
        let mut i_purchased = is_purchased.clone();
        let mut i_manufactured = is_manufactured.clone();
        let mut i_code = item_code.clone();
        let mut i_active = is_active.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        let cat = category.clone();
        let u = uom.clone();
        let cost = standard_cost.clone();
        let price = selling_price.clone();
        let reorder = reorder_level.clone();
        let desc = notes.clone();
        let raw = is_raw_material.clone();
        let finished = is_finished_good.clone();
        let purchased = is_purchased.clone();
        let manufactured = is_manufactured.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);

            let form = crate::models::ItemForm {
                item_code: code.read().clone(),
                item_name: name.read().clone(),
                description: {
                    let d = desc.read();
                    if d.is_empty() { None } else { Some(d.clone()) }
                },
                category: {
                    let c = cat.read();
                    if c.is_empty() { None } else { Some(c.clone()) }
                },
                unit_of_measure: Some(u.read().clone()),
                reorder_level: reorder.read().parse::<f64>().ok(),
                standard_cost: cost.read().parse::<f64>().ok(),
                selling_price: price.read().parse::<f64>().ok(),
                is_raw_material: Some(*raw.read()),
                is_finished_good: Some(*finished.read()),
                is_purchased: Some(*purchased.read()),
                is_manufactured: Some(*manufactured.read()),
            };

            let n = name.read().clone();
            let c = code.read().clone();
            let mut toast = toast.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();

            spawn(async move {
                let client = api.with(|a| a.clone());
                match client.create_item(&form).await {
                    Ok(_) => {
                        toast.success("Item Created", &format!("{} ({}) created. Creating another…", n, c));
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                        return;
                    }
                }

                // Reset form
                i_code.set(String::new());
                i_name.set(String::new());
                i_category.set(String::new());
                i_uom.set("pcs".to_string());
                i_cost.set(String::new());
                i_price.set(String::new());
                i_reorder.set(String::new());
                i_notes.set(String::new());
                i_active.set(true);
                i_raw.set(false);
                i_finished.set(false);
                i_purchased.set(true);
                i_manufactured.set(false);
                saving.set(false);
                dirty.set(false);
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
            else { nav.push("/inventory/items"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/inventory/items"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    // ── Computed values ──
    let cost_val = standard_cost.read().parse::<f64>().unwrap_or(0.0);
    let price_val = selling_price.read().parse::<f64>().unwrap_or(0.0);
    let margin = if price_val > 0.0 {
        ((price_val - cost_val) / price_val * 100.0).round()
    } else {
        0.0
    };
    let name_err = errors.read().get("name").cloned();
    let category_err = errors.read().get("category").cloned();
    let cost_err = errors.read().get("cost").cloned();
    let price_err = errors.read().get("price").cloned();
    let reorder_err = errors.read().get("reorder").cloned();

    let raw_chip_class = if *is_raw_material.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let finished_chip_class = if *is_finished_good.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let purchased_chip_class = if *is_purchased.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let manufactured_chip_class = if *is_manufactured.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let active_chip_class = if *is_active.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let active_icon = if *is_active.read() { "✅" } else { "⛔" };
    let active_label = if *is_active.read() { "Active" } else { "Inactive" };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page item-create-page",

            // ── Header ──
            div { class: "item-create-header",
                div {
                    a {
                        class: "item-back-link",
                        href: "/inventory/items",
                        "← Back to Items"
                    }
                    h1 { "New Inventory Item" }
                }
                if *is_dirty.read() {
                    span {
                        style: "font-size: 12px; color: var(--warning); font-weight: 500;",
                        "⚠ Unsaved changes"
                    }
                }
            }

            // ── Section: Basic Info ──
            div { class: "item-section",
                h2 { "Basic Information" }
                div { class: "item-form-row",
                    FormInput {
                        label: Some("Item Code".to_string()),
                        value: item_code.read().clone(),
                        oninput: on_code_change,
                        r#type: InputType::Text,
                        placeholder: Some("Leave empty for auto-generated".to_string()),
                        hint: Some("Auto-generated if empty".to_string()),
                    }
                    FormInput {
                        label: Some("Item Name".to_string()),
                        value: item_name.read().clone(),
                        oninput: on_name_change,
                        r#type: InputType::Text,
                        placeholder: Some("Enter item name".to_string()),
                        required: true,
                        error: name_err,
                    }
                }
                div { class: "item-form-row", style: "margin-top: 12px;",
                    div {
                        SearchableSelect {
                            options: category_options(),
                            selected_value: Some(category.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_category_change,
                            placeholder: "Select category…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    div {
                        SearchableSelect {
                            options: uom_options(),
                            selected_value: Some(uom.read().clone()),
                            on_select: on_uom_change,
                            placeholder: "Select unit…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            // ── Section: Pricing & Stock ──
            div { class: "item-section",
                h2 { "Pricing & Stock" }
                div { class: "item-form-row",
                    FormInput {
                        label: Some("Standard Cost (PKR)".to_string()),
                        value: standard_cost.read().clone(),
                        oninput: on_cost_change,
                        r#type: InputType::Number,
                        placeholder: Some("0.00".to_string()),
                        min: Some(0.0),
                        step: Some(0.01),
                        icon: Some("₹".to_string()),
                        error: cost_err,
                    }
                    FormInput {
                        label: Some("Selling Price (PKR)".to_string()),
                        value: selling_price.read().clone(),
                        oninput: on_price_change,
                        r#type: InputType::Number,
                        placeholder: Some("0.00".to_string()),
                        min: Some(0.0),
                        step: Some(0.01),
                        icon: Some("₹".to_string()),
                        error: price_err,
                    }
                    FormInput {
                        label: Some("Reorder Level".to_string()),
                        value: reorder_level.read().clone(),
                        oninput: on_reorder_change,
                        r#type: InputType::Number,
                        placeholder: Some("0".to_string()),
                        min: Some(0.0),
                        step: Some(1.0),
                        hint: Some("Alert when stock drops below".to_string()),
                        error: reorder_err,
                    }
                }

                // Margin preview card
                if price_val > 0.0 || cost_val > 0.0 {
                    div { style: "margin-top: 14px;",
                        StatCard {
                            title: if price_val > 0.0 && cost_val > 0.0 {
                                format!("Profit Margin (Cost: PKR {:.2})", cost_val)
                            } else if cost_val > 0.0 {
                                format!("Cost: PKR {:.2} (Set selling price to see margin)", cost_val)
                            } else {
                                "Enter cost & price to see margin".to_string()
                            },
                            value: if price_val > 0.0 && cost_val > 0.0 {
                                format!("PKR {:.2} ({:.0}%)", price_val - cost_val, margin)
                            } else {
                                "—".to_string()
                            },
                            variant: StatCardVariant::Primary,
                            icon: if margin >= 30.0 { Some("📈".to_string()) }
                                  else if margin >= 10.0 { Some("📊".to_string()) }
                                  else { Some("📉".to_string()) },
                        }
                    }
                }
            }

            // ── Section: Item Type ──
            div { class: "item-section",
                h2 { "Item Classification" }
                div { class: "item-type-grid",
                    // Raw Material
                    div {
                        class: "{raw_chip_class}",
                        onclick: toggle_raw,
                        span { class: "item-type-chip-icon", "🛢" }
                        span { "Raw Material" }
                    }
                    // Finished Good
                    div {
                        class: "{finished_chip_class}",
                        onclick: toggle_finished,
                        span { class: "item-type-chip-icon", "📦" }
                        span { "Finished Good" }
                    }
                    // Purchased
                    div {
                        class: "{purchased_chip_class}",
                        onclick: toggle_purchased,
                        span { class: "item-type-chip-icon", "🛒" }
                        span { "Purchased" }
                    }
                    // Manufactured
                    div {
                        class: "{manufactured_chip_class}",
                        onclick: toggle_manufactured,
                        span { class: "item-type-chip-icon", "⚙" }
                        span { "Manufactured" }
                    }
                }
            }

            // ── Section: Status ──
            div { class: "item-section",
                h2 { "Status" }
                div { class: "item-form-row",
                    div {
                        label { class: "cb-input-label", style: "margin-bottom: 6px; display: block;", "Status" }
                        button {
                            class: "{active_chip_class}",
                            onclick: move |_| {
                                let val = !*is_active.read();
                                is_active.set(val);
                                is_dirty.set(true);
                            },
                            style: "display: inline-flex; width: auto;",
                            span { class: "item-type-chip-icon", "{active_icon}" }
                            span { "{active_label}" }
                        }
                    }
                }
            }

            // ── Section: Notes ──
            div { class: "item-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes_change,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes or description…".to_string()),
                    hint: Some("Internal notes about this item.".to_string()),
                }
            }

            // ── Action Bar ──
            div { class: "item-action-bar",
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
                    onclick: save_item,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Item"
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
                    "You have unsaved changes. Are you sure you want to discard this item?"
                }
            }
        }
    }
}
