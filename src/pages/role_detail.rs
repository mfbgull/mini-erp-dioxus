//! Role Detail Page — Detail view for a system role with permission matrix
//! and user list.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::pages::role_list::Role;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.role-detail-page { max-width: 900px; margin: 0 auto; }
.role-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 20px; gap: 16px; flex-wrap: wrap; }
.role-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.role-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.role-detail-back:hover { text-decoration: underline; }
.role-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.role-detail-title-row h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.role-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.role-detail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.role-detail-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); display: flex; align-items: center; gap: 8px; }
.role-detail-section h2 span { font-size: 11px; font-weight: 400; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 10px; }
.role-detail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.role-detail-field { display: flex; flex-direction: column; gap: 3px; }
.role-detail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.role-detail-field-value { font-size: 14px; color: var(--text-primary); }
.role-detail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.role-detail-actions-left, .role-detail-actions-right { display: flex; align-items: center; gap: 8px; }
.role-perm-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.role-perm-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.role-perm-table thead th.centered { text-align: center; }
.role-perm-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.role-perm-table tbody td.centered { text-align: center; }
.role-perm-table tbody tr:hover { background: rgba(74,144,217,0.03); }
.perm-check { font-weight: 700; font-size: 14px; }
.perm-allowed { color: #28a745; }
.perm-denied { color: #ccc; }
.role-user-table { width: 100%; border-collapse: collapse; font-size: 13px; margin-top: 8px; }
.role-user-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.role-user-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.role-user-table tbody tr:hover { background: rgba(74,144,217,0.03); }
.role-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.role-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: role-spin 0.8s linear infinite; }
@keyframes role-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .role-detail-header { flex-direction: column; } .role-detail-kpis { grid-template-columns: 1fr 1fr; } .role-detail-info-grid { grid-template-columns: 1fr; } .role-detail-actions { flex-direction: column; align-items: stretch; } }
"##;

// ============================================================================
// Permission Model
// ============================================================================

#[derive(Clone, Debug)]
struct ModulePermission {
    module: &'static str,
    view: bool,
    edit: bool,
    create: bool,
    delete: bool,
}

fn role_permissions(role_name: &str) -> Vec<ModulePermission> {
    match role_name {
        "Admin" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Inventory", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Sales", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Purchasing", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Manufacturing", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Accounting", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Reports", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Settings", view: true, edit: true, create: true, delete: true },
        ],
        "Manager" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Sales", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Purchasing", view: true, edit: true, create: true, delete: true },
            ModulePermission { module: "Manufacturing", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Accounting", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: true, delete: false },
            ModulePermission { module: "Settings", view: true, edit: false, create: false, delete: false },
        ],
        "Sales" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Sales", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Purchasing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Manufacturing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Accounting", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Settings", view: false, edit: false, create: false, delete: false },
        ],
        "Accounts" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Sales", view: true, edit: true, create: false, delete: false },
            ModulePermission { module: "Purchasing", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Manufacturing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Accounting", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: true, delete: false },
            ModulePermission { module: "Settings", view: false, edit: false, create: false, delete: false },
        ],
        "Inventory" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Sales", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Purchasing", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Manufacturing", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Accounting", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Settings", view: false, edit: false, create: false, delete: false },
        ],
        "Production" => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Sales", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Purchasing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Manufacturing", view: true, edit: true, create: true, delete: false },
            ModulePermission { module: "Accounting", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Settings", view: false, edit: false, create: false, delete: false },
        ],
        _ => vec![
            ModulePermission { module: "Dashboard", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Inventory", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Sales", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Purchasing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Manufacturing", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Accounting", view: false, edit: false, create: false, delete: false },
            ModulePermission { module: "Reports", view: true, edit: false, create: false, delete: false },
            ModulePermission { module: "Settings", view: false, edit: false, create: false, delete: false },
        ],
    }
}

// ============================================================================
// User in role helper
// ============================================================================

fn users_in_role(role: &str) -> Vec<(&'static str, &'static str)> {
    match role {
        "Admin" => vec![("admin", "Administrator"), ("zainab.akhtar", "Zainab Akhtar")],
        "Manager" => vec![("ahmad.khan", "Ahmad Khan"), ("hira.pervaiz", "Hira Pervaiz")],
        "Sales" => vec![("fatima.ali", "Fatima Ali"), ("bilal.hussain", "Bilal Hussain"), ("raheel.butt", "Raheel Butt")],
        "Accounts" => vec![("sana.raza", "Sana Raza"), ("kamran.khan", "Kamran Khan")],
        "Inventory" => vec![("usman.siddiqui", "Usman Siddiqui"), ("noor.sheikh", "Noor Sheikh")],
        "Production" => vec![("tariq.mehmood", "Tariq Mehmood")],
        _ => vec![],
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn RoleDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    // ── Async fetch ──
    let id_clone = id.clone();
    let role_resource = use_resource(move || {
        let id_fetch = id_clone.clone();
        async move {
            crate::utils::sleep(std::time::Duration::from_millis(500)).await;
            let roles = crate::pages::role_list::sample_roles_data();
            let parsed = id_fetch.parse::<i64>().unwrap_or(0);
            roles.into_iter().find(|r| r.id == parsed)
        }
    });

    let is_loading = role_resource.read().is_none();
    let role_opt = role_resource.read().as_ref().cloned().flatten();

    // ── Modal state ──
    let mut show_delete_modal = use_signal(|| false);

    // ── Handlers ──
    let on_back = move |_: Event<MouseData>| { navigator.push("/roles"); };

    let on_edit = {
        let mut toast = toast.clone();
        move |_| { toast.info("Edit Role", "Role editing is not yet available."); }
    };

    let on_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_delete = {
        let mut modal = show_delete_modal.clone();
        let nav = navigator.clone();
        let mut toast = toast.clone();
        move |_| {
            modal.set(false);
            toast.success("Role Deleted", "Role has been permanently removed.");
            nav.push("/roles");
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "role-loading",
                div { class: "loading-spinner" }
                span { "Loading role details…" }
            }
        };
    }

    if role_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "role-loading",
                div { style: "font-size: 40px;", "🔐" }
                h2 { style: "margin: 0; color: var(--text-primary);", "Role Not Found" }
                p { "No role with ID \"{id}\" was found." }
                Button { variant: ButtonVariant::Primary, onclick: on_back, "← Back to Roles" }
            }
        };
    }

    let role = role_opt.as_ref().unwrap();
    let permissions = role_permissions(&role.role_name);
    let users = users_in_role(&role.role_name);
    let can_edit = !role.is_system;
    let is_system_str = if role.is_system { "System Role" } else { "Custom Role" };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page role-detail-page",

            // ── Header ──
            div { class: "role-detail-header",
                div { class: "role-detail-title-group",
                    button { class: "role-detail-back", r#type: "button", onclick: on_back, "← Back to Roles" }
                    div { class: "role-detail-title-row",
                        h1 { "{role.role_name}" }
                    }
                }
            }

            // ── KPI Cards ──
            div { class: "role-detail-kpis",
                StatCard {
                    title: "Users Assigned".to_string(),
                    value: format!("{}", role.user_count),
                    variant: if role.user_count == 0 { StatCardVariant::Warning } else { StatCardVariant::Primary },
                    icon: Some("👤".to_string()),
                    footer: Some(if role.user_count == 0 { "No users assigned yet".to_string() } else { "Active assignments".to_string() }),
                }
                StatCard {
                    title: "Type".to_string(),
                    value: is_system_str.to_string(),
                    variant: if role.is_system { StatCardVariant::Primary } else { StatCardVariant::Success },
                    icon: Some(if role.is_system { "🔒".to_string() } else { "🔓".to_string() }),
                    footer: Some(if role.is_system { "Built-in, cannot be deleted".to_string() } else { "User-defined role".to_string() }),
                }
            }

            // ── Info Section ──
            div { class: "role-detail-section",
                h2 { "Role Information" }
                div { class: "role-detail-info-grid",
                    div { class: "role-detail-field",
                        span { class: "role-detail-field-label", "Role Name" }
                        span { class: "role-detail-field-value", "{role.role_name}" }
                    }
                    div { class: "role-detail-field",
                        span { class: "role-detail-field-label", "Description" }
                        span { class: "role-detail-field-value", "{role.description}" }
                    }
                    div { class: "role-detail-field",
                        span { class: "role-detail-field-label", "Type" }
                        span { class: "role-detail-field-value", "{is_system_str}" }
                    }
                    div { class: "role-detail-field",
                        span { class: "role-detail-field-label", "User Count" }
                        span { class: "role-detail-field-value", "{role.user_count}" }
                    }
                    div { class: "role-detail-field",
                        span { class: "role-detail-field-label", "Created At" }
                        span { class: "role-detail-field-value", "{role.created_at}" }
                    }
                }
            }

            // ── Permissions Section ──
            div { class: "role-detail-section",
                h2 { "Module Permissions " span { "View / Edit / Create / Delete" } }
                table { class: "role-perm-table",
                    thead {
                        tr {
                            th { "Module" }
                            th { class: "centered", "View" }
                            th { class: "centered", "Edit" }
                            th { class: "centered", "Create" }
                            th { class: "centered", "Delete" }
                        }
                    }
                    tbody {
                        {permissions.iter().map(|p| {
                            let view_class = if p.view { "perm-check perm-allowed" } else { "perm-check perm-denied" };
                            let edit_class = if p.edit { "perm-check perm-allowed" } else { "perm-check perm-denied" };
                            let create_class = if p.create { "perm-check perm-allowed" } else { "perm-check perm-denied" };
                            let delete_class = if p.delete { "perm-check perm-allowed" } else { "perm-check perm-denied" };
                            rsx! {
                                tr {
                                    td { "{p.module}" }
                                    td { class: "centered", span { class: "{view_class}", if p.view { "✓" } else { "✗" } } }
                                    td { class: "centered", span { class: "{edit_class}", if p.edit { "✓" } else { "✗" } } }
                                    td { class: "centered", span { class: "{create_class}", if p.create { "✓" } else { "✗" } } }
                                    td { class: "centered", span { class: "{delete_class}", if p.delete { "✓" } else { "✗" } } }
                                }
                            }
                        })}
                    }
                }
            }

            // ── Users in this Role ──
            div { class: "role-detail-section",
                h2 { "Users with this Role " span { "{users.len()} user(s)" } }
                if users.is_empty() {
                    p { style: "color: var(--text-secondary); font-size: 13px; text-align: center; padding: 16px;",
                        "No users are currently assigned to this role."
                    }
                } else {
                    table { class: "role-user-table",
                        thead {
                            tr {
                                th { "Username" }
                                th { "Full Name" }
                            }
                        }
                        tbody {
                            {users.iter().map(|(username, full_name)| {
                                rsx! {
                                    tr {
                                        td { "{username}" }
                                        td { "{full_name}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "role-detail-actions",
                div { class: "role-detail-actions-left",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: on_edit,
                        disabled: !can_edit,
                        icon: Some("✏️".to_string()),
                        if can_edit { "Edit" } else { "System Role" }
                    }
                }
                div { class: "role-detail-actions-right",
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: on_delete,
                        disabled: role.is_system,
                        icon: Some("🗑️".to_string()),
                        "Delete"
                    }
                }
            }

            // ── Delete Confirmation Modal ──
            Modal {
                is_open: show_delete_modal,
                title: Some("Delete Role".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete Role" }
                },
                p { style: "margin: 0; color: var(--text-primary); font-size: 14px;",
                    "Are you sure you want to delete the \"{role.role_name}\" role? Users with this role will lose associated permissions."
                }
            }
        }
    }
}
