//! Goods Receipt List Page — DataGrid-backed list view for goods receipt notes.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct GoodsReceiptRow {
    pub id: i64,
    pub receipt_no: String,
    pub po_ref: String,
    pub received_date: String,
    pub warehouse: String,
    pub notes: String,
    pub created_at: String,
}



#[component]
pub fn GoodsReceiptListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let api = use_auth().api;

    let resource = use_resource(move || {
        let api = api.clone();
        let _ = *refresh_counter.read();
        async move {
            let client = api.with(|c| c.clone());
            match client.list_receipts().await {
                Ok(receipts) => {
                    let rows: Vec<GoodsReceiptRow> = receipts.iter().map(|r| {
                        let po_id = r.get("po_id").and_then(|v| v.as_i64()).unwrap_or(0);
                        GoodsReceiptRow {
                            id: r.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
                            receipt_no: r.get("receipt_no").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            po_ref: format!("PO-{}", po_id),
                            received_date: r.get("receipt_date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            warehouse: r.get("warehouse_name").and_then(|v| v.as_str()).unwrap_or("—").to_string(),
                            notes: r.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            created_at: r.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        }
                    }).collect();
                    rows
                }
                Err(_) => Vec::new(),
            }
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let items: Vec<GoodsReceiptRow> = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<GoodsReceiptRow>> = vec![
        ColumnDef::text("grn", "Receipt #", |g: &GoodsReceiptRow| g.receipt_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("po_ref", "PO Ref", |g: &GoodsReceiptRow| g.po_ref.clone())
            .with_width(ColumnWidth::Px(120)),
        ColumnDef::text("date", "Received", |g: &GoodsReceiptRow| g.received_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("warehouse", "Warehouse", |g: &GoodsReceiptRow| g.warehouse.clone())
            .with_width(ColumnWidth::Px(150)),
        ColumnDef::text("notes", "Notes", |g: &GoodsReceiptRow| g.notes.clone())
            .with_width(ColumnWidth::Fr(1.0)),
    ];

    let on_row_click = move |(_idx, g): (usize, GoodsReceiptRow)| {
        tracing::info!("Clicked Receipt: {}", g.receipt_no);
    };

    let this_month = chrono::Local::now().format("%Y-%m").to_string();
    let this_month_count = items.iter().filter(|g| g.received_date.starts_with(&this_month)).count();

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
                        span { class: "summary-label", "This Month" }
                        span { class: "summary-value", "{this_month_count}" }
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
