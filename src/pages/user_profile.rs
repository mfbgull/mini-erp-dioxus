//! User Profile Page — View and edit own profile, change password.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType,
    StatCard, StatCardVariant, use_toast,
};
use crate::models::ActivityLog;
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.profile-page { max-width: 800px; margin: 0 auto; }
.profile-header { display: flex; align-items: center; gap: 16px; margin-bottom: 24px; }
.profile-avatar { width: 72px; height: 72px; border-radius: 50%; background: var(--accent, #4a90d9); display: flex; align-items: center; justify-content: center; font-size: 28px; color: #fff; font-weight: 700; flex-shrink: 0; }
.profile-header-info h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.profile-header-info p { font-size: 13px; color: var(--text-secondary); margin: 4px 0 0 0; }
.profile-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.profile-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.profile-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.profile-form-row > * { flex: 1; min-width: 180px; }
.profile-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.profile-field-label { font-size: 13px; font-weight: 500; color: var(--text-primary); margin-bottom: 4px; display: block; }
.profile-field-label .required { color: var(--danger, #e53e3e); }
.profile-hint { font-size: 12px; color: var(--text-secondary); margin-top: 4px; }
.profile-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.profile-info-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; display: flex; flex-wrap: wrap; gap: 20px; }
.profile-info-field { display: flex; flex-direction: column; gap: 2px; }
.profile-info-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.profile-info-value { font-size: 14px; color: var(--text-primary); font-weight: 500; }
.activity-log { margin-top: 0; }
.activity-log-list { list-style: none; padding: 0; margin: 0; }
.activity-log-item { display: flex; align-items: flex-start; gap: 12px; padding: 10px 0; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.activity-log-item:last-child { border-bottom: none; }
.activity-log-icon { width: 32px; height: 32px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 14px; flex-shrink: 0; }
.activity-log-icon.create { background: #e6f7ed; color: #28a745; }
.activity-log-icon.update { background: #fff3cd; color: #856404; }
.activity-log-icon.delete { background: #fde8e8; color: #dc3545; }
.activity-log-icon.login { background: #e8f0fe; color: #4a90d9; }
.activity-log-icon.default { background: #f5f5f5; color: #6c757d; }
.activity-log-content { flex: 1; min-width: 0; }
.activity-log-action { font-size: 13px; color: var(--text-primary); margin: 0; }
.activity-log-meta { font-size: 11px; color: var(--text-secondary); margin: 2px 0 0 0; }
.activity-log-empty { text-align: center; padding: 24px; color: var(--text-secondary); font-size: 13px; }
@media (max-width: 768px) { .profile-form-row { flex-direction: column; } .profile-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn UserProfilePage() -> Element {
    let toast = use_toast();
    let auth = use_auth();
    let api = auth.api;

    let user = auth.user.read().clone();

    // Editable form signals
    let full_name = use_signal(|| user.as_ref().map(|u| u.full_name.clone()).unwrap_or_default());
    let email = use_signal(|| user.as_ref().map(|u| u.email.clone()).unwrap_or_default());
    let current_password = use_signal(String::new);
    let new_password = use_signal(String::new);
    let confirm_password = use_signal(String::new);
    let saving_profile = use_signal(|| false);
    let saving_password = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);

    // Profile save
    let save_profile = {
        let api = api.clone();
        let mut toast = toast.clone();
        let n = full_name.clone();
        let e = email.clone();
        let mut saving = saving_profile.clone();
        let mut dirty = is_dirty.clone();
        let user_id = user.as_ref().map(|u| u.id).unwrap_or(0);
        move |_| {
            if n.read().trim().is_empty() {
                toast.error("Validation", "Full name is required.");
                return;
            }
            saving.set(true);
            let form = serde_json::json!({
                "full_name": n.read().trim(),
                "email": e.read().trim(),
            });
            let api = api.clone();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.update_user(user_id, &form).await {
                    Ok(_) => {
                        toast.success("Profile Updated", "Your profile has been updated.");
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

    // Password change
    let change_password = {
        let api = api.clone();
        let mut toast = toast.clone();
        let mut cp = current_password.clone();
        let mut np = new_password.clone();
        let mut cn = confirm_password.clone();
        let mut saving = saving_password.clone();
        move |_| {
            if cp.read().is_empty() {
                toast.error("Validation", "Current password is required.");
                return;
            }
            if np.read().is_empty() {
                toast.error("Validation", "New password is required.");
                return;
            }
            if *np.read() != *cn.read() {
                toast.error("Validation", "New passwords do not match.");
                return;
            }
            if np.read().len() < 6 {
                toast.error("Validation", "New password must be at least 6 characters.");
                return;
            }
            saving.set(true);
            let api = api.clone();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let current = cp.read().clone();
            let new_pw = np.read().clone();
            spawn(async move {
                let client = api.read().clone();
                match client.change_password(&current, &new_pw).await {
                    Ok(_) => {
                        toast.success("Password Changed", "Your password has been updated.");
                        cp.set(String::new());
                        np.set(String::new());
                        cn.set(String::new());
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

    // Activity log filters
    let from_date = use_signal(|| {
        let d = chrono::Local::now() - chrono::Duration::days(30);
        d.format("%Y-%m-%d").to_string()
    });
    let to_date = use_signal(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let search_query = use_signal(String::new);

    // Fetch activity logs for current user
    let user_id = user.as_ref().map(|u| u.id).unwrap_or(0);
    let activity_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.read().clone();
            client.list_activity_logs().await.ok()
        }
    });

    let activity_logs: Vec<ActivityLog> = {
        let from = from_date.read().clone();
        let to = to_date.read().clone();
        let query = search_query.read().to_lowercase();
        activity_resource
            .read()
            .as_ref()
            .and_then(|opt| opt.as_ref())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|log| log.user_id == Some(user_id))
            .filter(|log| {
                if log.created_at.is_empty() { return true; }
                let date_part = log.created_at.split(' ').next().unwrap_or(&log.created_at);
                date_part >= from.as_str() && date_part <= to.as_str()
            })
            .filter(|log| {
                if query.is_empty() { return true; }
                let haystack = format!("{} {} {} {}",
                    log.action.to_lowercase(),
                    log.entity_type.to_lowercase(),
                    log.metadata.clone().unwrap_or_default().to_lowercase(),
                    log.ip_address.clone().unwrap_or_default().to_lowercase(),
                );
                haystack.contains(&query)
            })
            .take(50)
            .collect()
    };

    let activity_icon_class = |action: &str| -> &'static str {
        match action {
            a if a.contains("create") || a.contains("Create") => "create",
            a if a.contains("update") || a.contains("Update") || a.contains("edit") || a.contains("Edit") => "update",
            a if a.contains("delete") || a.contains("Delete") => "delete",
            a if a.contains("login") || a.contains("Login") => "login",
            _ => "default",
        }
    };

    let activity_icon = |action: &str| -> &'static str {
        match action {
            a if a.contains("create") || a.contains("Create") => "➕",
            a if a.contains("update") || a.contains("Update") || a.contains("edit") || a.contains("Edit") => "✏️",
            a if a.contains("delete") || a.contains("Delete") => "🗑",
            a if a.contains("login") || a.contains("Login") => "🔑",
            _ => "📋",
        }
    };

    let initials = user.as_ref().map(|u| {
        let parts: Vec<&str> = u.full_name.split_whitespace().collect();
        match parts.len() {
            0 => "?".to_string(),
            1 => parts[0][..1].to_uppercase(),
            _ => format!("{}{}", &parts[0][..1], &parts[parts.len()-1][..1]),
        }
    }).unwrap_or_else(|| "?".to_string());

    let role_display = user.as_ref().map(|u| u.role.clone()).unwrap_or_else(|| "Unknown".to_string());
    let username_display = user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Unknown".to_string());
    let status_display = user.as_ref().map(|u| if u.is_active { "Active".to_string() } else { "Disabled".to_string() }).unwrap_or_else(|| "Unknown".to_string());

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page profile-page",
            div { class: "profile-header",
                div { class: "profile-avatar", "{initials}" }
                div { class: "profile-header-info",
                    h1 { "{full_name}" }
                    p { "@{username_display} · {role_display}" }
                }
            }

            div { class: "profile-kpi-grid",
                StatCard { title: "Role".to_string(), value: role_display.clone(), icon: "🔐".to_string(), variant: StatCardVariant::Default, footer: None }
                StatCard { title: "Status".to_string(), value: status_display.clone(), icon: "✅".to_string(), variant: if status_display == "Active" { StatCardVariant::Success } else { StatCardVariant::Danger }, footer: None }
            }

            div { class: "profile-section",
                h2 { "Personal Information" }
                div { class: "profile-form-row",
                    div {
                        label { class: "profile-field-label", "Full Name ", span { class: "required", "*" } }
                        FormInput { value: "{full_name}", placeholder: "Your full name", r#type: InputType::Text,
                            oninput: { let mut n = full_name.clone(); let mut d = make_dirty.clone(); move |v: String| { n.set(v); d(); } } }
                    }
                    div {
                        label { class: "profile-field-label", "Email" }
                        FormInput { value: "{email}", placeholder: "your@email.com", r#type: InputType::Email,
                            oninput: { let mut e = email.clone(); let mut d = make_dirty.clone(); move |v: String| { e.set(v); d(); } } }
                    }
                }
                div { class: "profile-actions",
                    Button { variant: ButtonVariant::Primary, size: ButtonSize::Md, onclick: save_profile, loading: *saving_profile.read(), "Save Profile" }
                }
            }

            div { class: "profile-section",
                h2 { "Change Password" }
                p { class: "profile-hint", "Enter your current password and a new one." }
                div { class: "profile-form-row",
                    div {
                        label { class: "profile-field-label", "Current Password ", span { class: "required", "*" } }
                        FormInput { value: "{current_password}", placeholder: "Enter current password", r#type: InputType::Password,
                            oninput: { let mut p = current_password.clone(); move |v: String| { p.set(v); } } }
                    }
                }
                div { class: "profile-form-row", style: "margin-top: 12px;",
                    div {
                        label { class: "profile-field-label", "New Password ", span { class: "required", "*" } }
                        FormInput { value: "{new_password}", placeholder: "Min 6 characters", r#type: InputType::Password,
                            oninput: { let mut p = new_password.clone(); move |v: String| { p.set(v); } } }
                    }
                    div {
                        label { class: "profile-field-label", "Confirm New Password ", span { class: "required", "*" } }
                        FormInput { value: "{confirm_password}", placeholder: "Re-enter new password", r#type: InputType::Password,
                            oninput: { let mut p = confirm_password.clone(); move |v: String| { p.set(v); } } }
                    }
                }
                div { class: "profile-actions",
                    Button { variant: ButtonVariant::Primary, size: ButtonSize::Md, onclick: change_password, loading: *saving_password.read(), "Change Password" }
                }
            }

            div { class: "profile-section activity-log",
                div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;",
                    h2 { style: "margin: 0;", "Recent Activity" }
                    div { style: "display: flex; align-items: center; gap: 8px; flex-wrap: wrap;",
                        input {
                            r#type: "text",
                            placeholder: "Search actions...",
                            value: "{search_query}",
                            style: "border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 4px 10px; font-size: 13px; width: 160px;",
                            oninput: { let mut q = search_query.clone(); move |e: Event<FormData>| { q.set(e.value()); } }
                        }
                        label { style: "color: var(--text-secondary); font-size: 13px;", "From" }
                        input {
                            r#type: "date",
                            value: "{from_date}",
                            style: "border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 4px 8px; font-size: 13px;",
                            onchange: { let mut f = from_date.clone(); move |e: Event<FormData>| { f.set(e.value()); } }
                        }
                        label { style: "color: var(--text-secondary); font-size: 13px;", "To" }
                        input {
                            r#type: "date",
                            value: "{to_date}",
                            style: "border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 4px 8px; font-size: 13px;",
                            onchange: { let mut t = to_date.clone(); move |e: Event<FormData>| { t.set(e.value()); } }
                        }
                    }
                }
                if activity_logs.is_empty() {
                    div { class: "activity-log-empty", "No recent activity found." }
                } else {
                    ul { class: "activity-log-list",
                        {activity_logs.into_iter().map(|log| {
                            let icon_cls = activity_icon_class(&log.action);
                            let icon = activity_icon(&log.action);
                            let desc = format!("{} {}", log.action, log.entity_type);
                            let time = log.created_at.clone();
                            let ip = log.ip_address.clone().unwrap_or_default();
                            rsx! {
                                li { class: "activity-log-item",
                                    div { class: "activity-log-icon {icon_cls}", "{icon}" }
                                    div { class: "activity-log-content",
                                        p { class: "activity-log-action", "{desc}" }
                                        p { class: "activity-log-meta",
                                            if !time.is_empty() { "{time}" }
                                            if !ip.is_empty() { " · IP: {ip}" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
