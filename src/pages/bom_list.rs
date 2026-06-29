//! BOM List Page — DataGrid-backed list view for Bills of Materials.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct BomItem {
    pub id: i64,
    pub bom_code: String,
    pub item_name: String,
    pub item_code: String,
    pub total_quantity: f64,
    pub total_cost: f64,
    pub status: String,
    pub version: String,
    pub last_updated: String,
}

#[component]
pub fn BomListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let boms_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            client.list_boms().await
                .map(|server_boms| {
                    server_boms.into_iter().map(|b| BomItem {
                        id: b.id,
                        bom_code: b.bom_no,
                        item_name: b.finished_item_name.unwrap_or_default(),
                        item_code: b.finished_item_code.unwrap_or_default(),
                        total_quantity: b.quantity,
                        total_cost: 0.0, // ponytail: not in list endpoint
                        status: if b.is_active { "Active".to_string() } else { "Inactive".to_string() },
                        version: "1.0".to_string(), // ponytail: not in list endpoint
                        last_updated: b.updated_at,
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = boms_resource.read().is_none();
    let boms = boms_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let columns: Vec<ColumnDef<BomItem>> = vec![
        ColumnDef::text("code", "BOM Code", |b: &BomItem| b.bom_code.clone())
            .with_width(ColumnWidth::Px(130))
            .with_filter(FilterType::Text),
        ColumnDef::text("item", "Item", |b: &BomItem| {
            format!("{} - {}", b.item_code, b.item_name)
        })
        .with_width(ColumnWidth::Fr(1.0))
        .with_filter(FilterType::Text),
        ColumnDef::text("version", "Version", |b: &BomItem| b.version.clone())
            .with_width(ColumnWidth::Px(90)),
        ColumnDef::text("qty", "Qty Produced", |b: &BomItem| b.total_quantity.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Number {
                prefix: "",
                decimals: 2,
            }),
        ColumnDef::text("cost", "Total Cost", |b: &BomItem| b.total_cost.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency {
                code: "PKR",
                decimals: 2,
            }),
        ColumnDef::text("status", "Status", |b: &BomItem| b.status.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Active", BadgeColor::Green),
                    ("Inactive", BadgeColor::Gray),
                    ("Draft", BadgeColor::Yellow),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Active".to_string(), "Inactive".to_string(), "Draft".to_string()],
            }),
        ColumnDef::text("updated", "Last Updated", |b: &BomItem| b.last_updated.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Date {
                format: "%d-%b-%Y",
            })
            .with_filter(FilterType::Date),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, b): (usize, BomItem)| {
            nav.push(format!("/manufacturing/boms/{}", b.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/manufacturing/boms/new");
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
                    h1 { "Bill of Materials" }
                    p { class: "page-subtitle", "Manage product structures and component lists for manufacturing." }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New BOM" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: boms.clone(),
                pagination: PaginationMode::Client { page_size: 15 },
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
