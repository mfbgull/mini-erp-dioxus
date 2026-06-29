//! Expense Create Page — Form to create a new expense record.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::models::ExpenseForm;
use dioxus::prelude::*;
use std::collections::HashMap;

const PAGE_CSS: &str = r##"
.exp-create-page { max-width: 700px; margin: 0 auto; }
.exp-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.exp-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.exp-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.exp-section { background: #fff; border: 1px solid var(--border-color); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.exp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color); }
.exp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.exp-form-row > * { flex: 1; min-width: 180px; }
.exp-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color); }
@media (max-width: 768px) { .exp-form-row { flex-direction: column; } .exp-form-row > * { min-width: 100%; } .exp-action-bar { flex-direction: column; } }
"##;

fn category_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Travel".to_string(), label: "Travel".to_string() },
        SelectOption { value: "Office Supplies".to_string(), label: "Office Supplies".to_string() },
        SelectOption { value: "Utilities".to_string(), label: "Utilities".to_string() },
        SelectOption { value: "Maintenance".to_string(), label: "Maintenance".to_string() },
        SelectOption { value: "Salary".to_string(), label: "Salary".to_string() },
        SelectOption { value: "Other".to_string(), label: "Other".to_string() },
    ]
}

fn payment_method_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Cash".to_string(), label: "Cash".to_string() },
        SelectOption { value: "Bank".to_string(), label: "Bank Transfer".to_string() },
        SelectOption { value: "Credit Card".to_string(), label: "Credit Card".to_string() },
    ]
}

#[component]
pub fn ExpenseCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let category = use_signal(String::new);
    let description = use_signal(String::new);
    let amount = use_signal(String::new);
    let expense_date = use_signal(String::new);
    let paid_to = use_signal(String::new);
    let payment_method = use_signal(|| "Cash".to_string());
    let notes = use_signal(String::new);

    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    let validate = {
        let mut cat = category.clone();
        let mut desc = description.clone();
        let mut amt = amount.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if cat.read().is_empty() { errs.insert("cat", "Category is required.".to_string()); }
            if desc.read().trim().is_empty() { errs.insert("desc", "Description is required.".to_string()); }
            if let Ok(a) = amt.read().parse::<f64>() { if a <= 0.0 { errs.insert("amt", "Amount must be positive.".to_string()); } } else { errs.insert("amt", "Invalid amount.".to_string()); }
            let valid = errs.is_empty();
            if !valid { toast.warning("Validation Error", "Please fix errors."); }
            valid
        }
    };

    let make_dirty = { let mut d = is_dirty.clone(); move || d.set(true) };

    let on_cat = { let mut c = category.clone(); let mut d = make_dirty.clone(); move |v: String| { c.set(v); d(); } };
    let on_desc = { let mut d = description.clone(); let mut dirty = make_dirty.clone(); move |v: String| { d.set(v); dirty(); } };
    let on_amt = { let mut a = amount.clone(); let mut d = make_dirty.clone(); move |v: String| { a.set(v); d(); } };
    let on_date = { let mut dt = expense_date.clone(); let mut d = make_dirty.clone(); move |v: String| { dt.set(v); d(); } };
    let on_paid = { let mut p = paid_to.clone(); let mut d = make_dirty.clone(); move |v: String| { p.set(v); d(); } };
    let on_method = { let mut m = payment_method.clone(); let mut d = make_dirty.clone(); move |v: String| { m.set(v); d(); } };
    let on_notes = { let mut n = notes.clone(); let mut d = make_dirty.clone(); move |v: String| { n.set(v); d(); } };

    let save = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut cat = category.clone();
        let mut desc = description.clone();
        let mut amt = amount.clone();
        let mut dt = expense_date.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = use_auth().api;
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let c = cat.read().clone(); let d = desc.read().clone(); let a = amt.read().clone();
            let dt_val = dt.read().clone();
            let mut toast = toast.clone(); let nav = nav.clone();
            let api = api.clone();
            let mut saving = saving.clone(); let mut dirty = dirty.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = ExpenseForm {
                    category: c.clone(),
                    description: d.clone(),
                    amount: a.parse::<f64>().unwrap_or(0.0),
                    expense_date: dt_val,
                };
                match client.create_expense(&form).await {
                    Ok(_) => {
                        toast.success("Expense Created", &format!("{} of PKR {} created.", c, a));
                        dirty.set(false);
                        nav.push("/expenses");
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
        let mut cat = category.clone();
        let mut desc = description.clone();
        let mut amt = amount.clone();
        let mut validate = validate.clone();
        let mut i_cat = category.clone();
        let mut i_desc = description.clone();
        let mut i_amt = amount.clone();
        let mut i_date = expense_date.clone();
        let mut i_paid = paid_to.clone();
        let mut i_method = payment_method.clone();
        let mut i_notes = notes.clone();
        let mut dirty = is_dirty.clone();
        let api = use_auth().api;
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let c = cat.read().clone(); let d = desc.read().clone(); let a = amt.read().clone();
            let dt_val = i_date.read().clone();
            let mut toast = toast.clone();
            let api = api.clone();
            let mut saving = saving.clone(); let mut dirty = dirty.clone();
            let mut i_cat = i_cat.clone(); let mut i_desc = i_desc.clone();
            let mut i_amt = i_amt.clone(); let mut i_date = i_date.clone();
            let mut i_paid = i_paid.clone(); let mut i_method = i_method.clone();
            let mut i_notes = i_notes.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = ExpenseForm {
                    category: c.clone(),
                    description: d.clone(),
                    amount: a.parse::<f64>().unwrap_or(0.0),
                    expense_date: dt_val,
                };
                match client.create_expense(&form).await {
                    Ok(_) => {
                        toast.success("Expense Created", &format!("{} of PKR {} created. Creating another…", c, a));
                        i_cat.set(String::new()); i_desc.set(String::new()); i_amt.set(String::new());
                        i_date.set(String::new()); i_paid.set(String::new()); i_method.set("Cash".to_string()); i_notes.set(String::new());
                        dirty.set(false); saving.set(false);
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
        move |_| { if *dirty.read() { modal.set(true); } else { nav.push("/expenses"); } }
    };

    let cat_err = errors.read().get("cat").cloned();
    let desc_err = errors.read().get("desc").cloned();
    let amt_err = errors.read().get("amt").cloned();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page exp-create-page",
            div { class: "exp-create-header",
                div {
                    a { class: "exp-back-link", href: "/expenses", "← Back to Expenses" }
                    h1 { "New Expense" }
                }
                if *is_dirty.read() { span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" } }
            }

            div { class: "exp-section",
                h2 { "Expense Details" }
                div { class: "exp-form-row",
                    div {
                        SearchableSelect {
                            options: category_options(),
                            selected_value: Some(category.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_cat,
                            placeholder: "Select category…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    FormInput {
                        label: Some("Amount (PKR)".to_string()),
                        value: amount.read().clone(),
                        oninput: on_amt,
                        r#type: InputType::Number,
                        placeholder: Some("0.00".to_string()),
                        min: Some(0.0),
                        step: Some(0.01),
                        error: amt_err,
                    }
                }
                div { class: "exp-form-row", style: "margin-top: 12px;",
                    FormInput {
                        label: Some("Description".to_string()),
                        value: description.read().clone(),
                        oninput: on_desc,
                        r#type: InputType::Text,
                        placeholder: Some("What was this expense for?".to_string()),
                        required: true,
                        error: desc_err,
                    }
                }
                div { class: "exp-form-row", style: "margin-top: 12px;",
                    FormInput {
                        label: Some("Expense Date".to_string()),
                        value: expense_date.read().clone(),
                        oninput: on_date,
                        r#type: InputType::Date,
                    }
                    FormInput {
                        label: Some("Paid To".to_string()),
                        value: paid_to.read().clone(),
                        oninput: on_paid,
                        r#type: InputType::Text,
                        placeholder: Some("Vendor or person".to_string()),
                    }
                    div {
                        SearchableSelect {
                            options: payment_method_options(),
                            selected_value: Some(payment_method.read().clone()),
                            on_select: on_method,
                            placeholder: "Payment method…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            div { class: "exp-section",
                h2 { "Additional Info" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes…".to_string()),
                }
            }

            div { class: "exp-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Expense" }
            }

            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true, close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| show_discard_modal.set(false), "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: move |_| { show_discard_modal.set(false); navigator.push("/expenses"); }, "Discard" }
                },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "Unsaved changes will be lost." }
            }
        }
    }
}
