//! Customer Create Page — form to add a new customer

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::models::CustomerForm;
use dioxus::prelude::*;

/// Inline CSS for the customer create page.
const CUSTOMER_CREATE_CSS: &str = r#"
.customer-create-page { max-width: 800px; margin: 0 auto; }
.customer-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.customer-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.customer-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; cursor: pointer; }
.customer-back-link:hover { text-decoration: underline; }
.customer-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.customer-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.customer-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.customer-form-row > * { flex: 1; min-width: 180px; }
.customer-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.customer-field-label { font-size: 13px; font-weight: 500; color: var(--text-primary); margin-bottom: 4px; display: block; }
.customer-field-label .required { color: var(--danger, #e53e3e); }
@media (max-width: 768px) { .customer-form-row { flex-direction: column; } .customer-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn CustomerCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    // ── Form state ──
    let customer_code = use_signal(|| String::new());
    let customer_name = use_signal(|| String::new());
    let email = use_signal(|| String::new());
    let phone = use_signal(|| String::new());
    let billing_address = use_signal(|| String::new());
    let shipping_address = use_signal(|| String::new());
    let payment_terms = use_signal(|| "Net 30".to_string());
    let credit_limit = use_signal(|| String::new());
    let opening_balance = use_signal(|| String::new());

    // ── Payment terms options ──
    let terms_options = vec![
        SelectOption { value: "Net 30".to_string(), label: "Net 30".to_string() },
        SelectOption { value: "Net 15".to_string(), label: "Net 15".to_string() },
        SelectOption { value: "COD".to_string(), label: "COD".to_string() },
        SelectOption { value: "Due on Receipt".to_string(), label: "Due on Receipt".to_string() },
        SelectOption { value: "Net 60".to_string(), label: "Net 60".to_string() },
    ];

    // ── Dirty tracking & discard modal ──
    let dirty = use_signal(|| false);
    let show_discard = use_signal(|| false);
    let saving = use_signal(|| false);

    // ── Input handlers ──
    let on_code_change = {
        let mut s = customer_code.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_name_change = {
        let mut s = customer_name.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_email_change = {
        let mut s = email.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_phone_change = {
        let mut s = phone.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_ba_change = {
        let mut s = billing_address.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_sa_change = {
        let mut s = shipping_address.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_terms_change = {
        let mut s = payment_terms.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_cl_change = {
        let mut s = credit_limit.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };
    let on_ob_change = {
        let mut s = opening_balance.clone();
        let mut d = dirty.clone();
        move |v: String| { s.set(v); d.set(true); }
    };

    // ── Build form data ──
    let build_form = move || -> CustomerForm {
        CustomerForm {
            customer_code: customer_code.read().clone(),
            customer_name: customer_name.read().clone(),
            email: {
                let v = email.read().clone();
                if v.is_empty() { None } else { Some(v) }
            },
            phone: {
                let v = phone.read().clone();
                if v.is_empty() { None } else { Some(v) }
            },
            billing_address: {
                let v = billing_address.read().clone();
                if v.is_empty() { None } else { Some(v) }
            },
            shipping_address: {
                let v = shipping_address.read().clone();
                if v.is_empty() { None } else { Some(v) }
            },
            payment_terms: Some(payment_terms.read().clone()),
            credit_limit: credit_limit.read().parse::<f64>().ok(),
            opening_balance: opening_balance.read().parse::<f64>().ok(),
        }
    };

    // ── Save ──
    let save_customer = {
        let mut saving = saving.clone();
        let mut dirty = dirty.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        move |_| {
            saving.set(true);
            let form = build_form();
            if form.customer_code.trim().is_empty() || form.customer_name.trim().is_empty() {
                toast.error("Validation Error", "Customer code and name are required.");
                saving.set(false);
                return;
            }
            let client = api.with(|c| c.clone());
            let mut toast = toast.clone();
            let nav = nav.clone();
            let mut dirty = dirty.clone();
            let mut saving = saving.clone();
            spawn(async move {
                match client.create_customer(&form).await {
                    Ok(c) => {
                        toast.success("Customer Created", &format!("Customer {} created.", c.customer_name));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/customers");
                    }
                    Err(e) => {
                        toast.error("Failed", &format!("Could not create customer: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    // ── Save & New ──
    let save_and_new = {
        let mut saving = saving.clone();
        let mut dirty = dirty.clone();
        let mut toast = toast.clone();
        let mut cc = customer_code.clone();
        let mut cn = customer_name.clone();
        let mut em = email.clone();
        let mut ph = phone.clone();
        let mut ba = billing_address.clone();
        let mut sa = shipping_address.clone();
        let mut cl = credit_limit.clone();
        let mut ob = opening_balance.clone();
        let mut pt = payment_terms.clone();
        move |_| {
            saving.set(true);
            let form = build_form();
            if form.customer_code.trim().is_empty() || form.customer_name.trim().is_empty() {
                toast.error("Validation Error", "Customer code and name are required.");
                saving.set(false);
                return;
            }
            let client = api.with(|c| c.clone());
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let mut cc = cc.clone();
            let mut cn = cn.clone();
            let mut em = em.clone();
            let mut ph = ph.clone();
            let mut ba = ba.clone();
            let mut sa = sa.clone();
            let mut cl = cl.clone();
            let mut ob = ob.clone();
            let mut pt = pt.clone();
            spawn(async move {
                match client.create_customer(&form).await {
                    Ok(_) => {
                        toast.success("Customer Created", "Ready for next customer.");
                        saving.set(false);
                        cc.set(String::new());
                        cn.set(String::new());
                        em.set(String::new());
                        ph.set(String::new());
                        ba.set(String::new());
                        sa.set(String::new());
                        cl.set(String::new());
                        ob.set(String::new());
                        pt.set("Net 30".to_string());
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Failed", &format!("Could not create customer: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    // ── Navigation guard ──
    let go_back = {
        let dirty = dirty.clone();
        let mut show = show_discard.clone();
        move |_| {
            if dirty() {
                show.set(true);
            } else {
                navigator.push("/customers");
            }
        }
    };

    let confirm_discard = {
        let mut show = show_discard.clone();
        let nav = navigator.clone();
        move |_| {
            show.set(false);
            nav.push("/customers");
        }
    };

    let is_valid = !customer_code.read().is_empty() && !customer_name.read().is_empty();

    rsx! {
        style { "{CUSTOMER_CREATE_CSS}" }

        div { class: "customer-create-page",

            div { class: "customer-back-link", onclick: go_back, "← Back to Customers" }

            div { class: "customer-create-header",
                h1 { "New Customer" }
            }

            div { class: "customer-section",
                h2 { "Customer Information" }

                div { class: "customer-form-row",
                    FormInput {
                        label: Some("Customer Code *".to_string()),
                        value: customer_code.read().clone(),
                        oninput: on_code_change,
                        r#type: InputType::Text,
                        placeholder: Some("e.g., CUST-001".to_string()),
                    }
                    FormInput {
                        label: Some("Customer Name *".to_string()),
                        value: customer_name.read().clone(),
                        oninput: on_name_change,
                        r#type: InputType::Text,
                        placeholder: Some("e.g., Acme Corp".to_string()),
                    }
                }

                div { class: "customer-form-row",
                    FormInput {
                        label: Some("Email".to_string()),
                        value: email.read().clone(),
                        oninput: on_email_change,
                        r#type: InputType::Email,
                        placeholder: Some("contact@example.com".to_string()),
                    }
                    FormInput {
                        label: Some("Phone".to_string()),
                        value: phone.read().clone(),
                        oninput: on_phone_change,
                        r#type: InputType::Text,
                        placeholder: Some("+1 555-1234".to_string()),
                    }
                }

                div { class: "customer-form-row",
                    FormInput {
                        label: Some("Billing Address".to_string()),
                        value: billing_address.read().clone(),
                        oninput: on_ba_change,
                        r#type: InputType::Text,
                        placeholder: Some("123 Main St, City".to_string()),
                    }
                }

                div { class: "customer-form-row",
                    FormInput {
                        label: Some("Shipping Address".to_string()),
                        value: shipping_address.read().clone(),
                        oninput: on_sa_change,
                        r#type: InputType::Text,
                        placeholder: Some("123 Main St, City".to_string()),
                    }
                }

                div { class: "customer-form-row",
                    div {
                        span { class: "customer-field-label", "Payment Terms" }
                        SearchableSelect {
                            selected_value: Some(payment_terms.read().clone()),
                            on_select: on_terms_change,
                            options: terms_options.clone(),
                            placeholder: "Select terms…".to_string(),
                        }
                    }
                    FormInput {
                        label: Some("Credit Limit".to_string()),
                        value: credit_limit.read().clone(),
                        oninput: on_cl_change,
                        r#type: InputType::Number,
                        placeholder: Some("e.g., 50000".to_string()),
                    }
                    FormInput {
                        label: Some("Opening Balance".to_string()),
                        value: opening_balance.read().clone(),
                        oninput: on_ob_change,
                        r#type: InputType::Number,
                        placeholder: Some("0.00".to_string()),
                    }
                }
            }

            div { class: "customer-actions",
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Md,
                    disabled: saving(),
                    onclick: go_back,
                    "Cancel"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Md,
                    disabled: !is_valid || saving(),
                    onclick: save_and_new,
                    "Save & New"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    size: ButtonSize::Md,
                    disabled: !is_valid || saving(),
                    onclick: save_customer,
                    "Save"
                }
            }
        }

        // ── Discard confirmation modal ──
        Modal {
            is_open: show_discard,
            title: Some("Discard Changes?".to_string()),
            size: ModalSize::Sm,
            close_on_backdrop: true,
            close_on_escape: true,
            footer: rsx! {
                Button { variant: ButtonVariant::Secondary, onclick: {
                    let mut sd = show_discard.clone();
                    move |_| sd.set(false)
                }, "Stay" }
                Button { variant: ButtonVariant::Danger, onclick: confirm_discard, "Discard" }
            },
            p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;",
                "You have unsaved changes. Are you sure you want to leave?"
            }
        }
    }
}
