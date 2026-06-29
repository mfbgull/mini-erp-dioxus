//! Production Order List Page — DataGrid-backed list view for production orders.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct ProductionOrderItem {
    pub id: i64,
    pub prd_no: String,
    pub item_name: String,
    pub item_code: String,
    pub planned_qty: i32,
    pub completed_qty: i32,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
}

#[component]
pub fn ProductionListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let orders_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            client.list_production_orders().await
                .map(|server_orders| {
                    server_orders.into_iter().map(|o| ProductionOrderItem {
                        id: o.id,
                        prd_no: o.production_no,
                        item_name: o.output_item_name.unwrap_or_default(),
                        item_code: o.output_item_code.unwrap_or_default(),
                        planned_qty: o.output_quantity as i32,
                        completed_qty: 0, // ponytail: not in list endpoint
                        start_date: o.created_at.clone(),
                        end_date: o.created_at, // ponytail: not in list endpoint
                        status: o.status,
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = orders_resource.read().is_none();
    let orders = orders_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let columns: Vec<ColumnDef<ProductionOrderItem>> = vec![
        ColumnDef::text("no", "Production #", |o: &ProductionOrderItem| o.prd_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("item", "Item", |o: &ProductionOrderItem| {
            format!("{} - {}", o.item_code, o.item_name)
        })
        .with_width(ColumnWidth::Fr(1.0))
        .with_filter(FilterType::Text),
        ColumnDef::text("planned", "Planned Qty", |o: &ProductionOrderItem| o.planned_qty.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
        ColumnDef::text("completed", "Completed", |o: &ProductionOrderItem| o.completed_qty.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
        ColumnDef::text("progress", "Progress", |o: &ProductionOrderItem| {
            if o.planned_qty > 0 {
                format!("{:.1}%", (o.completed_qty as f64 / o.planned_qty as f64) * 100.0)
            } else { "0%".to_string() }
        })
        .with_align(TextAlign::Right)
        .with_width(ColumnWidth::Px(100)),
        ColumnDef::text("start", "Start Date", |o: &ProductionOrderItem| o.start_date.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("end", "End Date", |o: &ProductionOrderItem| o.end_date.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |o: &ProductionOrderItem| o.status.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Completed", BadgeColor::Green),
                    ("In Progress", BadgeColor::Blue),
                    ("Planned", BadgeColor::Yellow),
                    ("Cancelled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Planned".to_string(), "In Progress".to_string(), "Completed".to_string(), "Cancelled".to_string()],
            }),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, o): (usize, ProductionOrderItem)| {
            nav.push(format!("/manufacturing/production/{}", o.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/manufacturing/production/new");
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
                    h1 { "Production Orders" }
                    p { class: "page-subtitle", "Manage production runs, track progress, and monitor shop floor activities." }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Production Order" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: orders.clone(),
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
