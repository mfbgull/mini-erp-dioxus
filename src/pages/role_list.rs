//! Role List Page — DataGrid-backed list view for system roles/permissions.

use crate::auth::use_auth;
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
                        user_count: 0, // ponytail: not in API
                        is_system: r.is_system_role,
                        created_at: String::new(), // ponytail: not in API
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

    let on_new = move |_| {};
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
        }
    }
}
