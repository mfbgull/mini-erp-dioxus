//! User List Page — DataGrid-backed list view for system users.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Data Model
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub status: String, // "Active" | "Inactive" | "Disabled"
    pub last_login: String,
    pub created_at: String,
}

// ============================================================================
// Sample Data
// ============================================================================


// ============================================================================
// Summary
// ============================================================================

struct UserSummary {
    total: usize,
    active: usize,
    inactive: usize,
    disabled: usize,
}

fn compute_summary(users: &[User]) -> UserSummary {
    let total = users.len();
    let mut active = 0;
    let mut inactive = 0;
    let mut disabled = 0;
    for u in users {
        match u.status.as_str() {
            "Active" => active += 1,
            "Inactive" => inactive += 1,
            "Disabled" => disabled += 1,
            _ => {}
        }
    }
    UserSummary { total, active, inactive, disabled }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn UserListPage() -> Element {
    let navigator = use_navigator();

    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let users_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            client.list_users().await
                .map(|server_users| {
                    server_users.into_iter().map(|u| User {
                        id: u.id,
                        username: u.username,
                        full_name: u.full_name,
                        email: u.email,
                        role: u.role,
                        status: if u.is_active { "Active".to_string() } else { "Inactive".to_string() },
                        last_login: u.last_login.unwrap_or_default(),
                        created_at: u.created_at.unwrap_or_default(),
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = users_resource.read().is_none();
    let users = users_resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&users);

    let columns: Vec<ColumnDef<User>> = vec![
        ColumnDef::text("username", "Username", |u: &User| u.username.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("full_name", "Full Name", |u: &User| u.full_name.clone())
            .with_width(ColumnWidth::Fr(1.2))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("email", "Email", |u: &User| u.email.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("role", "Role", |u: &User| u.role.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Select {
                options: vec!["Admin".to_string(), "Manager".to_string(), "Sales".to_string(), "Accounts".to_string(), "Inventory".to_string(), "Production".to_string()],
            }),
        ColumnDef::text("status", "Status", |u: &User| u.status.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Active", BadgeColor::Green),
                    ("Inactive", BadgeColor::Yellow),
                    ("Disabled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Active".to_string(), "Inactive".to_string(), "Disabled".to_string()],
            }),
        ColumnDef::text("last_login", "Last Login", |u: &User| u.last_login.clone())
            .with_width(ColumnWidth::Px(160))
            .with_renderer(CellRenderer::DateTime { format: "%d-%b-%Y %H:%M" }),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, u): (usize, User)| {
            nav.push(format!("/users/{}", u.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/users/new");
        }
    };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| { counter += 1; }
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Users" }
                    p { class: "page-subtitle", "Manage system users, roles, and account access." }
                }
            }

            div { class: "customer-summary-bar",
                div { class: "summary-item",
                    span { class: "summary-label", "Total Users" }
                    span { class: "summary-value", "{summary.total}" }
                }
                div { class: "summary-item summary-ok",
                    span { class: "summary-label", "Active" }
                    span { class: "summary-value", "{summary.active}" }
                }
                div { class: "summary-item summary-warning",
                    span { class: "summary-label", "Inactive" }
                    span { class: "summary-value", "{summary.inactive}" }
                }
                div { class: "summary-item summary-danger",
                    span { class: "summary-label", "Disabled" }
                    span { class: "summary-value", "{summary.disabled}" }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New User" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: users.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 8,
            }
        }
    }
}
