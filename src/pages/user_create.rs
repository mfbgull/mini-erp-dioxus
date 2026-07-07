//! User Create Page — Form to create a new user account.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType,
    SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.user-create-page { max-width: 700px; margin: 0 auto; }
.user-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.user-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.user-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.user-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.user-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.user-form-row > * { flex: 1; min-width: 180px; }
.user-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.user-field-label { font-size: 13px; font-weight: 500; color: var(--text-primary); margin-bottom: 4px; display: block; }
.user-field-label .required { color: var(--danger, #e53e3e); }
@media (max-width: 768px) { .user-form-row { flex-direction: column; } .user-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn UserCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let username = use_signal(String::new);
    let full_name = use_signal(String::new);
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm_password = use_signal(String::new);
    let role_id = use_signal(|| "3".to_string()); // default to viewer
    let is_active = use_signal(|| true);
    let saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);

    // Load roles
    let roles = use_signal(Vec::<(i64, String)>::new);
    {
        let api = api.clone();
        let mut roles = roles.clone();
        use_effect(move || {
            let api = api.clone();
            let mut roles = roles.clone();
            spawn(async move {
                let client = api.read().clone();
                if let Ok(list) = client.list_roles().await {
                    let parsed: Vec<(i64, String)> = list.iter()
                        .map(|r| (r.id, r.role_name.clone()))
                        .collect();
                    roles.set(parsed);
                }
            });
        });
    }

    let role_options: Vec<SelectOption> = roles.read().iter()
        .map(|(id, name)| SelectOption { value: id.to_string(), label: name.clone() })
        .collect();

    let validate = {
        let u = username.clone();
        let n = full_name.clone();
        let p = password.clone();
        let cp = confirm_password.clone();
        let mut toast = toast.clone();
        move || -> bool {
            if u.read().trim().is_empty() {
                toast.error("Validation", "Username is required.");
                return false;
            }
            if n.read().trim().is_empty() {
                toast.error("Validation", "Full name is required.");
                return false;
            }
            if p.read().is_empty() {
                toast.error("Validation", "Password is required.");
                return false;
            }
            if *p.read() != *cp.read() {
                toast.error("Validation", "Passwords do not match.");
                return false;
            }
            if p.read().len() < 6 {
                toast.error("Validation", "Password must be at least 6 characters.");
                return false;
            }
            true
        }
    };

    let save_user = {
        let api = api.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut u = username.clone();
        let mut n = full_name.clone();
        let mut e = email.clone();
        let mut p = password.clone();
        let mut r = role_id.clone();
        let mut a = is_active.clone();
        let mut saving = saving.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let form = serde_json::json!({
                "username": u.read().trim(),
                "full_name": n.read().trim(),
                "email": e.read().trim(),
                "password": Some(p.read().clone()),
                "role_id": r.read().parse::<i64>().unwrap_or(3),
                "is_active": *a.read(),
            });
            let api = api.clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.create_user(&form).await {
                    Ok(_) => {
                        toast.success("User Created", &format!("User '{}' has been created.", u.read()));
                        dirty.set(false);
                        nav.push("/users");
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
        let mut u = username.clone();
        let mut n = full_name.clone();
        let mut e = email.clone();
        let mut p = password.clone();
        let mut cp = confirm_password.clone();
        let mut r = role_id.clone();
        let mut a = is_active.clone();
        let mut saving = saving.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let form = serde_json::json!({
                "username": u.read().trim(),
                "full_name": n.read().trim(),
                "email": e.read().trim(),
                "password": Some(p.read().clone()),
                "role_id": r.read().parse::<i64>().unwrap_or(3),
                "is_active": *a.read(),
            });
            let api = api.clone();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.create_user(&form).await {
                    Ok(_) => {
                        toast.success("User Created", "User created. Ready for next entry.");
                        u.set(String::new());
                        n.set(String::new());
                        e.set(String::new());
                        p.set(String::new());
                        cp.set(String::new());
                        r.set("3".to_string());
                        a.set(true);
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

    let make_dirty = { let mut d = is_dirty.clone(); move || d.set(true) };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page user-create-page",
            div { class: "user-create-header",
                h1 { "New User" }
            }

            div { class: "user-section",
                h2 { "Account Information" }
                div { class: "user-form-row",
                    div {
                        label { class: "user-field-label", "Username ", span { class: "required", "*" } }
                        FormInput { value: "{username}", placeholder: "e.g. john.doe", r#type: InputType::Text,
                            oninput: { let mut u = username.clone(); let mut d = make_dirty.clone(); move |v: String| { u.set(v); d(); } } }
                    }
                    div {
                        label { class: "user-field-label", "Full Name ", span { class: "required", "*" } }
                        FormInput { value: "{full_name}", placeholder: "John Doe", r#type: InputType::Text,
                            oninput: { let mut n = full_name.clone(); let mut d = make_dirty.clone(); move |v: String| { n.set(v); d(); } } }
                    }
                }
                div { class: "user-form-row",
                    div {
                        label { class: "user-field-label", "Email" }
                        FormInput { value: "{email}", placeholder: "john@example.com", r#type: InputType::Email,
                            oninput: { let mut e = email.clone(); let mut d = make_dirty.clone(); move |v: String| { e.set(v); d(); } } }
                    }
                }
            }

            div { class: "user-section",
                h2 { "Security" }
                div { class: "user-form-row",
                    div {
                        label { class: "user-field-label", "Password ", span { class: "required", "*" } }
                        FormInput { value: "{password}", placeholder: "Min 6 characters", r#type: InputType::Password,
                            oninput: { let mut p = password.clone(); let mut d = make_dirty.clone(); move |v: String| { p.set(v); d(); } } }
                    }
                    div {
                        label { class: "user-field-label", "Confirm Password ", span { class: "required", "*" } }
                        FormInput { value: "{confirm_password}", placeholder: "Re-enter password", r#type: InputType::Password,
                            oninput: { let mut cp = confirm_password.clone(); let mut d = make_dirty.clone(); move |v: String| { cp.set(v); d(); } } }
                    }
                }
            }

            div { class: "user-section",
                h2 { "Role & Status" }
                div { class: "user-form-row",
                    div {
                        label { class: "user-field-label", "Role" }
                        SearchableSelect {
                            options: role_options,
                            selected_value: Some(role_id.read().clone()),
                            on_select: { let mut r = role_id.clone(); let mut d = make_dirty.clone(); move |v: String| { r.set(v); d(); } },
                            placeholder: "Select role...",
                            searchable: true,
                        }
                    }
                    div {
                        label { class: "user-field-label", "Active"
                        }
                        div { style: "display: flex; align-items: center; gap: 8px; margin-top: 8px;",
                            input {
                                r#type: "checkbox",
                                checked: *is_active.read(),
                                onchange: { let mut a = is_active.clone(); let mut d = make_dirty.clone(); move |e: Event<FormData>| {
                                    a.set(e.value() == "true");
                                    d();
                                }}
                            }
                            span { style: "font-size: 13px; color: var(--text-secondary);", "User can log in" }
                        }
                    }
                }
            }

            div { class: "user-actions",
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Md, onclick: save_and_new, "Save & New" }
                Button { variant: ButtonVariant::Primary, size: ButtonSize::Md, onclick: save_user, "Create User" }
            }
        }
    }
}
