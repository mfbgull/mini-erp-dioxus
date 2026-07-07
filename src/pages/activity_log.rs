//! Activity Log Page — DataGrid-backed audit trail with module and action filters.

use crate::auth::use_auth;
use crate::components::common::use_toast;
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
pub struct ActivityEntry {
    pub id: i64,
    pub timestamp: String,
    pub user: String,
    pub action: String,      // "Create" | "Update" | "Delete" | "Login"
    pub module: String,      // "Inventory" | "Sales" | "Purchasing" | etc.
    pub description: String,
    pub ip_address: String,
}

// ============================================================================
// Sample Data
// ============================================================================


// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ActivityLogPage() -> Element {
    let _toast = use_toast();

    // ── Async data ──
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let activity_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            client.list_activity_logs().await
                .map(|logs| {
                    logs.into_iter().map(|l| ActivityEntry {
                        id: l.id,
                        timestamp: l.created_at,
                        user: l.username.unwrap_or_default(),
                        action: l.action,
                        module: l.entity_type,
                        description: l.metadata.unwrap_or_default(),
                        ip_address: l.ip_address.unwrap_or_default(),
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = activity_resource.read().is_none();
    let entries = activity_resource.read().cloned().unwrap_or_default();

    // ── Filter state ──
    let module_filter = use_signal(|| String::new());
    let action_filter = use_signal(|| String::new());

    // ── Filtered data ──
    let filtered: Vec<ActivityEntry> = entries.clone().into_iter().filter(|e| {
        let module_match = module_filter.read().is_empty() || e.module == *module_filter.read();
        let action_match = action_filter.read().is_empty() || e.action == *action_filter.read();
        module_match && action_match
    }).collect();

    // ── Column definitions ──
    let columns: Vec<ColumnDef<ActivityEntry>> = vec![
        ColumnDef::text("timestamp", "Timestamp", |e: &ActivityEntry| e.timestamp.clone())
            .with_width(ColumnWidth::Px(160))
            .with_renderer(CellRenderer::DateTime { format: "%d-%b-%Y %H:%M" })
            .with_filter(FilterType::Date),
        ColumnDef::text("user", "User", |e: &ActivityEntry| e.user.clone())
            .with_width(ColumnWidth::Px(130))
            .with_filter(FilterType::Text),
        ColumnDef::text("action", "Action", |e: &ActivityEntry| e.action.clone())
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Create", BadgeColor::Green),
                    ("Update", BadgeColor::Blue),
                    ("Delete", BadgeColor::Red),
                    ("Login", BadgeColor::Yellow),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Create".to_string(), "Update".to_string(), "Delete".to_string(), "Login".to_string()],
            }),
        ColumnDef::text("module", "Module", |e: &ActivityEntry| e.module.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Select {
                options: vec![
                    "System".to_string(), "Inventory".to_string(), "Sales".to_string(), "Purchasing".to_string(),
                    "Manufacturing".to_string(), "Accounting".to_string(), "Settings".to_string(), "Reports".to_string(),
                ],
            }),
        ColumnDef::text("description", "Description", |e: &ActivityEntry| e.description.clone())
            .with_width(ColumnWidth::Fr(1.5))
            .with_filter(FilterType::Text),
        ColumnDef::text("ip", "IP Address", |e: &ActivityEntry| e.ip_address.clone())
            .with_width(ColumnWidth::Px(130)),
    ];

    // ── Handlers ──
    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| { counter += 1; }
    };

    let mut on_module_change = {
        let mut mf = module_filter.clone();
        move |v: String| { mf.set(v); }
    };

    let mut on_action_change = {
        let mut af = action_filter.clone();
        move |v: String| { af.set(v); }
    };

    let module_options = {
        let mut modules: Vec<String> = entries.iter().map(|e| e.module.clone()).collect();
        modules.sort();
        modules.dedup();
        modules
    };

    let action_options = vec!["Create", "Update", "Delete", "Login"];

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Activity Log" }
                    p { class: "page-subtitle", "Audit trail of all system activities, changes, and user logins." }
                }
            }

            // ── Filter Bar ──
            div { style: "display: flex; gap: 12px; align-items: flex-end; margin-bottom: 16px; flex-wrap: wrap;",
                div { style: "min-width: 160px;",
                    label { style: "font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; display: block; margin-bottom: 4px;", "Module" }
                    select {
                        style: "width: 100%; padding: 6px 10px; border: 1px solid var(--border-color,#e0e0e0); border-radius: 6px; font-size: 13px; background: #fff;",
                        value: "{module_filter.read()}",
                        oninput: move |e| on_module_change(e.value()),
                        option { value: "", "All Modules" }
                        {module_options.iter().map(|m| rsx! {
                            option { value: "{m}", "{m}" }
                        })}
                    }
                }
                div { style: "min-width: 140px;",
                    label { style: "font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; display: block; margin-bottom: 4px;", "Action" }
                    select {
                        style: "width: 100%; padding: 6px 10px; border: 1px solid var(--border-color,#e0e0e0); border-radius: 6px; font-size: 13px; background: #fff;",
                        value: "{action_filter.read()}",
                        oninput: move |e| on_action_change(e.value()),
                        option { value: "", "All Actions" }
                        {action_options.iter().map(|a| rsx! {
                            option { value: "{a}", "{a}" }
                        })}
                    }
                }
                button {
                    class: "toolbar-btn",
                    r#type: "button",
                    onclick: on_refresh,
                    "🔄 Refresh"
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: filtered.clone(),
                pagination: PaginationMode::Client { page_size: 15 },
                selection_mode: SelectionMode::Single,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 10,
            }
        }
    }
}
