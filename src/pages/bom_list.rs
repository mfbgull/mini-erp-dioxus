//! BOM List Page — DataGrid-backed list view for Bills of Materials.

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

async fn fetch_boms() -> Vec<BomItem> {
    crate::utils::sleep(std::time::Duration::from_millis(500)).await;
    sample_boms()
}

fn sample_boms() -> Vec<BomItem> {
    vec![
        BomItem {
            id: 1,
            bom_code: "BOM-0001".to_string(),
            item_name: "Premium Widget Alpha".to_string(),
            item_code: "ITM-0001".to_string(),
            total_quantity: 1.0,
            total_cost: 2850.0,
            status: "Active".to_string(),
            version: "v1.2".to_string(),
            last_updated: "2026-06-20".to_string(),
        },
        BomItem {
            id: 2,
            bom_code: "BOM-0002".to_string(),
            item_name: "Steel Bracket XR-200".to_string(),
            item_code: "ITM-0004".to_string(),
            total_quantity: 1.0,
            total_cost: 1240.0,
            status: "Active".to_string(),
            version: "v1.0".to_string(),
            last_updated: "2026-06-18".to_string(),
        },
        BomItem {
            id: 3,
            bom_code: "BOM-0003".to_string(),
            item_name: "Rubber Gasket Set".to_string(),
            item_code: "ITM-0005".to_string(),
            total_quantity: 1.0,
            total_cost: 320.0,
            status: "Draft".to_string(),
            version: "v0.9".to_string(),
            last_updated: "2026-06-22".to_string(),
        },
        BomItem {
            id: 4,
            bom_code: "BOM-0004".to_string(),
            item_name: "Assembly Kit Type-B".to_string(),
            item_code: "ITM-0008".to_string(),
            total_quantity: 1.0,
            total_cost: 5670.0,
            status: "Active".to_string(),
            version: "v2.1".to_string(),
            last_updated: "2026-06-15".to_string(),
        },
        BomItem {
            id: 5,
            bom_code: "BOM-0005".to_string(),
            item_name: "Control Panel CX-12".to_string(),
            item_code: "ITM-0012".to_string(),
            total_quantity: 1.0,
            total_cost: 12300.0,
            status: "Inactive".to_string(),
            version: "v1.0".to_string(),
            last_updated: "2026-05-30".to_string(),
        },
        BomItem {
            id: 6,
            bom_code: "BOM-0006".to_string(),
            item_name: "Hydraulic Pump HP-45".to_string(),
            item_code: "ITM-0015".to_string(),
            total_quantity: 1.0,
            total_cost: 8900.0,
            status: "Active".to_string(),
            version: "v1.3".to_string(),
            last_updated: "2026-06-25".to_string(),
        },
    ]
}

#[component]
pub fn BomListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let boms_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_boms().await
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
