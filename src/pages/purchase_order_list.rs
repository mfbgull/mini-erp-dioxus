//! Purchase Order List Page — DataGrid-backed list view for purchase orders.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct PurchaseOrder {
    pub id: i64,
    pub po_no: String,
    pub supplier_name: String,
    pub order_date: String,
    pub expected_date: String,
    pub status: String,
    pub total_amount: f64,
    pub item_count: i32,
}

struct PoSummary {
    total_count: usize,
    total_amount: f64,
    open_count: usize,
    received_count: usize,
}



fn compute_summary(orders: &[PurchaseOrder]) -> PoSummary {
    let total_count = orders.len();
    let total_amount: f64 = orders.iter().map(|o| o.total_amount).sum();
    let open_count = orders.iter().filter(|o| matches!(o.status.as_str(), "Draft" | "Sent" | "Confirmed" | "Partially Received")).count();
    let received_count = orders.iter().filter(|o| o.status == "Received").count();
    PoSummary { total_count, total_amount, open_count, received_count }
}

#[component]
pub fn PurchaseOrderListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_purchase_orders().await
                .map(|server_pos| {
                    server_pos.into_iter().map(|po| PurchaseOrder {
                        id: po.id,
                        po_no: po.po_no,
                        supplier_name: po.supplier_name.unwrap_or_default(),
                        order_date: po.po_date,
                        expected_date: String::new(), // ponytail: not in list endpoint
                        status: po.status,
                        total_amount: po.total_amount,
                        item_count: 0, // ponytail: not in list endpoint
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let orders = resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&orders);

    let columns: Vec<ColumnDef<PurchaseOrder>> = vec![
        ColumnDef::text("po_no", "PO #", |o: &PurchaseOrder| o.po_no.clone())
            .with_width(ColumnWidth::Px(130))
            .with_filter(FilterType::Text),
        ColumnDef::text("supplier", "Supplier", |o: &PurchaseOrder| o.supplier_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("order_date", "Order Date", |o: &PurchaseOrder| o.order_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("expected", "Expected", |o: &PurchaseOrder| o.expected_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |o: &PurchaseOrder| o.status.clone())
            .with_width(ColumnWidth::Px(150))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Gray),
                    ("Sent", BadgeColor::Blue),
                    ("Confirmed", BadgeColor::Green),
                    ("Partially Received", BadgeColor::Yellow),
                    ("Received", BadgeColor::Green),
                    ("Cancelled", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Sent".to_string(), "Confirmed".to_string(), "Partially Received".to_string(), "Received".to_string(), "Cancelled".to_string()],
            }),
        ColumnDef::text("amount", "Amount", |o: &PurchaseOrder| o.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("items", "Items", |o: &PurchaseOrder| o.item_count.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
    ];

    let on_row_click = move |(_idx, o): (usize, PurchaseOrder)| {
        navigator.push(format!("/purchases/orders/{}", o.id));
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/purchases/orders/new"); } };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Purchase Orders" }
                    p { class: "page-subtitle", "Create, track, and receive purchase orders from suppliers." }
                }
                if is_loading { div { class: "loading-badge", div { class: "loading-badge-spinner" } span { "Loading…" } } }
            }

            div { class: "invoice-summary-bar",
                if is_loading { {[0; 5].iter().map(|_| rsx! { div { class: "summary-item summary-skeleton", div { class: "skeleton-text", style: "width: 60%; height: 10px;" } div { class: "skeleton-text", style: "width: 80%; height: 20px; margin-top: 6px;" } } })} } else {
                    div { class: "summary-item", span { class: "summary-label", "Total Orders" } span { class: "summary-value", "{summary.total_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Total Amount" } span { class: "summary-value summary-amount", "PKR {summary.total_amount:.0}" } }
                    div { class: "summary-item", span { class: "summary-label", "Open" } span { class: "summary-value", "{summary.open_count}" } }
                    div { class: "summary-item summary-warning", span { class: "summary-label", "Draft" } span { class: "summary-value", "{orders.iter().filter(|o| o.status == \"Draft\").count()}" } }
                    div { class: "summary-item", span { class: "summary-label", "Received" } span { class: "summary-value", "{summary.received_count}" } }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Purchase Order" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, "📥 Export" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: orders.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 8,
                virtual_scroll: true,
                virtual_scroll_height: 500.0,
            }
        }
    }
}
