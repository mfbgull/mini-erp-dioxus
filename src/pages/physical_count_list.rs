//! Physical Count List Page — DataGrid-backed list view for physical inventory counts.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode, RowHeight,
    SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct PhysicalCountItem {
    pub id: i64,
    pub count_no: String,
    pub count_date: String,
    pub warehouse_name: String,
    pub status: String,
    pub notes: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[component]
pub fn PhysicalCountListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let api = use_auth().api;
    let counts_resource: Resource<Vec<PhysicalCountItem>> = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.read().clone();
            client.list_physical_counts().await
                .unwrap_or_default()
                .into_iter()
                .map(|c| PhysicalCountItem {
                    id: c.id,
                    count_no: c.count_no,
                    count_date: c.count_date,
                    warehouse_name: c.warehouse_name.unwrap_or_default(),
                    status: c.status,
                    notes: c.notes,
                    created_at: c.created_at,
                    completed_at: c.completed_at,
                })
                .collect()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = counts_resource.read().is_none();
    let counts: Vec<PhysicalCountItem> = counts_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let columns: Vec<ColumnDef<PhysicalCountItem>> = vec![
        ColumnDef::text("no", "Count No", |c: &PhysicalCountItem| c.count_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("date", "Count Date", |c: &PhysicalCountItem| c.count_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(crate::components::data_grid::CellRenderer::Date {
                format: "%d-%b-%Y",
            })
            .with_filter(FilterType::Date),
        ColumnDef::text("warehouse", "Warehouse", |c: &PhysicalCountItem| {
            c.warehouse_name.clone()
        })
        .with_width(ColumnWidth::Fr(0.8))
        .with_filter(FilterType::Text),
        ColumnDef::text("status", "Status", |c: &PhysicalCountItem| c.status.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(crate::components::data_grid::CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Yellow),
                    ("Completed", BadgeColor::Green),
                    ("Cancelled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Completed".to_string(), "Cancelled".to_string()],
            }),
        ColumnDef::text("notes", "Notes", |c: &PhysicalCountItem| c.notes.clone())
            .with_width(ColumnWidth::Fr(0.6)),
        ColumnDef::text("created", "Created", |c: &PhysicalCountItem| c.created_at.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(crate::components::data_grid::CellRenderer::Date {
                format: "%d-%b-%Y",
            }),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, c): (usize, PhysicalCountItem)| {
            nav.push(format!("/inventory/physical-counts/{}", c.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/inventory/physical-counts/new");
        }
    };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| {
            counter += 1;
        }
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Physical Counts" }
                    p { class: "page-subtitle", "Manage physical inventory counts to verify stock accuracy." }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Count" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: counts.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
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
