//! User Detail Page — Detail view for a system user with KPI cards,
//! user info, and action bar.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.user-detail-page { max-width: 800px; margin: 0 auto; }
.user-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 20px; gap: 16px; flex-wrap: wrap; }
.user-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.user-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.user-detail-back:hover { text-decoration: underline; }
.user-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.user-detail-title-row h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.user-detail-username { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.user-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.user-status-active { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.user-status-inactive { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.user-status-disabled { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.user-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.user-detail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.user-detail-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.user-detail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.user-detail-field { display: flex; flex-direction: column; gap: 3px; }
.user-detail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.user-detail-field-value { font-size: 14px; color: var(--text-primary); }
.user-detail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.user-detail-actions-left, .user-detail-actions-right { display: flex; align-items: center; gap: 8px; }
.user-detail-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.user-detail-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: user-spin 0.8s linear infinite; }
@keyframes user-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .user-detail-header { flex-direction: column; } .user-detail-kpis { grid-template-columns: 1fr 1fr; } .user-detail-info-grid { grid-template-columns: 1fr; } .user-detail-actions { flex-direction: column; align-items: stretch; } }
"##;

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn UserDetailPage(id: String) -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();

    // ── Async fetch ──
    let api = use_auth().api;
    let id_clone = id.clone();
    let user_resource = use_resource(move || {
        let api = api.clone();
        let id_fetch = id_clone.clone();
        async move {
            let client = api.with(|c| c.clone());
            let parsed = id_fetch.parse::<i64>().ok()?;
            client.get_user(parsed).await.ok().map(|u| crate::pages::user_list::User {
                id: u.id,
                username: u.username,
                full_name: u.full_name,
                email: u.email,
                role: u.role,
                status: if u.is_active { "Active".to_string() } else { "Inactive".to_string() },
                last_login: u.last_login.unwrap_or_default(),
                created_at: u.created_at.unwrap_or_default(),
            })
        }
    });

    let is_loading = user_resource.read().is_none();
    let user_opt = user_resource.read().as_ref().cloned().flatten();

    // ── Modal state ──
    let mut show_delete_modal = use_signal(|| false);

    // ── Handlers ──

    let on_back = move |_: Event<MouseData>| {
        navigator.push("/users");
    };

    let on_edit = {
        let nav = navigator.clone();
        let u = user_opt.clone();
        move |_| {
            if let Some(ref user) = u {
                nav.push(format!("/users/{}/edit", user.id));
            }
        }
    };

    let on_reset_password = {
        let api = api.clone();
        let mut toast = toast.clone();
        let u = user_opt.clone();
        let mut show_reset_modal = use_signal(|| false);
        move |_| {
            if let Some(ref user) = u {
                let api = api.clone();
                let mut toast = toast.clone();
                let uid = user.id;
                spawn(async move {
                    let client = api.read().clone();
                    match client.reset_user_password(uid, "temp123").await {
                        Ok(_) => toast.success("Password Reset", "Password has been reset to 'temp123'. User should change it on next login."),
                        Err(e) => toast.error("Error", &e),
                    }
                });
            }
        }
    };

    let on_toggle_status = {
        let api = api.clone();
        let mut toast = toast.clone();
        let u = user_opt.clone();
        move |_| {
            if let Some(ref user) = u {
                let api = api.clone();
                let mut toast = toast.clone();
                let uid = user.id;
                let current = user.status.clone();
                spawn(async move {
                    let client = api.read().clone();
                    match client.toggle_user_status(uid).await {
                        Ok(_) => {
                            if current == "Disabled" {
                                toast.success("User Enabled", "User account has been re-enabled.");
                            } else {
                                toast.warning("User Disabled", "User account has been disabled.");
                            }
                        }
                        Err(e) => toast.error("Error", &e),
                    }
                });
            }
        }
    };

    let on_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_delete = {
        let api = api.clone();
        let mut modal = show_delete_modal.clone();
        let nav = navigator.clone();
        let mut toast = toast.clone();
        let u = user_opt.clone();
        move |_| {
            modal.set(false);
            if let Some(ref user) = u {
                let api = api.clone();
                let mut toast = toast.clone();
                let mut nav = nav.clone();
                let uid = user.id;
                spawn(async move {
                    let client = api.read().clone();
                    match client.delete_user(uid).await {
                        Ok(_) => {
                            toast.success("User Deleted", "User has been permanently removed.");
                            nav.push("/users");
                        }
                        Err(e) => toast.error("Error", &e),
                    }
                });
            }
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "user-detail-loading",
                div { class: "loading-spinner" }
                span { "Loading user details…" }
            }
        };
    }

    if user_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "user-detail-loading",
                div { style: "font-size: 40px;", "👤" }
                h2 { style: "margin: 0; color: var(--text-primary);", "User Not Found" }
                p { "No user with ID \"{id}\" was found." }
                Button { variant: ButtonVariant::Primary, onclick: on_back, "← Back to Users" }
            }
        };
    }

    let user = user_opt.as_ref().unwrap();

    let status_class = match user.status.as_str() {
        "Active" => "user-status-active",
        "Inactive" => "user-status-inactive",
        _ => "user-status-disabled",
    };

    let status_icon = match user.status.as_str() {
        "Active" => "✓",
        "Inactive" => "○",
        _ => "✗",
    };

    // Mock login count
    let login_count = match user.id {
        1 => 1245, 2 => 892, 3 => 567, 4 => 423, 5 => 345,
        6 => 89, 7 => 234, 8 => 45, 9 => 167, 10 => 98,
        11 => 56, 12 => 78, _ => 0,
    };

    let toggle_label = if user.status == "Disabled" { "Enable User" } else { "Disable User" };
    let toggle_icon = if user.status == "Disabled" { "✅" } else { "⛔" };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page user-detail-page",

            // ── Header ──
            div { class: "user-detail-header",
                div { class: "user-detail-title-group",
                    button { class: "user-detail-back", r#type: "button", onclick: on_back, "← Back to Users" }
                    div { class: "user-detail-title-row",
                        h1 { "{user.full_name}" }
                        span { class: "user-detail-username", "@{user.username}" }
                        span { class: "user-status-badge {status_class}", "{status_icon} {user.status}" }
                    }
                }
            }

            // ── KPI Cards ──
            div { class: "user-detail-kpis",
                StatCard {
                    title: "Login Count".to_string(),
                    value: format!("{}", login_count),
                    variant: StatCardVariant::Primary,
                    icon: Some("🔑".to_string()),
                    footer: Some("Total successful logins".to_string()),
                }
                StatCard {
                    title: "Last Login".to_string(),
                    value: user.last_login.clone(),
                    variant: StatCardVariant::Primary,
                    icon: Some("🕐".to_string()),
                    footer: Some("Most recent session".to_string()),
                }
            }

            // ── User Info Section ──
            div { class: "user-detail-section",
                h2 { "User Information" }
                div { class: "user-detail-info-grid",
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Username" }
                        span { class: "user-detail-field-value", "{user.username}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Full Name" }
                        span { class: "user-detail-field-value", "{user.full_name}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Email" }
                        span { class: "user-detail-field-value", "{user.email}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Role" }
                        span { class: "user-detail-field-value", "{user.role}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Status" }
                        span { class: "user-detail-field-value", "{user.status}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Last Login" }
                        span { class: "user-detail-field-value", "{user.last_login}" }
                    }
                    div { class: "user-detail-field",
                        span { class: "user-detail-field-label", "Created At" }
                        span { class: "user-detail-field-value", "{user.created_at}" }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "user-detail-actions",
                div { class: "user-detail-actions-left",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: on_edit,
                        icon: Some("✏️".to_string()),
                        "Edit"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: on_reset_password,
                        icon: Some("🔑".to_string()),
                        "Reset Password"
                    }
                    Button {
                        variant: ButtonVariant::Warning,
                        onclick: on_toggle_status,
                        icon: Some(toggle_icon.to_string()),
                        "{toggle_label}"
                    }
                }
                div { class: "user-detail-actions-right",
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: on_delete,
                        icon: Some("🗑️".to_string()),
                        "Delete"
                    }
                }
            }

            // ── Delete Confirmation Modal ──
            Modal {
                is_open: show_delete_modal,
                title: Some("Delete User".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete User" }
                },
                p { style: "margin: 0; color: var(--text-primary); font-size: 14px;",
                    "Are you sure you want to permanently delete {user.full_name} ({user.username})? This action cannot be undone."
                }
            }
        }
    }
}
