//! Supplier Edit Page

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType, use_toast};
use crate::models::SupplierForm;
use dioxus::prelude::*;

const EDIT_CSS: &str = r#"
.sp-edit-page { max-width: 800px; margin: 0 auto; }
.sp-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.sp-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.sp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.sp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.sp-form-row > * { flex: 1; min-width: 180px; }
.sp-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.sp-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.sp-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .sp-form-row { flex-direction: column; } .sp-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn SupplierEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_supplier(parsed_id).await.ok()
        }
    });

    let supplier_code = use_signal(String::new);
    let supplier_name = use_signal(String::new);
    let email = use_signal(String::new);
    let phone = use_signal(String::new);
    let address = use_signal(String::new);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);

    {
        let res = resource.clone();
        let mut sc = supplier_code.clone();
        let mut sn = supplier_name.clone();
        let mut em = email.clone();
        let mut ph = phone.clone();
        let mut addr = address.clone();
        let mut ld = loaded.clone();
        use_effect(move || {
            if !*ld.read() {
                let guard = res.read();
                if let Some(Some(ref s)) = &*guard {
                    if !*ld.read() {
                        sc.set(s.supplier_code.clone());
                        sn.set(s.supplier_name.clone());
                        em.set(s.email.clone());
                        ph.set(s.phone.clone());
                        addr.set(s.address.clone());
                        ld.set(true);
                    }
                }
            }
        });
    }

    if resource.read().is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "sp-edit-page", div { class: "sp-loading", div { class: "loading-spinner" }, span { "Loading supplier..." } } }
        };
    }
    let data = resource.read().clone().flatten();
    if data.is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "sp-edit-page", div { class: "sp-loading", h2 { "Supplier Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/suppliers"); }, "\u{2190} Back" } } }
        };
    }

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let sc = supplier_code.clone();
        let sn = supplier_name.clone();
        let em = email.clone();
        let ph = phone.clone();
        let addr = address.clone();
        move |_| {
            saving.set(true);
            let form = SupplierForm {
                supplier_code: sc.read().clone(),
                supplier_name: sn.read().clone(),
                email: { let v = em.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                phone: { let v = ph.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                address: { let v = addr.read(); if v.is_empty() { None } else { Some(v.clone()) } },
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let sn_display = sn.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_supplier(parsed_id, &form).await {
                    Ok(_) => {
                        toast.success("Supplier Updated", &format!("{} updated.", sn_display));
                        nav.push(format!("/suppliers/{}", parsed_id));
                    }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page sp-edit-page",
            div { class: "sp-edit-header", h1 { "Edit Supplier" } }
            div { class: "sp-section",
                h2 { "Basic Information" }
                div { class: "sp-form-row",
                    FormInput { label: Some("Supplier Code *".to_string()), value: "{supplier_code}", placeholder: "e.g. SUP-001", r#type: InputType::Text, oninput: { let mut s = supplier_code.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Supplier Name *".to_string()), value: "{supplier_name}", placeholder: "e.g. ABC Suppliers", r#type: InputType::Text, oninput: { let mut s = supplier_name.clone(); move |v| { s.set(v); } } }
                }
                div { class: "sp-form-row",
                    FormInput { label: Some("Email".to_string()), value: "{email}", placeholder: "email@example.com", r#type: InputType::Email, oninput: { let mut s = email.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Phone".to_string()), value: "{phone}", placeholder: "+92 300 1234567", r#type: InputType::Tel, oninput: { let mut s = phone.clone(); move |v| { s.set(v); } } }
                }
                div { class: "sp-form-row",
                    FormInput { label: Some("Address".to_string()), value: "{address}", placeholder: "Full address", r#type: InputType::Text, oninput: { let mut s = address.clone(); move |v| { s.set(v); } } }
                }
            }
            div { class: "sp-actions",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/suppliers/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
