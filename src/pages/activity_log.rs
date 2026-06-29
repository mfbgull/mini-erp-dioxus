//! Activity Log Page — DataGrid-backed audit trail with module and action filters.

use crate::components::common::{Button, ButtonVariant, DateRangePicker, use_toast};
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

async fn fetch_activity() -> Vec<ActivityEntry> {
    crate::utils::sleep(std::time::Duration::from_millis(700)).await;
    sample_activity_data()
}

pub fn sample_activity_data() -> Vec<ActivityEntry> {
    vec![
        ActivityEntry { id: 1, timestamp: "2026-06-27 10:32:00".to_string(), user: "ahmad.khan".to_string(), action: "Create".to_string(), module: "Sales".to_string(), description: "Created invoice INV-2026-0045 for Alpha Traders".to_string(), ip_address: "192.168.1.101".to_string() },
        ActivityEntry { id: 2, timestamp: "2026-06-27 10:15:00".to_string(), user: "fatima.ali".to_string(), action: "Update".to_string(), module: "Sales".to_string(), description: "Modified quotation QTN-2026-0023 for Beta Industries".to_string(), ip_address: "192.168.1.102".to_string() },
        ActivityEntry { id: 3, timestamp: "2026-06-27 09:45:00".to_string(), user: "usman.siddiqui".to_string(), action: "Create".to_string(), module: "Inventory".to_string(), description: "Created stock movement IN from supplier PO-034".to_string(), ip_address: "192.168.1.103".to_string() },
        ActivityEntry { id: 4, timestamp: "2026-06-27 09:30:00".to_string(), user: "admin".to_string(), action: "Login".to_string(), module: "System".to_string(), description: "User logged in from office workstation".to_string(), ip_address: "192.168.1.100".to_string() },
        ActivityEntry { id: 5, timestamp: "2026-06-27 08:55:00".to_string(), user: "sana.raza".to_string(), action: "Create".to_string(), module: "Accounting".to_string(), description: "Recorded payment INV-2026-0038 — PKR 125,400.00".to_string(), ip_address: "192.168.1.104".to_string() },
        ActivityEntry { id: 6, timestamp: "2026-06-26 17:20:00".to_string(), user: "ahmad.khan".to_string(), action: "Delete".to_string(), module: "Inventory".to_string(), description: "Deleted expired physical count PC-2026-0003".to_string(), ip_address: "192.168.1.101".to_string() },
        ActivityEntry { id: 7, timestamp: "2026-06-26 16:45:00".to_string(), user: "fatima.ali".to_string(), action: "Update".to_string(), module: "Sales".to_string(), description: "Marked invoice INV-2026-0040 as Paid".to_string(), ip_address: "192.168.1.102".to_string() },
        ActivityEntry { id: 8, timestamp: "2026-06-26 16:00:00".to_string(), user: "admin".to_string(), action: "Create".to_string(), module: "Settings".to_string(), description: "Added new user raheel.butt with Sales role".to_string(), ip_address: "192.168.1.100".to_string() },
        ActivityEntry { id: 9, timestamp: "2026-06-26 14:30:00".to_string(), user: "tariq.mehmood".to_string(), action: "Update".to_string(), module: "Manufacturing".to_string(), description: "Completed production order PROD-2026-0012".to_string(), ip_address: "192.168.1.105".to_string() },
        ActivityEntry { id: 10, timestamp: "2026-06-26 11:10:00".to_string(), user: "zainab.akhtar".to_string(), action: "Login".to_string(), module: "System".to_string(), description: "User logged in remotely".to_string(), ip_address: "203.0.113.45".to_string() },
        ActivityEntry { id: 11, timestamp: "2026-06-26 10:05:00".to_string(), user: "usman.siddiqui".to_string(), action: "Create".to_string(), module: "Purchasing".to_string(), description: "Created purchase order PO-2026-0035 for Raw Materials".to_string(), ip_address: "192.168.1.103".to_string() },
        ActivityEntry { id: 12, timestamp: "2026-06-25 15:30:00".to_string(), user: "sana.raza".to_string(), action: "Update".to_string(), module: "Accounting".to_string(), description: "Adjusted journal entry for depreciation".to_string(), ip_address: "192.168.1.104".to_string() },
        ActivityEntry { id: 13, timestamp: "2026-06-25 14:00:00".to_string(), user: "ahmad.khan".to_string(), action: "Delete".to_string(), module: "Sales".to_string(), description: "Cancelled and deleted quote QTN-2026-0019".to_string(), ip_address: "192.168.1.101".to_string() },
        ActivityEntry { id: 14, timestamp: "2026-06-25 11:20:00".to_string(), user: "kamran.khan".to_string(), action: "Create".to_string(), module: "Accounting".to_string(), description: "Created expense entry EX-2026-0021 — Office Supplies".to_string(), ip_address: "192.168.1.106".to_string() },
        ActivityEntry { id: 15, timestamp: "2026-06-25 09:00:00".to_string(), user: "admin".to_string(), action: "Login".to_string(), module: "System".to_string(), description: "User logged in from console".to_string(), ip_address: "127.0.0.1".to_string() },
        ActivityEntry { id: 16, timestamp: "2026-06-24 16:50:00".to_string(), user: "fatima.ali".to_string(), action: "Create".to_string(), module: "Sales".to_string(), description: "Created sales order SO-2026-0022 for Gamma Supplies".to_string(), ip_address: "192.168.1.102".to_string() },
        ActivityEntry { id: 17, timestamp: "2026-06-24 15:10:00".to_string(), user: "usman.siddiqui".to_string(), action: "Update".to_string(), module: "Inventory".to_string(), description: "Adjusted stock level — ITM-0005 (Rubber Gasket Set)".to_string(), ip_address: "192.168.1.103".to_string() },
        ActivityEntry { id: 18, timestamp: "2026-06-24 13:25:00".to_string(), user: "hira.pervaiz".to_string(), action: "Create".to_string(), module: "Reports".to_string(), description: "Generated monthly sales report for May 2026".to_string(), ip_address: "192.168.1.107".to_string() },
        ActivityEntry { id: 19, timestamp: "2026-06-24 10:00:00".to_string(), user: "noor.sheikh".to_string(), action: "Login".to_string(), module: "System".to_string(), description: "User logged in after password reset".to_string(), ip_address: "192.168.1.108".to_string() },
        ActivityEntry { id: 20, timestamp: "2026-06-23 17:30:00".to_string(), user: "ahmad.khan".to_string(), action: "Update".to_string(), module: "Purchasing".to_string(), description: "Approved purchase order PO-2026-0033".to_string(), ip_address: "192.168.1.101".to_string() },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ActivityLogPage() -> Element {
    let _toast = use_toast();

    // ── Async data ──
    let refresh_counter = use_signal(|| 0u32);
    let activity_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_activity().await
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
