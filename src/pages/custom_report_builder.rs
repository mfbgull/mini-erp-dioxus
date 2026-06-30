//! Custom Report Builder Page — Create custom reports with field selection, filters, and grouping.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast,
};
use crate::models::CustomReportForm;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.crb-page { max-width: 800px; margin: 0 auto; }
.crb-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.crb-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.crb-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.crb-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }

.crb-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.crb-form-row > * { flex: 1; min-width: 180px; }
@media (max-width: 768px) { .crb-form-row { flex-direction: column; } .crb-form-row > * { min-width: 100%; } }

.crb-field-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }
.crb-field-group { margin-bottom: 12px; }
.crb-field-group-label { font-size: 12px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; margin-bottom: 8px; display: block; }

.crb-field-item { display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; cursor: pointer; font-size: 13px; color: var(--text-primary); user-select: none; transition: all 0.1s ease; }
.crb-field-item:hover { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.04); }
.crb-field-item input[type="checkbox"] { accent-color: var(--accent, #4a90d9); }
.crb-field-item label { cursor: pointer; flex: 1; }
.crb-field-item-selected { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.06); }

.crb-mock-table { width: 100%; border-collapse: collapse; font-size: 12px; margin-top: 12px; }
.crb-mock-table thead th { text-align: left; padding: 6px 8px; font-weight: 600; font-size: 10px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.crb-mock-table thead th.text-right { text-align: right; }
.crb-mock-table tbody td { padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); font-family: monospace; font-size: 11px; }
.crb-mock-table tbody td.text-right { text-align: right; }

.crb-preview-box { max-height: 300px; overflow-y: auto; margin-top: 12px; }

.crb-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
.crb-empty-preview { text-align: center; padding: 40px; color: var(--text-secondary); font-size: 14px; }
"##;

// ============================================================================
// Types
// ============================================================================

fn module_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "sales".to_string(), label: "Sales".to_string() },
        SelectOption { value: "purchasing".to_string(), label: "Purchasing".to_string() },
        SelectOption { value: "inventory".to_string(), label: "Inventory".to_string() },
        SelectOption { value: "accounting".to_string(), label: "Accounting".to_string() },
    ]
}

fn group_by_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "none".to_string(), label: "None".to_string() },
        SelectOption { value: "month".to_string(), label: "Month".to_string() },
        SelectOption { value: "quarter".to_string(), label: "Quarter".to_string() },
        SelectOption { value: "category".to_string(), label: "Category".to_string() },
        SelectOption { value: "customer".to_string(), label: "Customer".to_string() },
        SelectOption { value: "warehouse".to_string(), label: "Warehouse".to_string() },
    ]
}

fn sort_by_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "date".to_string(), label: "Date".to_string() },
        SelectOption { value: "amount".to_string(), label: "Amount".to_string() },
        SelectOption { value: "name".to_string(), label: "Name".to_string() },
    ]
}

fn sales_fields() -> Vec<(&'static str, &'static str)> {
    vec![
        ("invoice_no", "Invoice #"), ("customer", "Customer"), ("date", "Date"),
        ("amount", "Amount"), ("tax", "Tax"), ("total", "Total"),
        ("status", "Status"), ("category", "Category"),
    ]
}

fn preview_data() -> Vec<Vec<String>> {
    vec![
        vec!["INV-2026-0045".to_string(), "Alpha Traders".to_string(), "2026-06-22".to_string(), "156,000".to_string(), "23,400".to_string(), "179,400".to_string(), "Unpaid".to_string(), "Widgets".to_string()],
        vec!["INV-2026-0044".to_string(), "Delta Corp".to_string(), "2026-06-21".to_string(), "98,765".to_string(), "14,815".to_string(), "113,580".to_string(), "Paid".to_string(), "Fasteners".to_string()],
        vec!["INV-2026-0043".to_string(), "Gamma Supplies".to_string(), "2026-06-20".to_string(), "234,500".to_string(), "35,175".to_string(), "269,675".to_string(), "Partially Paid".to_string(), "Electrical".to_string()],
        vec!["INV-2026-0042".to_string(), "Epsilon LLC".to_string(), "2026-06-19".to_string(), "67,500".to_string(), "10,125".to_string(), "77,625".to_string(), "Overdue".to_string(), "Consumables".to_string()],
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn CustomReportBuilderPage() -> Element {
    let toast = use_toast();
    let mut report_name = use_signal(String::new);
    let mut module = use_signal(|| "sales".to_string());
    let mut from_date = use_signal(|| "2026-01-01".to_string());
    let mut to_date = use_signal(|| "2026-06-27".to_string());
    let mut group_by = use_signal(|| "none".to_string());
    let mut sort_by = use_signal(|| "date".to_string());
    let mut sort_order = use_signal(|| "desc".to_string());
    let selected_fields = use_signal(|| {
        let mut s = std::collections::HashSet::new();
        s.insert("invoice_no".to_string());
        s.insert("customer".to_string());
        s.insert("date".to_string());
        s.insert("amount".to_string());
        s.insert("status".to_string());
        s
    });
    let show_preview = use_signal(|| false);
    let is_saving = use_signal(|| false);

    let fields = sales_fields();

    let mut toggle_field = {
        let mut sel = selected_fields.clone();
        move |field: String| {
            let mut s = sel.write();
            if s.contains(&field) {
                s.remove(&field);
            } else {
                s.insert(field);
            }
        }
    };

    let preview = preview_data();

    let on_preview = {
        let mut sp = show_preview.clone();
        let mut t = toast.clone();
        move |_| {
            if selected_fields.read().is_empty() {
                t.warning("No Fields", "Select at least one field to preview.");
                return;
            }
            sp.set(true);
            t.info("Preview", "Generating report preview…");
        }
    };

    let api = use_auth().api;
    let t_gen = toast.clone();
    let on_generate = {
        let mut saving = is_saving.clone();
        let api = api.clone();
        move |_| {
            let mut t = t_gen.clone();
            let api = api.clone();
            if report_name.read().trim().is_empty() {
                t.warning("Name Required", "Enter a report name.");
                return;
            }
            saving.set(true);
            let name = report_name.read().clone();
            let module = module.read().clone();
            let config = serde_json::json!({
                "module": module,
                "fields": selected_fields.read().iter().cloned().collect::<Vec<_>>(),
                "from_date": from_date.read().clone(),
                "to_date": to_date.read().clone(),
                "group_by": group_by.read().clone(),
                "sort_by": sort_by.read().clone(),
                "sort_order": sort_order.read().clone(),
            });
            let mut t2 = t.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                let form = CustomReportForm {
                    name,
                    config: config.to_string(),
                };
                match client.create_custom_report(&form).await {
                    Ok(_) => {
                        saving.set(false);
                        t2.success("Report Generated", "Report has been generated.");
                    }
                    Err(e) => {
                        saving.set(false);
                        t2.error("Generation Failed", &e);
                    }
                }
            });
        }
    };

    let t_save = toast.clone();
    let on_save = {
        let mut saving = is_saving.clone();
        let api = api.clone();
        move |_| {
            let mut t = t_save.clone();
            let api = api.clone();
            if report_name.read().trim().is_empty() {
                t.warning("Name Required", "Enter a report name.");
                return;
            }
            saving.set(true);
            let name = report_name.read().clone();
            let module = module.read().clone();
            let config = serde_json::json!({
                "module": module,
                "fields": selected_fields.read().iter().cloned().collect::<Vec<_>>(),
                "from_date": from_date.read().clone(),
                "to_date": to_date.read().clone(),
                "group_by": group_by.read().clone(),
                "sort_by": sort_by.read().clone(),
                "sort_order": sort_order.read().clone(),
            });
            let mut t2 = t.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                let form = CustomReportForm {
                    name: name.clone(),
                    config: config.to_string(),
                };
                match client.create_custom_report(&form).await {
                    Ok(_) => {
                        saving.set(false);
                        t2.success("Report Saved", &format!("\"{}\" has been saved as a custom report.", name));
                    }
                    Err(e) => {
                        saving.set(false);
                        t2.error("Save Failed", &e);
                    }
                }
            });
        }
    };

    let fields_list: Vec<_> = fields.iter().filter(|(k, _)| selected_fields.read().contains(*k)).collect();
    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page crb-page",

            div { class: "crb-header",
                h1 { "Custom Report Builder" }
            }

            // Section: Report Info
            div { class: "crb-section",
                h2 { "Report Information" }
                div { class: "crb-form-row",
                    FormInput {
                        label: Some("Report Name".to_string()),
                        value: report_name.read().clone(),
                        oninput: move |v: String| { report_name.set(v); },
                        r#type: InputType::Text,
                        placeholder: Some("Enter report name".to_string()),
                        required: true,
                    }
                    div {
                        SearchableSelect {
                            options: module_options(),
                            selected_value: Some(module.read().clone()),
                            on_select: move |v: String| { module.set(v); },
                            placeholder: "Select module…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            // Section: Field Selection
            div { class: "crb-section",
                h2 { "Fields to Include" }
                div { class: "crb-field-grid",
                    {fields.iter().map(|(key, label)| {
                        let is_selected = selected_fields.read().contains(*key);
                        let cls = if is_selected { "crb-field-item crb-field-item-selected" } else { "crb-field-item" };
                        let k = key.to_string();
                        rsx! {
                            div {
                                key: "{key}",
                                class: "{cls}",
                                onclick: move |_| { toggle_field(k.clone()); },
                                input { r#type: "checkbox", checked: is_selected, oninput: move |_| {} }
                                label { "{label}" }
                            }
                        }
                    })}
                }
            }

            // Section: Filters & Options
            div { class: "crb-section",
                h2 { "Filters & Options" }
                div { class: "crb-form-row",
                    FormInput {
                        label: Some("From Date".to_string()),
                        value: from_date.read().clone(),
                        oninput: move |v: String| { from_date.set(v); },
                        r#type: InputType::Date,
                    }
                    FormInput {
                        label: Some("To Date".to_string()),
                        value: to_date.read().clone(),
                        oninput: move |v: String| { to_date.set(v); },
                        r#type: InputType::Date,
                    }
                }
                div { class: "crb-form-row", style: "margin-top: 12px;",
                    div {
                        SearchableSelect {
                            options: group_by_options(),
                            selected_value: Some(group_by.read().clone()),
                            on_select: move |v: String| { group_by.set(v); },
                            placeholder: "Group by…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    div {
                        SearchableSelect {
                            options: sort_by_options(),
                            selected_value: Some(sort_by.read().clone()),
                            on_select: move |v: String| { sort_by.set(v); },
                            placeholder: "Sort by…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    div { style: "display: flex; align-items: flex-end; gap: 8px;",
                        Button {
                            variant: if *sort_order.read() == "asc" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                            size: ButtonSize::Sm,
                            onclick: move |_| { sort_order.set("asc".to_string()); },
                            "Asc"
                        }
                        Button {
                            variant: if *sort_order.read() == "desc" { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                            size: ButtonSize::Sm,
                            onclick: move |_| { sort_order.set("desc".to_string()); },
                            "Desc"
                        }
                    }
                }
            }

            // Preview section
            div { class: "crb-section",
                div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                    h2 { style: "margin: 0; border: none; padding: 0;", "Preview" }
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        onclick: on_preview,
                        icon: Some("👁".to_string()),
                        "Refresh Preview"
                    }
                }

                if *show_preview.read() {
                    if fields_list.is_empty() {
                        div { class: "crb-empty-preview", "Select fields above to see preview data." }
                    } else {
                        div { class: "crb-preview-box",
                            table { class: "crb-mock-table",
                                thead {
                                    tr {
                                        {fields_list.iter().map(|(_, label)| rsx! {
                                            th { key: "{label}", "{label}" }
                                        })}
                                    }
                                }
                                tbody {
                                    {preview.iter().map(|row| rsx! {
                                        tr {
                                            {fields_list.iter().map(|(key, _)| {
                                                let idx = fields.iter().position(|(k, _)| k == key).unwrap_or(0);
                                                rsx! {
                                                    td { key: "{key}", "{row[idx]}" }
                                                }
                                            })}
                                        }
                                    })}
                                }
                            }
                        }
                    }
                } else {
                    div { class: "crb-empty-preview", "Click \"Refresh Preview\" to see sample data." }
                }
            }

            // Action bar
            div { class: "crb-action-bar",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: {
                        let mut sp = show_preview.clone();
                        let mut t = toast.clone();
                        move |_| {
                            if selected_fields.read().is_empty() {
                                t.warning("No Fields", "Select at least one field to preview.");
                                return;
                            }
                            sp.set(true);
                            t.info("Preview", "Generating report preview…");
                        }
                    },
                    icon: Some("👁".to_string()),
                    "Preview"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: on_generate,
                    loading: *is_saving.read(),
                    icon: Some("⚡".to_string()),
                    "Generate Report"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_save,
                    loading: *is_saving.read(),
                    icon: Some("💾".to_string()),
                    "Save Custom Report"
                }
            }
        }
    }
}
