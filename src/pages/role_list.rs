//! Role List Page — DataGrid-backed list view for system roles/permissions.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType, Modal, ModalSize, use_toast};
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Data Model
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct Role {
    pub id: i64,
    pub role_name: String,
    pub description: String,
    pub user_count: i32,
    pub is_system: bool,
    pub created_at: String,
}

// ============================================================================
// Sample Data
// ============================================================================


// ============================================================================
// Summary
// ============================================================================

struct RoleSummary {
    total: usize,
    system: usize,
    custom: usize,
    total_users: i32,
}

fn compute_summary(roles: &[Role]) -> RoleSummary {
    let total = roles.len();
    let system = roles.iter().filter(|r| r.is_system).count();
    let custom = total - system;
    let total_users = roles.iter().map(|r| r.user_count).sum();
    RoleSummary { total, system, custom, total_users }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn RoleListPage() -> Element {
    let navigator = use_navigator();

    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let roles_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            client.list_roles().await
                .map(|server_roles| {
                    server_roles.into_iter().map(|r| Role {
                        id: r.id,
                        role_name: r.role_name,
                        description: r.description,
                        user_count: r.user_count as i32,
                        is_system: r.is_system_role,
                        created_at: String::new(),
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = roles_resource.read().is_none();
    let roles = roles_resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&roles);

    let columns: Vec<ColumnDef<Role>> = vec![
        ColumnDef::text("name", "Role Name", |r: &Role| r.role_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("description", "Description", |r: &Role| r.description.clone())
            .with_width(ColumnWidth::Fr(1.5)),
        ColumnDef::text("users", "Users", |r: &Role| r.user_count.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
        ColumnDef::text("system", "System", |r: &Role| {
            if r.is_system { "System" } else { "Custom" }
        }.to_string())
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("System", BadgeColor::Blue),
                    ("Custom", BadgeColor::Green),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["System".to_string(), "Custom".to_string()],
            }),
        ColumnDef::text("created", "Created At", |r: &Role| r.created_at.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" }),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, r): (usize, Role)| {
            nav.push(format!("/roles/{}", r.id));
        }
    };

    // ── Create role modal state ──
    let mut show_create_modal = use_signal(|| false);
    let mut new_role_name = use_signal(|| String::new());
    let mut new_role_desc = use_signal(|| String::new());
    let mut is_creating = use_signal(|| false);
    let mut name_error = use_signal(|| String::new());
    let toast = use_toast();

    let on_new = {
        let mut modal = show_create_modal.clone();
        move |_| { modal.set(true); }
    };
    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| { counter += 1; }
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Roles & Permissions" }
                    p { class: "page-subtitle", "Manage system roles and assign permissions." }
                }
            }

            div { class: "customer-summary-bar",
                div { class: "summary-item",
                    span { class: "summary-label", "Total Roles" }
                    span { class: "summary-value", "{summary.total}" }
                }
                div { class: "summary-item summary-ok",
                    span { class: "summary-label", "System" }
                    span { class: "summary-value", "{summary.system}" }
                }
                div { class: "summary-item",
                    span { class: "summary-label", "Custom" }
                    span { class: "summary-value", "{summary.custom}" }
                }
                div { class: "summary-item",
                    span { class: "summary-label", "Total Users Covered" }
                    span { class: "summary-value", "{summary.total_users}" }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Role" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: roles.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Single,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 5,
            }

            // ── Create Role Modal ──
            Modal {
                is_open: show_create_modal,
                title: Some("Create New Role".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            show_create_modal.set(false);
                            new_role_name.set(String::new());
                            new_role_desc.set(String::new());
                            name_error.set(String::new());
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: *is_creating.read(),
                        onclick: {
                            let modal = show_create_modal.clone();
                            let name = new_role_name.clone();
                            let desc = new_role_desc.clone();
                            let mut creating = is_creating.clone();
                            let mut err = name_error.clone();
                            let refresh = refresh_counter.clone();
                            let api_clone = api.clone();
                            let toast = toast.clone();
                            move |_| {
                                let name_val = name.read().trim().to_string();
                                if name_val.is_empty() {
                                    err.set("Role name is required.".to_string());
                                    return;
                                }
                                err.set(String::new());
                                creating.set(true);
                                let body = serde_json::json!({
                                    "role_name": name_val,
                                    "description": if desc.read().trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(desc.read().trim().to_string()) },
                                });
                                let api2 = api_clone.clone();
                                let mut modal2 = modal.clone();
                                let mut name2 = name.clone();
                                let mut desc2 = desc.clone();
                                let mut creating2 = creating.clone();
                                let mut refresh2 = refresh.clone();
                                let mut toast2 = toast.clone();
                                spawn(async move {
                                    match api2.with(|c| c.clone()).create_role(&body).await {
                                        Ok(_) => {
                                            toast2.success("Role Created", "New role has been created successfully.");
                                            modal2.set(false);
                                            name2.set(String::new());
                                            desc2.set(String::new());
                                            creating2.set(false);
                                            refresh2 += 1;
                                        }
                                        Err(e) => {
                                            toast2.error("Create Failed", &e);
                                            creating2.set(false);
                                        }
                                    }
                                });
                            }
                        },
                        if *is_creating.read() { "Creating…" } else { "Create Role" }
                    }
                },
                div { style: "display: flex; flex-direction: column; gap: 16px;",
                    FormInput {
                        label: "Role Name".to_string(),
                        value: new_role_name.read().clone(),
                        required: true,
                        placeholder: Some("e.g. Quality Control".to_string()),
                        error: if name_error.read().is_empty() { None } else { Some(name_error.read().clone()) },
                        oninput: move |v: String| { new_role_name.set(v); name_error.set(String::new()); },
                    }
                    FormInput {
                        label: "Description".to_string(),
                        value: new_role_desc.read().clone(),
                        r#type: InputType::TextArea,
                        placeholder: Some("Brief description of this role's responsibilities.".to_string()),
                        oninput: move |v: String| { new_role_desc.set(v); },
                    }
                }
            }
        }
    }
}
