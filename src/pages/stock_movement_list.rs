//! Stock Movement List Page — DataGrid-backed list view for tracking stock movements.

use crate::auth::use_auth;
use crate::components::common::{Modal, ModalSize};
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use crate::components::inventory::StockAdjustmentForm;
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct StockMovementItem {
    pub id: i64,
    pub movement_no: String,
    pub item_code: String,
    pub item_name: String,
    pub warehouse_name: String,
    pub movement_type: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub reference_doctype: Option<String>,
    pub reference_docno: Option<String>,
    pub created_at: String,
}



#[component]
pub fn StockMovementListPage() -> Element {
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let show_modal = use_signal(|| false);
    let movements_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        let client = api.read().clone();
        client.list_stock_movements().await
            .unwrap_or_default()
            .into_iter()
            .map(|m| StockMovementItem {
                id: m.id,
                movement_no: m.movement_no,
                item_code: m.item_code.unwrap_or_default(),
                item_name: m.item_name.unwrap_or_default(),
                warehouse_name: m.warehouse_name.unwrap_or_default(),
                movement_type: m.movement_type,
                quantity: m.quantity,
                unit_cost: m.unit_cost,
                reference_doctype: m.reference_doctype,
                reference_docno: m.reference_docno,
                created_at: m.created_at,
            })
            .collect::<Vec<_>>()
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = movements_resource.read().is_none();
    let movements = movements_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let columns: Vec<ColumnDef<StockMovementItem>> = vec![
        ColumnDef::text("no", "Movement No", |m: &StockMovementItem| m.movement_no.clone())
            .with_width(ColumnWidth::Px(150))
            .with_filter(FilterType::Text),
        ColumnDef::text("item", "Item", |m: &StockMovementItem| {
            format!("{} - {}", m.item_code, m.item_name)
        })
        .with_width(ColumnWidth::Fr(1.0))
        .with_filter(FilterType::Text),
        ColumnDef::text("warehouse", "Warehouse", |m: &StockMovementItem| m.warehouse_name.clone())
            .with_width(ColumnWidth::Px(150)),
        ColumnDef::text("type", "Type", |m: &StockMovementItem| m.movement_type.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("IN", BadgeColor::Green),
                    ("OUT", BadgeColor::Red),
                    ("ADJUSTMENT", BadgeColor::Yellow),
                    ("TRANSFER", BadgeColor::Blue),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["IN".to_string(), "OUT".to_string(), "ADJUSTMENT".to_string(), "TRANSFER".to_string()],
            }),
        ColumnDef::text("qty", "Quantity", |m: &StockMovementItem| m.quantity.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Number {
                prefix: "",
                decimals: 0,
            }),
        ColumnDef::text("cost", "Unit Cost", |m: &StockMovementItem| m.unit_cost.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Currency {
                code: "PKR",
                decimals: 2,
            }),
        ColumnDef::text("ref", "Reference", |m: &StockMovementItem| {
            match (&m.reference_doctype, &m.reference_docno) {
                (Some(dt), Some(dn)) => format!("{} {}", dt, dn),
                _ => "-".to_string(),
            }
        })
        .with_width(ColumnWidth::Px(150)),
        ColumnDef::text("date", "Date", |m: &StockMovementItem| m.created_at.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date {
                format: "%d-%b-%Y",
            })
            .with_filter(FilterType::Date),
    ];

    let on_row_click = move |(_idx, m): (usize, StockMovementItem)| {
        tracing::info!("Clicked movement: {}", m.movement_no);
    };

    let on_new = { let mut m = show_modal.clone(); move |_| { m.set(true); } };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| {
            counter += 1;
        }
    };

    let on_modal_success = {
        let mut m = show_modal.clone();
        let mut counter = refresh_counter.clone();
        move |_| { m.set(false); counter += 1; }
    };
    let on_modal_cancel = { let mut m = show_modal.clone(); move |_| { m.set(false); } };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Stock Movements" }
                    p { class: "page-subtitle", "Track all stock movements — receipts, issues, adjustments, and transfers." }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Movement" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: movements.clone(),
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

            Modal {
                is_open: show_modal,
                title: Some("Stock Movement".to_string()),
                size: ModalSize::Lg,
                close_on_backdrop: false,
                StockAdjustmentForm {
                    on_success: on_modal_success,
                    on_cancel: on_modal_cancel,
                }
            }
        }
    }
}
