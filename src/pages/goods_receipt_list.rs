//! Goods Receipt List Page — DataGrid-backed list view for goods receipt notes.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct GoodsReceipt {
    pub id: i64,
    pub grn_no: String,
    pub po_ref: String,
    pub supplier_name: String,
    pub received_date: String,
    pub status: String,
    pub total_items: i32,
    pub warehouse: String,
}



#[component]
pub fn GoodsReceiptListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    // ponytail: no standalone receipts endpoint -- add when server exposes GET /api/receipts
    let resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        Vec::new()
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let items: Vec<GoodsReceipt> = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<GoodsReceipt>> = vec![
        ColumnDef::text("grn", "GRN #", |g: &GoodsReceipt| g.grn_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("po_ref", "PO Ref", |g: &GoodsReceipt| g.po_ref.clone())
            .with_width(ColumnWidth::Px(120)),
        ColumnDef::text("supplier", "Supplier", |g: &GoodsReceipt| g.supplier_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("date", "Received", |g: &GoodsReceipt| g.received_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("warehouse", "Warehouse", |g: &GoodsReceipt| g.warehouse.clone())
            .with_width(ColumnWidth::Px(150)),
        ColumnDef::text("status", "Status", |g: &GoodsReceipt| g.status.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Gray),
                    ("Completed", BadgeColor::Green),
                    ("Cancelled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Completed".to_string(), "Cancelled".to_string()],
            }),
        ColumnDef::text("items", "Items", |g: &GoodsReceipt| g.total_items.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
    ];

    let total_items: i32 = items.iter().map(|g| g.total_items).sum();
    let pending = items.iter().filter(|g| g.status == "Draft").count();

    let on_row_click = move |(_idx, g): (usize, GoodsReceipt)| {
        tracing::info!("Clicked GRN: {}", g.grn_no);
    };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| counter += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Goods Receipts" }
                    p { class: "page-subtitle", "Record and track goods received against purchase orders." }
                }
            }

            div { class: "invoice-summary-bar",
                if !is_loading {
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Receipts" }
                        span { class: "summary-value", "{items.len()}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Items" }
                        span { class: "summary-value", "{total_items}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Completed" }
                        span { class: "summary-value", "{items.iter().filter(|g| g.status == \"Completed\").count()}" }
                    }
                    div { class: "summary-item summary-warning",
                        span { class: "summary-label", "Pending (Draft)" }
                        span { class: "summary-value", "{pending}" }
                    }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: items.clone(),
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
