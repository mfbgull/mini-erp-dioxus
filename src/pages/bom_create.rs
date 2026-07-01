//! BOM Create Page — Form to create a new Bill of Materials with component lines.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::models::{BomForm, BomItemForm, Item};
use dioxus::prelude::*;
use std::collections::HashMap;

const PAGE_CSS: &str = r##"
.bom-create-page {
    max-width: 860px;
    margin: 0 auto;
}
.bom-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}
.bom-create-header h1 {
    font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary);
}
.bom-back-link {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px;
}
.bom-back-link:hover { text-decoration: underline; }
.bom-section {
    background: #fff; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px;
}
.bom-section h2 {
    font-size: 15px; font-weight: 600; color: var(--text-primary);
    margin: 0 0 16px 0; padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}
.bom-form-row {
    display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap;
}
.bom-form-row > * { flex: 1; min-width: 180px; }
.bom-components-section { margin-top: 4px; }
.bom-comp-header {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 10px;
}
.bom-comp-header span { font-size: 13px; color: var(--text-secondary); font-weight: 500; }
.bom-comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.bom-comp-table thead th {
    text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px;
    text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary);
    border-bottom: 2px solid var(--border-color, #e0e0e0);
}
.bom-comp-table thead th.text-right { text-align: right; }
.bom-comp-table tbody td {
    padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary); vertical-align: middle;
}
.bom-comp-table tbody td.text-right { text-align: right; }
.bom-comp-table tbody input, .bom-comp-table tbody select {
    width: 100%; padding: 5px 8px; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px; font-size: 12px; box-sizing: border-box;
}
.bom-comp-remove {
    padding: 4px 8px; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px; background: #fff; cursor: pointer;
    font-size: 13px; color: var(--text-secondary);
}
.bom-comp-remove:hover { border-color: #dc3545; color: #dc3545; }
.bom-total-row {
    display: flex; justify-content: flex-end; align-items: center;
    gap: 24px; padding: 12px 10px 0; margin-top: 8px;
    border-top: 2px solid var(--border-color, #e0e0e0);
}
.bom-total-label { font-size: 13px; font-weight: 600; color: var(--text-secondary); }
.bom-total-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
.bom-action-bar {
    display: flex; justify-content: flex-end; gap: 8px;
    margin-top: 20px; padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}
@media (max-width: 768px) {
    .bom-form-row { flex-direction: column; }
    .bom-form-row > * { min-width: 100%; }
    .bom-action-bar { flex-direction: column; }
}
"##;

fn uom_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "pcs".to_string(), label: "Pieces (pcs)".to_string() },
        SelectOption { value: "kg".to_string(), label: "Kilograms (kg)".to_string() },
        SelectOption { value: "liters".to_string(), label: "Liters".to_string() },
        SelectOption { value: "meters".to_string(), label: "Meters (m)".to_string() },
        SelectOption { value: "sheets".to_string(), label: "Sheets".to_string() },
    ]
}

#[derive(Clone, PartialEq, Debug)]
struct BomComponentLine {
    id: usize,
    item_code: String,
    item_label: String,
    quantity: f64,
    uom: String,
    unit_cost: f64,
    scrap_pct: f64,
}

#[component]
pub fn BomCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let api = use_auth().api;
    let item_map = use_signal(HashMap::<i64, Item>::new);
    let item_options_signal = use_signal(Vec::<SelectOption>::new);
    let item_map_by_code = use_signal(HashMap::<String, i64>::new);

    {
        let api = api.clone();
        let mut item_map = item_map.clone();
        let mut item_opts = item_options_signal.clone();
        let mut code_map = item_map_by_code.clone();
        use_effect(move || {
            let client = api.read().clone();
            let mut item_map = item_map.clone();
            let mut item_opts = item_opts.clone();
            let mut code_map = code_map.clone();
            spawn(async move {
                if let Ok(items) = client.list_items_catalog().await {
                    let mut map = HashMap::new();
                    let mut opts = Vec::new();
                    let mut c_map = HashMap::new();
                    for item in &items {
                        map.insert(item.id, item.clone());
                        opts.push(SelectOption {
                            value: item.id.to_string(),
                            label: format!("{} ({})", item.item_name, item.item_code),
                        });
                        c_map.insert(item.item_code.clone(), item.id);
                    }
                    item_map.set(map);
                    item_opts.set(opts);
                    code_map.set(c_map);
                }
            });
        });
    }

    let bom_code = use_signal(String::new);
    let description = use_signal(String::new);
    let finished_item = use_signal(String::new);
    let quantity_produced = use_signal(|| "1.0".to_string());
    let component_lines = use_signal(|| vec![
        BomComponentLine { id: 1, item_code: String::new(), item_label: String::new(), quantity: 1.0, uom: "pcs".to_string(), unit_cost: 0.0, scrap_pct: 0.0 },
    ]);
    let next_line_id = use_signal(|| 2usize);

    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    let total_cost = component_lines.read().iter()
        .map(|l| {
            let base = l.quantity * l.unit_cost;
            base * (1.0 + l.scrap_pct / 100.0)
        })
        .sum::<f64>();

    let validate = {
        let mut fi = finished_item.clone();
        let mut cls = component_lines.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if fi.read().is_empty() { errs.insert("item", "Finished item is required.".to_string()); }
            let has_all = cls.read().iter().all(|l| !l.item_code.is_empty() && l.quantity > 0.0);
            if !has_all { errs.insert("components", "All component lines must have an item and quantity > 0.".to_string()); }
            let is_valid = errs.is_empty();
            if !is_valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            is_valid
        }
    };

    let on_item_change = {
        let mut fi = finished_item.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { fi.set(v); dirty.set(true); }
    };

    let on_qty_change = {
        let mut q = quantity_produced.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { q.set(v); dirty.set(true); }
    };

    let add_line = {
        let mut lines = component_lines.clone();
        let mut nid = next_line_id.clone();
        move |_| {
            let id = *nid.read();
            lines.write().push(BomComponentLine { id, item_code: String::new(), item_label: String::new(), quantity: 1.0, uom: "pcs".to_string(), unit_cost: 0.0, scrap_pct: 0.0 });
            nid.set(id + 1);
        }
    };

    let remove_line = {
        let mut lines = component_lines.clone();
        move |id: usize| {
            if lines.read().len() > 1 {
                lines.write().retain(|l| l.id != id);
            }
        }
    };

    let save_bom = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let bc = bom_code.clone();
        let desc = description.clone();
        let fi = finished_item.clone();
        let qty = quantity_produced.clone();
        let cls = component_lines.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let bom_name = bc.read().clone();
            let description = Some(desc.read().clone()).filter(|s| !s.is_empty());
            let finished_item_id = fi.read().parse::<i64>().unwrap_or(0);
            let quantity = qty.read().parse::<f64>().unwrap_or(0.0);
            let items: Vec<BomItemForm> = cls.read().iter().map(|l| {
                BomItemForm {
                    item_id: l.item_code.parse::<i64>().unwrap_or(0),
                    quantity: l.quantity,
                    unit_cost: if l.unit_cost == 0.0 { None } else { Some(l.unit_cost) },
                }
            }).collect();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let api = api.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = BomForm {
                    bom_name: if bom_name.is_empty() { "New BOM".to_string() } else { bom_name },
                    finished_item_id,
                    quantity,
                    description,
                    items,
                };
                match client.create_bom(&form).await {
                    Ok(_) => {
                        toast.success("BOM Created", "Bill of Material has been created.");
                        dirty.set(false);
                        nav.push("/manufacturing/boms");
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let bc = bom_code.clone();
        let desc = description.clone();
        let fi = finished_item.clone();
        let qty = quantity_produced.clone();
        let mut lines = component_lines.clone();
        let mut nid = next_line_id.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let bom_name = bc.read().clone();
            let description = Some(desc.read().clone()).filter(|s| !s.is_empty());
            let finished_item_id = fi.read().parse::<i64>().unwrap_or(0);
            let quantity = qty.read().parse::<f64>().unwrap_or(0.0);
            let items: Vec<BomItemForm> = lines.read().iter().map(|l| {
                BomItemForm {
                    item_id: l.item_code.parse::<i64>().unwrap_or(0),
                    quantity: l.quantity,
                    unit_cost: if l.unit_cost == 0.0 { None } else { Some(l.unit_cost) },
                }
            }).collect();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let mut lines = lines.clone();
            let mut nid = nid.clone();
            let mut fi = fi.clone();
            let mut qty = qty.clone();
            let mut bc = bc.clone();
            let mut desc = desc.clone();
            let api = api.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = BomForm {
                    bom_name: if bom_name.is_empty() { "New BOM".to_string() } else { bom_name },
                    finished_item_id,
                    quantity,
                    description,
                    items,
                };
                match client.create_bom(&form).await {
                    Ok(_) => {
                        toast.success("BOM Created", "Created. Creating another...");
                        bc.set(String::new());
                        desc.set(String::new());
                        fi.set(String::new());
                        qty.set("1.0".to_string());
                        lines.set(vec![BomComponentLine { id: 1, item_code: String::new(), item_label: String::new(), quantity: 1.0, uom: "pcs".to_string(), unit_cost: 0.0, scrap_pct: 0.0 }]);
                        nid.set(2);
                        dirty.set(false);
                        saving.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let mut dirty = is_dirty.clone();
        let mut nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/manufacturing/boms"); }
        }
    };

    let confirm_discard = {
        let mut nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/manufacturing/boms"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page bom-create-page",
            div { class: "bom-create-header",
                div {
                    a { class: "bom-back-link", href: "/manufacturing/boms", "← Back to BOM List" }
                    h1 { "New Bill of Materials" }
                }
                if *is_dirty.read() {
                    span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" }
                }
            }

            div { class: "bom-section",
                h2 { "Header Information" }
                div { class: "bom-form-row",
                    FormInput {
                        label: Some("BOM Code".to_string()),
                        value: bom_code.read().clone(),
                        oninput: move |_| {},
                        r#type: InputType::Text,
                        disabled: true,
                        hint: Some("Auto-generated by server".to_string()),
                    }
                    FormInput {
                        label: Some("Quantity Produced".to_string()),
                        value: quantity_produced.read().clone(),
                        oninput: on_qty_change,
                        r#type: InputType::Number,
                        placeholder: Some("1.0".to_string()),
                        min: Some(0.0),
                        step: Some(0.01),
                        hint: Some("Number of items this BOM produces".to_string()),
                    }
                }
                div { class: "bom-form-row", style: "margin-top: 12px;",
                    SearchableSelect {
                        options: item_options_signal.read().clone(),
                        selected_value: Some(finished_item.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_item_change,
                        placeholder: "Select finished item...",
                        searchable: true,
                        class: Some("cb-input-group".to_string()),
                    }
                }
                div { style: "margin-top: 12px;",
                    label { style: "display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px;", "Description" }
                    textarea {
                        style: "width: 100%; min-height: 60px; padding: 8px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; font-size: 13px; box-sizing: border-box; resize: vertical;",
                        placeholder: "Optional description or notes about this BOM...",
                        value: description.read().clone(),
                        oninput: {
                            let mut d = description.clone();
                            let mut dirty = is_dirty.clone();
                            move |e| { d.set(e.value()); dirty.set(true); }
                        },
                    }
                }
            }

            div { class: "bom-section bom-components-section",
                div { class: "bom-comp-header",
                    h2 { style: "margin: 0; border: none; padding: 0;", "Components" }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        onclick: add_line,
                        icon: Some("＋".to_string()),
                        "Add Component"
                    }
                }
                table { class: "bom-comp-table",
                    thead {
                        tr {
                            th { style: "width: 30%;", "Item" }
                            th { style: "width: 12%;", "Quantity" }
                            th { style: "width: 10%;", "UOM" }
                            th { style: "width: 16%;", "Unit Cost (PKR)" }
                            th { style: "width: 10%;", "Scrap %" }
                            th { style: "width: 14%;", "Sub-total" }
                            th { style: "width: 8%;", "" }
                        }
                    }
                    tbody {
                        {component_lines.read().iter().map(|line| {
                            let line_id = line.id;
                            let lid = line.id;
                            let rl = remove_line.clone();
                            let lines_signal = component_lines.clone();
                            let raw_opts = item_options_signal.read().clone();

                            let sub_total = {
                                let base = line.quantity * line.unit_cost;
                                base * (1.0 + line.scrap_pct / 100.0)
                            };

                            rsx! {
                                tr { key: "{lid}",
                                    td {
                                        SearchableSelect {
                                            options: raw_opts,
                                            selected_value: if line.item_code.is_empty() { None } else { Some(line.item_code.clone()) },
                                            on_select: {
                                                let mut ls = lines_signal.clone();
                                                let item_opts = item_options_signal.clone();
                                                move |v: String| {
                                                    let label = item_opts.read().iter()
                                                        .find(|o| o.value == v)
                                                        .map(|o| o.label.clone())
                                                        .unwrap_or_default();
                                                    ls.write().iter_mut().find(|l| l.id == lid).map(|l| {
                                                        l.item_code = v;
                                                        l.item_label = label;
                                                    });
                                                }
                                            },
                                            placeholder: "Select item...",
                                            searchable: true,
                                            class: Some("cb-input-group".to_string()),
                                        }
                                    }
                                    td {
                                        input {
                                            r#type: "number",
                                            min: "0.01",
                                            step: "0.01",
                                            value: "{line.quantity}",
                                            oninput: {
                                                let mut ls = lines_signal.clone();
                                                move |e| {
                                                    if let Ok(v) = e.value().parse::<f64>() {
                                                        ls.write().iter_mut().find(|l| l.id == lid).map(|l| l.quantity = v);
                                                    }
                                                }
                                            },
                                        }
                                    }
                                    td { "{line.uom}" }
                                    td {
                                        input {
                                            r#type: "number",
                                            min: "0",
                                            step: "0.01",
                                            value: if line.unit_cost == 0.0 { String::new() } else { line.unit_cost.to_string() },
                                            oninput: {
                                                let mut ls = lines_signal.clone();
                                                move |e| {
                                                    if let Ok(v) = e.value().parse::<f64>() {
                                                        ls.write().iter_mut().find(|l| l.id == lid).map(|l| l.unit_cost = v);
                                                    } else if e.value().is_empty() {
                                                        ls.write().iter_mut().find(|l| l.id == lid).map(|l| l.unit_cost = 0.0);
                                                    }
                                                }
                                            },
                                            placeholder: "0.00",
                                        }
                                    }
                                    td {
                                        input {
                                            r#type: "number",
                                            min: "0",
                                            max: "100",
                                            step: "0.1",
                                            value: if line.scrap_pct == 0.0 { String::new() } else { line.scrap_pct.to_string() },
                                            oninput: {
                                                let mut ls = lines_signal.clone();
                                                move |e| {
                                                    if let Ok(v) = e.value().parse::<f64>() {
                                                        ls.write().iter_mut().find(|l| l.id == lid).map(|l| l.scrap_pct = v);
                                                    } else if e.value().is_empty() {
                                                        ls.write().iter_mut().find(|l| l.id == lid).map(|l| l.scrap_pct = 0.0);
                                                    }
                                                }
                                            },
                                            placeholder: "0",
                                        }
                                    }
                                    td { class: "text-right",
                                        "PKR {sub_total:.2}"
                                    }
                                    td {
                                        button {
                                            class: "bom-comp-remove",
                                            r#type: "button",
                                            onclick: { let mut rl = rl.clone(); move |_| rl(line_id) },
                                            disabled: component_lines.read().len() <= 1,
                                            "×"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }

                div { class: "bom-total-row",
                    span { class: "bom-total-label", "Total Cost (with scrap):" }
                    span { class: "bom-total-value", "PKR {total_cost:.2}" }
                }
            }

            div { class: "bom-action-bar",
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
                    onclick: save_bom,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save BOM"
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
                    "You have unsaved changes. Are you sure you want to discard this BOM?"
                }
            }
        }
    }
}
