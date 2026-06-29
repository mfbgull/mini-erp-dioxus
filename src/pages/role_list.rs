//! Role List Page — DataGrid-backed list view for system roles/permissions.

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

async fn fetch_roles() -> Vec<Role> {
    crate::utils::sleep(std::time::Duration::from_millis(600)).await;
    sample_roles_data()
}

pub fn sample_roles_data() -> Vec<Role> {
    vec![
        Role { id: 1, role_name: "Admin".to_string(), description: "Full system access with all permissions.".to_string(), user_count: 2, is_system: true, created_at: "2025-01-01".to_string() },
        Role { id: 2, role_name: "Manager".to_string(), description: "Can manage operations, approve orders, view reports.".to_string(), user_count: 2, is_system: true, created_at: "2025-01-01".to_string() },
        Role { id: 3, role_name: "Sales".to_string(), description: "Create and manage quotations, sales orders, and invoices.".to_string(), user_count: 3, is_system: true, created_at: "2025-01-01".to_string() },
        Role { id: 4, role_name: "Accounts".to_string(), description: "Manage invoices, payments, expenses, and financial reports.".to_string(), user_count: 2, is_system: false, created_at: "2025-03-15".to_string() },
        Role { id: 5, role_name: "Inventory".to_string(), description: "Manage stock movements, physical counts, and warehouse.".to_string(), user_count: 2, is_system: true, created_at: "2025-01-01".to_string() },
        Role { id: 6, role_name: "Production".to_string(), description: "Manage BOMs, production orders, and manufacturing.".to_string(), user_count: 1, is_system: false, created_at: "2025-06-01".to_string() },
        Role { id: 7, role_name: "Purchasing".to_string(), description: "Create and manage purchase orders and direct purchases.".to_string(), user_count: 0, is_system: false, created_at: "2025-07-10".to_string() },
        Role { id: 8, role_name: "Viewer".to_string(), description: "Read-only access to dashboards and reports.".to_string(), user_count: 0, is_system: false, created_at: "2025-09-20".to_string() },
    ]
}

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

    let refresh_counter = use_signal(|| 0u32);
    let roles_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_roles().await
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
