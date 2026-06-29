//! Supplier Create Page — form to add a new supplier

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonSize, ButtonVariant, FormInput, InputType, use_toast};
use crate::models::SupplierForm;
use dioxus::prelude::*;

const SUPPLIER_CREATE_CSS: &str = r#"
.sp-create-page { max-width: 800px; margin: 0 auto; }
.sp-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.sp-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.sp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.sp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.sp-form-row > * { flex: 1; min-width: 180px; }
.sp-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.sp-field-label { font-size: 13px; font-weight: 500; color: var(--text-primary); margin-bottom: 4px; display: block; }
.sp-field-label .required { color: var(--danger, #e53e3e); }
@media (max-width: 768px) { .sp-form-row { flex-direction: column; } .sp-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn SupplierCreatePage() -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let supplier_code = use_signal(String::new);
    let supplier_name = use_signal(String::new);
    let email = use_signal(String::new);
    let phone = use_signal(String::new);
    let address = use_signal(String::new);
    let mut saving = use_signal(|| false);

    // ── Input handlers ──
    let on_code_change = { let mut s = supplier_code.clone(); move |v: String| { s.set(v); } };
    let on_name_change = { let mut s = supplier_name.clone(); move |v: String| { s.set(v); } };
    let on_email_change = { let mut s = email.clone(); move |v: String| { s.set(v); } };
    let on_phone_change = { let mut s = phone.clone(); move |v: String| { s.set(v); } };
    let on_address_change = { let mut s = address.clone(); move |v: String| { s.set(v); } };

    let validate = {
        let mut toast = toast.clone();
        let code = supplier_code.clone();
        let name = supplier_name.clone();
        move || -> bool {
            if code.read().trim().is_empty() {
                toast.error("Validation Error", "Supplier code is required.");
                return false;
            }
            if name.read().trim().is_empty() {
                toast.error("Validation Error", "Supplier name is required.");
                return false;
            }
            true
        }
    };

    let build_form = {
        let code = supplier_code.clone();
        let name = supplier_name.clone();
        let email = email.clone();
        let phone = phone.clone();
        let addr = address.clone();
        move || SupplierForm {
            supplier_code: code.read().trim().to_string(),
            supplier_name: name.read().trim().to_string(),
            email: Some(email.read().trim().to_string()).filter(|s| !s.is_empty()),
            phone: Some(phone.read().trim().to_string()).filter(|s| !s.is_empty()),
            address: Some(addr.read().trim().to_string()).filter(|s| !s.is_empty()),
        }
    };

    let save_supplier = {
        let api = api.clone();
        let toast = toast.clone();
        let navigator = navigator.clone();
        let mut saving = saving.clone();
        let mut validate = validate.clone();
        let mut build_form = build_form.clone();
        move |_: MouseEvent| {
            if !validate() { return; }
            saving.set(true);
            let api = api.clone();
            let mut toast = toast.clone();
            let navigator = navigator.clone();
            let mut saving = saving.clone();
            let form = build_form();
            spawn(async move {
                let client = api.read().clone();
                match client.create_supplier(&form).await {
                    Ok(s) => {
                        toast.success("Supplier Created", &format!("{} ({})", s.supplier_name, s.supplier_code));
                        navigator.push("/suppliers");
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
        let api = api.clone();
        let mut toast = toast.clone();
        let mut saving = saving.clone();
        let mut validate = validate.clone();
        let mut build_form = build_form.clone();
        let mut supplier_code = supplier_code.clone();
        let mut supplier_name = supplier_name.clone();
        let mut email = email.clone();
        let mut phone = phone.clone();
        let mut address = address.clone();
        move |_: MouseEvent| {
            if !validate() { return; }
            saving.set(true);
            let api = api.clone();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let form = build_form();
            spawn(async move {
                let client = api.read().clone();
                match client.create_supplier(&form).await {
                    Ok(_) => {
                        toast.success("Supplier Created", "Ready for next entry.");
                        supplier_code.set(String::new());
                        supplier_name.set(String::new());
                        email.set(String::new());
                        phone.set(String::new());
                        address.set(String::new());
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

    rsx! {
        style { "{SUPPLIER_CREATE_CSS}" }
        div { class: "page sp-create-page",
            div { class: "sp-create-header",
                h1 { "New Supplier" }
            }

            div { class: "sp-section",
                h2 { "Basic Information" }
                div { class: "sp-form-row",
                    div {
                        label { class: "sp-field-label", "Supplier Code ", span { class: "required", "*" } }
                        FormInput { value: "{supplier_code}", placeholder: "e.g. SUP-001", r#type: InputType::Text, oninput: on_code_change }
                    }
                    div {
                        label { class: "sp-field-label", "Supplier Name ", span { class: "required", "*" } }
                        FormInput { value: "{supplier_name}", placeholder: "e.g. ABC Suppliers", r#type: InputType::Text, oninput: on_name_change }
                    }
                }
                div { class: "sp-form-row",
                    div {
                        label { class: "sp-field-label", "Email" }
                        FormInput { value: "{email}", placeholder: "email@example.com", r#type: InputType::Email, oninput: on_email_change }
                    }
                    div {
                        label { class: "sp-field-label", "Phone" }
                        FormInput { value: "{phone}", placeholder: "+92 300 1234567", r#type: InputType::Tel, oninput: on_phone_change }
                    }
                }
                div { class: "sp-form-row",
                    div {
                        label { class: "sp-field-label", "Address" }
                        FormInput { value: "{address}", placeholder: "Full address", r#type: InputType::Text, oninput: on_address_change }
                    }
                }
            }

            div { class: "sp-actions",
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Md, onclick: save_and_new, "Save & New" }
                Button { variant: ButtonVariant::Primary, size: ButtonSize::Md, onclick: save_supplier, "Save Supplier" }
            }
        }
    }
}
