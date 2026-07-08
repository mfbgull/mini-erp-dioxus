//! Customer Edit Page

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast};
use crate::models::CustomerForm;
use dioxus::prelude::*;

const EDIT_CSS: &str = r#"
.cust-edit-page { max-width: 800px; margin: 0 auto; }
.cust-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.cust-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.cust-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.cust-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.cust-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.cust-form-row > * { flex: 1; min-width: 180px; }
.cust-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.cust-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.cust-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .cust-form-row { flex-direction: column; } .cust-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn CustomerEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_customer(parsed_id).await.ok()
        }
    });

    let customer_code = use_signal(String::new);
    let customer_name = use_signal(String::new);
    let email = use_signal(String::new);
    let phone = use_signal(String::new);
    let billing_address = use_signal(String::new);
    let shipping_address = use_signal(String::new);
    let payment_terms = use_signal(|| "Net 30".to_string());
    let credit_limit = use_signal(String::new);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);

    {
        let res = resource.clone();
        let mut cc = customer_code.clone();
        let mut cn = customer_name.clone();
        let mut em = email.clone();
        let mut ph = phone.clone();
        let mut ba = billing_address.clone();
        let mut sa = shipping_address.clone();
        let mut pt = payment_terms.clone();
        let mut cl = credit_limit.clone();
        let mut ld = loaded.clone();
        use_effect(move || {
            if !*ld.read() {
                let guard = res.read();
                if let Some(Some(ref c)) = &*guard {
                    if !*ld.read() {
                        cc.set(c.customer_code.clone());
                        cn.set(c.customer_name.clone());
                        em.set(c.email.clone());
                        ph.set(c.phone.clone());
                        ba.set(c.billing_address.clone());
                        sa.set(c.shipping_address.clone());
                        pt.set(c.payment_terms.clone());
                        cl.set(c.credit_limit.to_string());
                        ld.set(true);
                    }
                }
            }
        });
    }

    if resource.read().is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "cust-edit-page", div { class: "cust-loading", div { class: "loading-spinner" }, span { "Loading customer..." } } }
        };
    }
    let data = resource.read().clone().flatten();
    if data.is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "cust-edit-page", div { class: "cust-loading", h2 { "Customer Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/customers"); }, "\u{2190} Back" } } }
        };
    }

    let terms_options = vec![
        SelectOption { value: "Net 30".to_string(), label: "Net 30".to_string() },
        SelectOption { value: "Net 15".to_string(), label: "Net 15".to_string() },
        SelectOption { value: "COD".to_string(), label: "COD".to_string() },
        SelectOption { value: "Due on Receipt".to_string(), label: "Due on Receipt".to_string() },
        SelectOption { value: "Net 60".to_string(), label: "Net 60".to_string() },
    ];

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let cc = customer_code.clone();
        let cn = customer_name.clone();
        let em = email.clone();
        let ph = phone.clone();
        let ba = billing_address.clone();
        let sa = shipping_address.clone();
        let pt = payment_terms.clone();
        let cl = credit_limit.clone();
        move |_| {
            if cn.read().trim().is_empty() { toast.error("Validation", "Customer name is required."); return; }
            saving.set(true);
            let form = CustomerForm {
                customer_code: cc.read().clone(),
                customer_name: cn.read().clone(),
                email: { let v = em.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                phone: { let v = ph.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                billing_address: { let v = ba.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                shipping_address: { let v = sa.read(); if v.is_empty() { None } else { Some(v.clone()) } },
                payment_terms: Some(pt.read().clone()),
                credit_limit: cl.read().parse::<f64>().ok(),
                opening_balance: None,
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let cn = cn.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_customer(parsed_id, &form).await {
                    Ok(_) => {
                        toast.success("Customer Updated", &format!("{} updated.", cn));
                        nav.push(format!("/customers/{}", parsed_id));
                    }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page cust-edit-page",
            div { class: "cust-edit-header", h1 { "Edit Customer" } }
            div { class: "cust-section",
                h2 { "Customer Information" }
                div { class: "cust-form-row",
                    FormInput { label: Some("Customer Code *".to_string()), value: "{customer_code}", placeholder: "e.g. CUST-001", r#type: InputType::Text, oninput: { let mut s = customer_code.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Customer Name *".to_string()), value: "{customer_name}", placeholder: "e.g. Acme Corp", r#type: InputType::Text, oninput: { let mut s = customer_name.clone(); move |v| { s.set(v); } } }
                }
                div { class: "cust-form-row",
                    FormInput { label: Some("Email".to_string()), value: "{email}", placeholder: "contact@example.com", r#type: InputType::Email, oninput: { let mut s = email.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Phone".to_string()), value: "{phone}", placeholder: "+1 555-1234", r#type: InputType::Text, oninput: { let mut s = phone.clone(); move |v| { s.set(v); } } }
                }
                div { class: "cust-form-row",
                    FormInput { label: Some("Billing Address".to_string()), value: "{billing_address}", placeholder: "123 Main St", r#type: InputType::Text, oninput: { let mut s = billing_address.clone(); move |v| { s.set(v); } } }
                }
                div { class: "cust-form-row",
                    FormInput { label: Some("Shipping Address".to_string()), value: "{shipping_address}", placeholder: "123 Main St", r#type: InputType::Text, oninput: { let mut s = shipping_address.clone(); move |v| { s.set(v); } } }
                }
                div { class: "cust-form-row",
                    div {
                        span { class: "cb-input-label", style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Payment Terms" }
                        SearchableSelect {
                            selected_value: Some(payment_terms.read().clone()),
                            on_select: { let mut s = payment_terms.clone(); move |v: String| { s.set(v); } },
                            options: terms_options,
                            placeholder: "Select terms...".to_string(),
                        }
                    }
                    FormInput { label: Some("Credit Limit".to_string()), value: "{credit_limit}", placeholder: "e.g. 50000", r#type: InputType::Number, oninput: { let mut s = credit_limit.clone(); move |v| { s.set(v); } } }
                }
            }
            div { class: "cust-actions",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/customers/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
