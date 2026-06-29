//! BOM Create Page — Form to create a new Bill of Materials with component lines.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

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

static NEXT_BOM_ID: AtomicU32 = AtomicU32::new(1);

fn generate_bom_code() -> String {
    let seq = NEXT_BOM_ID.fetch_add(1, Ordering::Relaxed);
    format!("BOM-{:04}", seq)
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

fn raw_material_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "ITM-0020".to_string(), label: "Steel Plate 6mm".to_string() },
        SelectOption { value: "ITM-0021".to_string(), label: "Aluminum Rod 20mm".to_string() },
        SelectOption { value: "ITM-0022".to_string(), label: "Copper Wire 2.5mm".to_string() },
        SelectOption { value: "ITM-0023".to_string(), label: "Rubber Sheet 5mm".to_string() },
        SelectOption { value: "ITM-0024".to_string(), label: "Brass Fitting Set".to_string() },
        SelectOption { value: "ITM-0025".to_string(), label: "Bolt M8 x 30mm".to_string() },
        SelectOption { value: "ITM-0026".to_string(), label: "Nut M8".to_string() },
        SelectOption { value: "ITM-0027".to_string(), label: "Washer M8".to_string() },
        SelectOption { value: "ITM-0028".to_string(), label: "Hydraulic Seal Kit".to_string() },
        SelectOption { value: "ITM-0029".to_string(), label: "Control PCB v3".to_string() },
    ]
}

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

    let bom_code = use_signal(|| generate_bom_code());
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
        let mut bc = bom_code.clone();
        let mut fi = finished_item.clone();
        let mut qty = quantity_produced.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let code = bc.read().clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("BOM Created", &format!("Bill of Material {} has been created.", code));
                saving.set(false);
                dirty.set(false);
                nav.push("/manufacturing/boms");
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut bc = bom_code.clone();
        let mut fi = finished_item.clone();
        let mut qty = quantity_produced.clone();
        let mut lines = component_lines.clone();
        let mut nid = next_line_id.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let code = bc.read().clone();
            let mut toast = toast.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("BOM Created", &format!("{} created. Creating another...", code));
                bc.set(generate_bom_code());
                fi.set(String::new());
                qty.set("1.0".to_string());
                lines.set(vec![BomComponentLine { id: 1, item_code: String::new(), item_label: String::new(), quantity: 1.0, uom: "pcs".to_string(), unit_cost: 0.0, scrap_pct: 0.0 }]);
                nid.set(2);
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
                        hint: Some("Auto-generated".to_string()),
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
                        options: finished_item_options(),
                        selected_value: Some(finished_item.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_item_change,
                        placeholder: "Select finished item...",
                        searchable: true,
                        class: Some("cb-input-group".to_string()),
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
                            let mut rl = remove_line.clone();
                            let mut lines_signal = component_lines.clone();
                            let raw_opts = raw_material_options();

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
                                                move |v: String| {
                                                    let label = raw_material_options().iter()
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
