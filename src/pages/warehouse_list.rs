//! Warehouse List Page — DataGrid-backed list view for managing warehouses.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode, RowHeight,
    SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct WarehouseItem {
    pub id: i64,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub location: String,
    pub is_active: bool,
    pub created_at: String,
}

#[component]
pub fn WarehouseListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let warehouses_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            match client.list_warehouses().await {
                Ok(server_whs) => server_whs.into_iter().map(|w| WarehouseItem {
                    id: w.id,
                    warehouse_code: w.warehouse_code,
                    warehouse_name: w.warehouse_name,
                    location: w.location,
                    is_active: w.is_active,
                    created_at: w.created_at,
                }).collect(),
                Err(_) => Vec::new(),
            }
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = warehouses_resource.read().is_none();
    let warehouses = warehouses_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let columns: Vec<ColumnDef<WarehouseItem>> = vec![
        ColumnDef::text("code", "Code", |w: &WarehouseItem| w.warehouse_code.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Text),
        ColumnDef::text("name", "Warehouse Name", |w: &WarehouseItem| w.warehouse_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("location", "Location", |w: &WarehouseItem| w.location.clone())
            .with_width(ColumnWidth::Fr(0.8))
            .with_filter(FilterType::Text),
        ColumnDef::text("status", "Status", |w: &WarehouseItem| {
            if w.is_active { "Active".to_string() } else { "Inactive".to_string() }
        })
        .with_width(ColumnWidth::Px(100))
        .with_renderer(crate::components::data_grid::CellRenderer::Badge {
            color_map: vec![("Active", BadgeColor::Green), ("Inactive", BadgeColor::Gray)],
            default_color: BadgeColor::Gray,
        }),
        ColumnDef::text("created", "Created", |w: &WarehouseItem| w.created_at.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(crate::components::data_grid::CellRenderer::Date {
                format: "%d-%b-%Y",
            }),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, w): (usize, WarehouseItem)| {
            nav.push(format!("/inventory/warehouses/{}", w.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/inventory/warehouses/new");
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
                    h1 { "Warehouses" }
                    p { class: "page-subtitle", "Manage your warehouse locations and stock areas." }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Warehouse" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: warehouses.clone(),
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
