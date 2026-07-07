//! User Edit Page — Form to edit an existing user account.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType,
    SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.user-edit-page { max-width: 700px; margin: 0 auto; }
.user-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.user-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.user-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.user-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.user-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.user-form-row > * { flex: 1; min-width: 180px; }
.user-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.user-field-label { font-size: 13px; font-weight: 500; color: var(--text-primary); margin-bottom: 4px; display: block; }
.user-field-label .required { color: var(--danger, #e53e3e); }
.user-hint { font-size: 12px; color: var(--text-secondary); margin-top: 4px; }
.user-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.user-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .user-form-row { flex-direction: column; } .user-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn UserEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let parsed_id = id.parse::<i64>().unwrap_or(0);

    // Load user data
    let user_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.read().clone();
            client.get_user(parsed_id).await.ok()
        }
    });

    // Form signals — populated after user loads
    let username = use_signal(String::new);
    let full_name = use_signal(String::new);
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm_password = use_signal(String::new);
    let role_id = use_signal(String::new);
    let is_active = use_signal(|| true);
    let saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let data_loaded = use_signal(|| false);

    // Populate form when user data arrives
    {
        let user_data = user_resource.read().as_ref().cloned().flatten();
        let mut username = username.clone();
        let mut full_name = full_name.clone();
        let mut email = email.clone();
        let mut role_id = role_id.clone();
        let mut is_active = is_active.clone();
        let mut data_loaded = data_loaded.clone();
        use_effect(move || {
            if let Some(ref user) = user_data {
                if !*data_loaded.read() {
                    username.set(user.username.clone());
                    full_name.set(user.full_name.clone());
                    email.set(user.email.clone());
                    role_id.set(user.role_id.map(|r| r.to_string()).unwrap_or_else(|| "3".to_string()));
                    is_active.set(user.is_active);
                    data_loaded.set(true);
                }
            }
        });
    }

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

    let is_loading = user_resource.read().is_none();
    let user_data = user_resource.read().as_ref().cloned().flatten();

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "user-edit-page",
                div { class: "user-loading",
                    div { class: "loading-spinner" }
                    span { "Loading user..." }
                }
            }
        };
    }

    if user_data.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "user-edit-page",
                div { class: "user-loading",
                    div { style: "font-size: 40px;", "👤" }
                    h2 { style: "margin: 0;", "User Not Found" }
                    p { "No user with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/users"); }, "← Back to Users" }
                }
            }
        };
    }

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
            // Password is optional on edit — only validate if provided
            if !p.read().is_empty() {
                if *p.read() != *cp.read() {
                    toast.error("Validation", "Passwords do not match.");
                    return false;
                }
                if p.read().len() < 6 {
                    toast.error("Validation", "Password must be at least 6 characters.");
                    return false;
                }
            }
            true
        }
    };

    let save_changes = {
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
        let user_id = parsed_id;
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let mut form = serde_json::json!({
                "username": u.read().trim(),
                "full_name": n.read().trim(),
                "email": e.read().trim(),
                "role_id": r.read().parse::<i64>().unwrap_or(3),
                "is_active": *a.read(),
            });
            // Only include password if provided
            let pw = p.read().clone();
            if !pw.is_empty() {
                form["password"] = serde_json::json!(Some(pw));
            }
            let api = api.clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.update_user(user_id, &form).await {
                    Ok(_) => {
                        toast.success("User Updated", &format!("User '{}' has been updated.", u.read()));
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

    let make_dirty = { let mut d = is_dirty.clone(); move || d.set(true) };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page user-edit-page",
            div { class: "user-edit-header",
                h1 { "Edit User" }
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
                h2 { "Password" }
                p { class: "user-hint", "Leave blank to keep the current password." }
                div { class: "user-form-row",
                    div {
                        label { class: "user-field-label", "New Password" }
                        FormInput { value: "{password}", placeholder: "Min 6 characters (leave blank to keep current)", r#type: InputType::Password,
                            oninput: { let mut p = password.clone(); let mut d = make_dirty.clone(); move |v: String| { p.set(v); d(); } } }
                    }
                    div {
                        label { class: "user-field-label", "Confirm New Password" }
                        FormInput { value: "{confirm_password}", placeholder: "Re-enter new password", r#type: InputType::Password,
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
                        label { class: "user-field-label", "Active" }
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
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Md, onclick: move |_| { let _ = navigator.push("/users"); }, "Cancel" }
                Button { variant: ButtonVariant::Primary, size: ButtonSize::Md, onclick: save_changes, "Save Changes" }
            }
        }
    }
}
