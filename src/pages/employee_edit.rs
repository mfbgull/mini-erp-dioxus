//! Employee Edit Page

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast};
use crate::models::EmployeeForm;
use dioxus::prelude::*;

const EDIT_CSS: &str = r#"
.emp-edit-page { max-width: 800px; margin: 0 auto; }
.emp-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.emp-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.emp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.emp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.emp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.emp-form-row > * { flex: 1; min-width: 180px; }
.emp-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.emp-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.emp-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .emp-form-row { flex-direction: column; } .emp-form-row > * { min-width: 100%; } }
"#;

fn departments() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Management".to_string(), label: "Management".to_string() },
        SelectOption { value: "Sales".to_string(), label: "Sales".to_string() },
        SelectOption { value: "Accounts".to_string(), label: "Accounts".to_string() },
        SelectOption { value: "Procurement".to_string(), label: "Procurement".to_string() },
        SelectOption { value: "Production".to_string(), label: "Production".to_string() },
        SelectOption { value: "Warehouse".to_string(), label: "Warehouse".to_string() },
        SelectOption { value: "HR".to_string(), label: "HR".to_string() },
        SelectOption { value: "IT".to_string(), label: "IT".to_string() },
    ]
}

fn designations() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Manager".to_string(), label: "Manager".to_string() },
        SelectOption { value: "Supervisor".to_string(), label: "Supervisor".to_string() },
        SelectOption { value: "Sales Representative".to_string(), label: "Sales Representative".to_string() },
        SelectOption { value: "Accountant".to_string(), label: "Accountant".to_string() },
        SelectOption { value: "Procurement Officer".to_string(), label: "Procurement Officer".to_string() },
        SelectOption { value: "Production Supervisor".to_string(), label: "Production Supervisor".to_string() },
        SelectOption { value: "Machine Operator".to_string(), label: "Machine Operator".to_string() },
        SelectOption { value: "Warehouse Manager".to_string(), label: "Warehouse Manager".to_string() },
        SelectOption { value: "Store Keeper".to_string(), label: "Store Keeper".to_string() },
        SelectOption { value: "HR Assistant".to_string(), label: "HR Assistant".to_string() },
    ]
}

#[component]
pub fn EmployeeEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_employee(parsed_id).await.ok()
        }
    });

    let employee_code = use_signal(String::new);
    let first_name = use_signal(String::new);
    let last_name = use_signal(String::new);
    let email = use_signal(String::new);
    let phone = use_signal(String::new);
    let cnic_no = use_signal(String::new);
    let address = use_signal(String::new);
    let city = use_signal(String::new);
    let department = use_signal(String::new);
    let designation = use_signal(String::new);
    let salary = use_signal(String::new);
    let bank_name = use_signal(String::new);
    let bank_account_no = use_signal(String::new);
    let emergency_contact_name = use_signal(String::new);
    let emergency_contact_phone = use_signal(String::new);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);

    {
        let res = resource.clone();
        let mut ec = employee_code.clone();
        let mut fn_ = first_name.clone();
        let mut ln = last_name.clone();
        let mut em = email.clone();
        let mut ph = phone.clone();
        let mut cn = cnic_no.clone();
        let mut addr = address.clone();
        let mut ci = city.clone();
        let mut dept = department.clone();
        let mut desig = designation.clone();
        let mut sal = salary.clone();
        let mut bn = bank_name.clone();
        let mut ba = bank_account_no.clone();
        let mut ecn = emergency_contact_name.clone();
        let mut ecp = emergency_contact_phone.clone();
        let mut ld = loaded.clone();
        use_effect(move || {
            if !*ld.read() {
                let guard = res.read();
                if let Some(Some(ref e)) = &*guard {
                    if !*ld.read() {
                        ec.set(e.employee_code.clone());
                        fn_.set(e.first_name.clone());
                        ln.set(e.last_name.clone());
                        em.set(e.email.clone());
                        ph.set(e.phone.clone());
                        cn.set(e.cnic_no.clone());
                        addr.set(e.address.clone());
                        ci.set(e.city.clone());
                        dept.set(e.department.clone());
                        desig.set(e.designation.clone());
                        sal.set(e.salary.to_string());
                        bn.set(e.bank_name.clone());
                        ba.set(e.bank_account_no.clone());
                        ecn.set(e.emergency_contact_name.clone());
                        ecp.set(e.emergency_contact_phone.clone());
                        ld.set(true);
                    }
                }
            }
        });
    }

    if resource.read().is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "emp-edit-page", div { class: "emp-loading", div { class: "loading-spinner" }, span { "Loading employee..." } } }
        };
    }
    let data = resource.read().clone().flatten();
    if data.is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "emp-edit-page", div { class: "emp-loading", h2 { "Employee Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/employees"); }, "\u{2190} Back" } } }
        };
    }

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let ec = employee_code.clone();
        let fn_ = first_name.clone();
        let ln = last_name.clone();
        let em = email.clone();
        let ph = phone.clone();
        let cn = cnic_no.clone();
        let addr = address.clone();
        let ci = city.clone();
        let dept = department.clone();
        let desig = designation.clone();
        let sal = salary.clone();
        let bn = bank_name.clone();
        let ba = bank_account_no.clone();
        let ecn = emergency_contact_name.clone();
        let ecp = emergency_contact_phone.clone();
        move |_| {
            saving.set(true);
            let form = EmployeeForm {
                employee_code: ec.read().clone(),
                first_name: fn_.read().clone(),
                last_name: ln.read().clone(),
                email: { let v = em.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                phone: { let v = ph.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                cnic_no: { let v = cn.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                address: { let v = addr.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                city: { let v = ci.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                department: { let v = dept.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                designation: { let v = desig.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                salary: sal.read().parse::<f64>().ok(),
                bank_name: { let v = bn.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                bank_account_no: { let v = ba.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                emergency_contact_name: { let v = ecn.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                emergency_contact_phone: { let v = ecp.read(); if v.is_empty() { None } else { Some(v.clone()) } },
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let fn_display = fn_.read().clone();
            let ln_display = ln.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_employee(parsed_id, &form).await {
                    Ok(_) => {
                        toast.success("Employee Updated", &format!("{} {} updated.", fn_display, ln_display));
                        nav.push(format!("/employees/{}", parsed_id));
                    }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page emp-edit-page",
            div { class: "emp-edit-header", h1 { "Edit Employee" } }

            div { class: "emp-section",
                h2 { "Personal Information" }
                div { class: "emp-form-row",
                    FormInput { label: Some("Employee Code".to_string()), value: "{employee_code}", placeholder: "EMP-001", r#type: InputType::Text, oninput: { let mut s = employee_code.clone(); move |v| { s.set(v); } } }
                }
                div { class: "emp-form-row",
                    FormInput { label: Some("First Name *".to_string()), value: "{first_name}", placeholder: "John", r#type: InputType::Text, oninput: { let mut s = first_name.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Last Name *".to_string()), value: "{last_name}", placeholder: "Doe", r#type: InputType::Text, oninput: { let mut s = last_name.clone(); move |v| { s.set(v); } } }
                }
                div { class: "emp-form-row",
                    FormInput { label: Some("Email".to_string()), value: "{email}", placeholder: "john@example.com", r#type: InputType::Email, oninput: { let mut s = email.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Phone".to_string()), value: "{phone}", placeholder: "+92 300 1234567", r#type: InputType::Tel, oninput: { let mut s = phone.clone(); move |v| { s.set(v); } } }
                }
                div { class: "emp-form-row",
                    FormInput { label: Some("CNIC No".to_string()), value: "{cnic_no}", placeholder: "XXXXX-XXXXXXX-X", r#type: InputType::Text, oninput: { let mut s = cnic_no.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("City".to_string()), value: "{city}", placeholder: "City", r#type: InputType::Text, oninput: { let mut s = city.clone(); move |v| { s.set(v); } } }
                }
                div { class: "emp-form-row",
                    FormInput { label: Some("Address".to_string()), value: "{address}", placeholder: "Full address", r#type: InputType::Text, oninput: { let mut s = address.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "emp-section",
                h2 { "Employment Details" }
                div { class: "emp-form-row",
                    div {
                        label { style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Department" }
                        SearchableSelect {
                            selected_value: Some(department.read().clone()),
                            on_select: { let mut s = department.clone(); move |v: String| { s.set(v); } },
                            options: departments(),
                            placeholder: "Select department...",
                            searchable: true,
                        }
                    }
                    div {
                        label { style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Designation" }
                        SearchableSelect {
                            selected_value: Some(designation.read().clone()),
                            on_select: { let mut s = designation.clone(); move |v: String| { s.set(v); } },
                            options: designations(),
                            placeholder: "Select designation...",
                            searchable: true,
                        }
                    }
                }
                div { class: "emp-form-row",
                    FormInput { label: Some("Salary".to_string()), value: "{salary}", placeholder: "0.00", r#type: InputType::Number, oninput: { let mut s = salary.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "emp-section",
                h2 { "Bank Information" }
                div { class: "emp-form-row",
                    FormInput { label: Some("Bank Name".to_string()), value: "{bank_name}", placeholder: "e.g. HBL", r#type: InputType::Text, oninput: { let mut s = bank_name.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Account No".to_string()), value: "{bank_account_no}", placeholder: "XXXX-XXXX-XXXX", r#type: InputType::Text, oninput: { let mut s = bank_account_no.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "emp-section",
                h2 { "Emergency Contact" }
                div { class: "emp-form-row",
                    FormInput { label: Some("Contact Name".to_string()), value: "{emergency_contact_name}", placeholder: "Spouse/Parent", r#type: InputType::Text, oninput: { let mut s = emergency_contact_name.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Contact Phone".to_string()), value: "{emergency_contact_phone}", placeholder: "+92 300 1234567", r#type: InputType::Tel, oninput: { let mut s = emergency_contact_phone.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "emp-actions",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/employees/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
