//! User List Page — DataGrid-backed list view for system users.

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

async fn fetch_users() -> Vec<User> {
    crate::utils::sleep(std::time::Duration::from_millis(800)).await;
    sample_users_data()
}

pub fn sample_users_data() -> Vec<User> {
    vec![
        User { id: 1, username: "admin".to_string(), full_name: "Administrator".to_string(), email: "admin@minierp.pk".to_string(), role: "Admin".to_string(), status: "Active".to_string(), last_login: "2026-06-27 08:30:00".to_string(), created_at: "2025-01-01".to_string() },
        User { id: 2, username: "ahmad.khan".to_string(), full_name: "Ahmad Khan".to_string(), email: "ahmad.khan@minierp.pk".to_string(), role: "Manager".to_string(), status: "Active".to_string(), last_login: "2026-06-27 09:15:00".to_string(), created_at: "2025-03-10".to_string() },
        User { id: 3, username: "fatima.ali".to_string(), full_name: "Fatima Ali".to_string(), email: "fatima.ali@minierp.pk".to_string(), role: "Sales".to_string(), status: "Active".to_string(), last_login: "2026-06-26 14:45:00".to_string(), created_at: "2025-03-15".to_string() },
        User { id: 4, username: "usman.siddiqui".to_string(), full_name: "Usman Siddiqui".to_string(), email: "usman.siddiqui@minierp.pk".to_string(), role: "Inventory".to_string(), status: "Active".to_string(), last_login: "2026-06-25 11:20:00".to_string(), created_at: "2025-04-01".to_string() },
        User { id: 5, username: "sana.raza".to_string(), full_name: "Sana Raza".to_string(), email: "sana.raza@minierp.pk".to_string(), role: "Accounts".to_string(), status: "Active".to_string(), last_login: "2026-06-26 16:10:00".to_string(), created_at: "2025-04-10".to_string() },
        User { id: 6, username: "bilal.hussain".to_string(), full_name: "Bilal Hussain".to_string(), email: "bilal.hussain@minierp.pk".to_string(), role: "Sales".to_string(), status: "Inactive".to_string(), last_login: "2026-05-30 10:00:00".to_string(), created_at: "2025-05-05".to_string() },
        User { id: 7, username: "hira.pervaiz".to_string(), full_name: "Hira Pervaiz".to_string(), email: "hira.pervaiz@minierp.pk".to_string(), role: "Manager".to_string(), status: "Active".to_string(), last_login: "2026-06-27 07:55:00".to_string(), created_at: "2025-05-20".to_string() },
        User { id: 8, username: "tariq.mehmood".to_string(), full_name: "Tariq Mehmood".to_string(), email: "tariq.mehmood@minierp.pk".to_string(), role: "Production".to_string(), status: "Disabled".to_string(), last_login: "2026-04-15 08:30:00".to_string(), created_at: "2025-06-01".to_string() },
        User { id: 9, username: "zainab.akhtar".to_string(), full_name: "Zainab Akhtar".to_string(), email: "zainab.akhtar@minierp.pk".to_string(), role: "Admin".to_string(), status: "Active".to_string(), last_login: "2026-06-26 18:00:00".to_string(), created_at: "2025-07-12".to_string() },
        User { id: 10, username: "kamran.khan".to_string(), full_name: "Kamran Khan".to_string(), email: "kamran.khan@minierp.pk".to_string(), role: "Accounts".to_string(), status: "Active".to_string(), last_login: "2026-06-25 13:40:00".to_string(), created_at: "2025-08-01".to_string() },
        User { id: 11, username: "noor.sheikh".to_string(), full_name: "Noor Sheikh".to_string(), email: "noor.sheikh@minierp.pk".to_string(), role: "Inventory".to_string(), status: "Inactive".to_string(), last_login: "2026-05-20 09:15:00".to_string(), created_at: "2025-09-05".to_string() },
        User { id: 12, username: "raheel.butt".to_string(), full_name: "Raheel Butt".to_string(), email: "raheel.butt@minierp.pk".to_string(), role: "Sales".to_string(), status: "Active".to_string(), last_login: "2026-06-27 10:05:00".to_string(), created_at: "2025-10-01".to_string() },
    ]
}

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

    let refresh_counter = use_signal(|| 0u32);
    let users_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_users().await
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
            nav.push("/users");
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
