//! Employee Create Page — Form to create a new employee record.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

const PAGE_CSS: &str = r##"
.emp-create-page { max-width: 800px; margin: 0 auto; }
.emp-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.emp-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.emp-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.emp-back-link:hover { text-decoration: underline; }
.emp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.emp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.emp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.emp-form-row > * { flex: 1; min-width: 180px; }
.emp-type-grid { display: flex; gap: 10px; flex-wrap: wrap; }
.emp-type-chip { display: flex; align-items: center; gap: 8px; padding: 10px 14px; border: 1.5px solid var(--border-color, #e0e0e0); border-radius: 8px; cursor: pointer; transition: all 0.15s ease; font-size: 13px; color: var(--text-primary); background: #fff; user-select: none; }
.emp-type-chip:hover { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.04); }
.emp-type-chip-active { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.08); color: var(--accent, #4a90d9); font-weight: 600; }
.emp-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
@media (max-width: 768px) { .emp-form-row { flex-direction: column; } .emp-form-row > * { min-width: 100%; } .emp-action-bar { flex-direction: column; } }
"##;

static NEXT_EMP_ID: AtomicU32 = AtomicU32::new(16);

fn generate_emp_code() -> String {
    let seq = NEXT_EMP_ID.fetch_add(1, Ordering::Relaxed);
    format!("EMP-{:04}", seq)
}

fn department_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Sales".to_string(), label: "Sales".to_string() },
        SelectOption { value: "Purchasing".to_string(), label: "Purchasing".to_string() },
        SelectOption { value: "Warehouse".to_string(), label: "Warehouse".to_string() },
        SelectOption { value: "Manufacturing".to_string(), label: "Manufacturing".to_string() },
        SelectOption { value: "Admin".to_string(), label: "Admin".to_string() },
        SelectOption { value: "Finance".to_string(), label: "Finance".to_string() },
    ]
}

fn designation_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Sales Manager".to_string(), label: "Sales Manager".to_string() },
        SelectOption { value: "Sales Representative".to_string(), label: "Sales Representative".to_string() },
        SelectOption { value: "Sales Trainee".to_string(), label: "Sales Trainee".to_string() },
        SelectOption { value: "Chief Accountant".to_string(), label: "Chief Accountant".to_string() },
        SelectOption { value: "Accounts Clerk".to_string(), label: "Accounts Clerk".to_string() },
        SelectOption { value: "Tax Specialist".to_string(), label: "Tax Specialist".to_string() },
        SelectOption { value: "Procurement Officer".to_string(), label: "Procurement Officer".to_string() },
        SelectOption { value: "Buyer".to_string(), label: "Buyer".to_string() },
        SelectOption { value: "Production Supervisor".to_string(), label: "Production Supervisor".to_string() },
        SelectOption { value: "Machine Operator".to_string(), label: "Machine Operator".to_string() },
        SelectOption { value: "Quality Inspector".to_string(), label: "Quality Inspector".to_string() },
        SelectOption { value: "Warehouse Manager".to_string(), label: "Warehouse Manager".to_string() },
        SelectOption { value: "Store Keeper".to_string(), label: "Store Keeper".to_string() },
        SelectOption { value: "HR Assistant".to_string(), label: "HR Assistant".to_string() },
        SelectOption { value: "Office Assistant".to_string(), label: "Office Assistant".to_string() },
    ]
}

#[component]
pub fn EmployeeCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let emp_code = use_signal(|| generate_emp_code());
    let full_name = use_signal(String::new);
    let email = use_signal(String::new);
    let phone = use_signal(String::new);
    let department = use_signal(String::new);
    let designation = use_signal(String::new);
    let employment_type = use_signal(|| "Permanent".to_string());
    let join_date = use_signal(|| String::new());
    let mut is_active = use_signal(|| true);

    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    let validate = {
        let mut name = full_name.clone();
        let mut dept = department.clone();
        let mut desig = designation.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&'static str, String>::new();
            if name.read().trim().is_empty() { errs.insert("name", "Full name is required.".to_string()); }
            if dept.read().is_empty() { errs.insert("dept", "Department is required.".to_string()); }
            if desig.read().is_empty() { errs.insert("desig", "Designation is required.".to_string()); }
            let valid = errs.is_empty();
            if !valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            valid
        }
    };

    let make_dirty = { let mut d = is_dirty.clone(); move || d.set(true) };

    let on_name = { let mut n = full_name.clone(); let mut d = make_dirty.clone(); move |v: String| { n.set(v); d(); } };
    let on_email = { let mut e = email.clone(); let mut d = make_dirty.clone(); move |v: String| { e.set(v); d(); } };
    let on_phone = { let mut p = phone.clone(); let mut d = make_dirty.clone(); move |v: String| { p.set(v); d(); } };
    let on_dept = { let mut d = department.clone(); let mut dirty = make_dirty.clone(); move |v: String| { d.set(v); dirty(); } };
    let on_desig = { let mut d = designation.clone(); let mut dirty = make_dirty.clone(); move |v: String| { d.set(v); dirty(); } };
    let on_join = { let mut j = join_date.clone(); let mut d = make_dirty.clone(); move |v: String| { j.set(v); d(); } };

    let save_emp = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut name = full_name.clone();
        let mut code = emp_code.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let n = name.read().clone(); let c = code.read().clone();
            let mut toast = toast.clone(); let nav = nav.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("Employee Created", &format!("{} ({}) has been created.", n, c));
                saving.set(false); dirty.set(false);
                nav.push("/crm/employees");
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut name = full_name.clone();
        let mut code = emp_code.clone();
        let mut validate = validate.clone();
        let mut i_code = emp_code.clone();
        let mut i_name = full_name.clone();
        let mut i_email = email.clone();
        let mut i_phone = phone.clone();
        let mut i_dept = department.clone();
        let mut i_desig = designation.clone();
        let mut i_type = employment_type.clone();
        let mut i_join = join_date.clone();
        let mut i_active = is_active.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let n = name.read().clone(); let c = code.read().clone();
            let mut toast = toast.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                toast.success("Employee Created", &format!("{} ({}) created. Creating another…", n, c));
                i_code.set(generate_emp_code());
                i_name.set(String::new()); i_email.set(String::new()); i_phone.set(String::new());
                i_dept.set(String::new()); i_desig.set(String::new());
                i_type.set("Permanent".to_string()); i_join.set(String::new()); i_active.set(true);
                saving.set(false); dirty.set(false);
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let mut dirty = is_dirty.clone();
        let mut nav = navigator.clone();
        move |_| { if *dirty.read() { modal.set(true); } else { nav.push("/crm/employees"); } }
    };

    let type_options = ["Permanent", "Contract", "Intern"];
    let name_err = errors.read().get("name").cloned();
    let dept_err = errors.read().get("dept").cloned();
    let desig_err = errors.read().get("desig").cloned();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page emp-create-page",
            div { class: "emp-create-header",
                div {
                    a { class: "emp-back-link", href: "/crm/employees", "← Back to Employees" }
                    h1 { "New Employee" }
                }
                if *is_dirty.read() { span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" } }
            }

            div { class: "emp-section",
                h2 { "Basic Information" }
                div { class: "emp-form-row",
                    FormInput { label: Some("Employee Code".to_string()), value: emp_code.read().clone(), oninput: move |_| {}, r#type: InputType::Text, disabled: true, hint: Some("Auto-generated".to_string()) }
                    FormInput { label: Some("Full Name".to_string()), value: full_name.read().clone(), oninput: on_name, r#type: InputType::Text, placeholder: Some("Enter full name".to_string()), required: true, error: name_err }
                }
                div { class: "emp-form-row", style: "margin-top: 12px;",
                    FormInput { label: Some("Email".to_string()), value: email.read().clone(), oninput: on_email, r#type: InputType::Email, placeholder: Some("email@company.com".to_string()) }
                    FormInput { label: Some("Phone".to_string()), value: phone.read().clone(), oninput: on_phone, r#type: InputType::Text, placeholder: Some("+92 300 123 4567".to_string()) }
                }
            }

            div { class: "emp-section",
                h2 { "Department & Designation" }
                div { class: "emp-form-row",
                    div {
                        SearchableSelect {
                            options: department_options(),
                            selected_value: Some(department.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_dept,
                            placeholder: "Select department…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    div {
                        SearchableSelect {
                            options: designation_options(),
                            selected_value: Some(designation.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_desig,
                            placeholder: "Select designation…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            div { class: "emp-section",
                h2 { "Employment Details" }
                div { class: "emp-form-row",
                    div {
                        label { class: "cb-input-label", style: "margin-bottom: 6px; display: block;", "Employment Type" }
                        div { class: "emp-type-grid",
                            {type_options.iter().map(|opt| {
                                let is_sel = *employment_type.read() == *opt;
                                let cls = if is_sel { "emp-type-chip emp-type-chip-active" } else { "emp-type-chip" };
                                rsx! {
                                    button {
                                        class: "{cls}", r#type: "button",
                                        onclick: {
                                            let opt = opt.to_string();
                                            let mut et = employment_type.clone();
                                            let mut d = make_dirty.clone();
                                            move |_| { et.set(opt.clone()); d(); }
                                        },
                                        "{opt}"
                                    }
                                }
                            })}
                        }
                    }
                }
                div { class: "emp-form-row", style: "margin-top: 12px;",
                    FormInput {
                        label: Some("Join Date".to_string()),
                        value: join_date.read().clone(),
                        oninput: on_join,
                        r#type: InputType::Date,
                        placeholder: Some("2026-01-01".to_string()),
                    }
                    div {
                        label { class: "cb-input-label", style: "margin-bottom: 6px; display: block;", "Status" }
                        button {
                            class: if *is_active.read() { "emp-type-chip emp-type-chip-active" } else { "emp-type-chip" },
                            onclick: move |_| { let v = !*is_active.read(); is_active.set(v); is_dirty.set(true); },
                            style: "display: inline-flex; width: auto;",
                            span { if *is_active.read() { "✅" } else { "⛔" } }
                            span { if *is_active.read() { " Active" } else { " Inactive" } }
                        }
                    }
                }
            }

            div { class: "emp-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save_emp, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Employee" }
            }

            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true, close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| show_discard_modal.set(false), "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: move |_| { show_discard_modal.set(false); navigator.push("/crm/employees"); }, "Discard" }
                },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "You have unsaved changes. Are you sure you want to discard?" }
            }
        }
    }
}
