//! Direct Purchase List Page — DataGrid-backed list view for direct purchases.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct DirectPurchase {
    pub id: i64,
    pub dp_no: String,
    pub supplier_name: String,
    pub date: String,
    pub status: String,
    pub total_amount: f64,
    pub item_count: i32,
}



#[component]
pub fn DirectPurchaseListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        let client = use_auth().api;
        let result = client.read().list_direct_purchases().await;
        match result {
            Ok(models) => models.into_iter().map(|m| DirectPurchase {
                id: m.id,
                dp_no: m.purchase_no,
                supplier_name: m.supplier_name,
                date: m.purchase_date,
                status: "Completed".to_string(), // ponytail: server doesn't have status for direct purchases
                total_amount: m.total_cost,
                item_count: 0, // server model lacks item_count
            }).collect(),
            Err(_) => vec![],
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let items = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<DirectPurchase>> = vec![
        ColumnDef::text("dp_no", "DP #", |d: &DirectPurchase| d.dp_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("supplier", "Supplier", |d: &DirectPurchase| d.supplier_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("date", "Date", |d: &DirectPurchase| d.date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |d: &DirectPurchase| d.status.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Gray),
                    ("Approved", BadgeColor::Blue),
                    ("Received", BadgeColor::Green),
                    ("Cancelled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Approved".to_string(), "Received".to_string(), "Cancelled".to_string()],
            }),
        ColumnDef::text("amount", "Amount", |d: &DirectPurchase| d.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("items", "Items", |d: &DirectPurchase| d.item_count.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
    ];

    let total_amount: f64 = items.iter().map(|d| d.total_amount).sum();
    let total_items: i32 = items.iter().map(|d| d.item_count).sum();

    let on_row_click = move |(_idx, d): (usize, DirectPurchase)| {
        navigator.push(format!("/purchases/direct/{}", d.id));
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/purchases/direct/new"); } };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| counter += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Direct Purchases" }
                    p { class: "page-subtitle", "Record and manage purchases made without a purchase order." }
                }
            }

            div { class: "invoice-summary-bar",
                if !is_loading {
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Purchases" }
                        span { class: "summary-value", "{items.len()}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Amount" }
                        span { class: "summary-value summary-amount", "PKR {total_amount:.0}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Items" }
                        span { class: "summary-value", "{total_items}" }
                    }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: on_new, "＋ New Direct Purchase" }
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
