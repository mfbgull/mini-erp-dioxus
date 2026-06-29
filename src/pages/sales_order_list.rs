//! Sales Order List Page — DataGrid-backed list for sales orders with status
//! badges, summary bar, toolbar, and row click navigation.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use crate::utils::sleep;
use std::collections::HashSet;
use std::time::Duration;

// ============================================================================
// Data Model
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct SalesOrder {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub order_date: String,
    pub delivery_date: String,
    pub status: String,
    pub total_amount: f64,
    pub item_count: i32,
}

// ============================================================================
// Sample Data
// ============================================================================

async fn fetch_orders() -> Vec<SalesOrder> {
    sleep(Duration::from_millis(800)).await;
    sample_orders_data()
}

fn sample_orders_data() -> Vec<SalesOrder> {
    vec![
        SalesOrder { id: 1, order_no: "SO-2026-0001".to_string(), customer_name: "Alpha Traders".to_string(), order_date: "2026-06-01".to_string(), delivery_date: "2026-06-15".to_string(), status: "Draft".to_string(), total_amount: 125_400.00, item_count: 4 },
        SalesOrder { id: 2, order_no: "SO-2026-0002".to_string(), customer_name: "Beta Industries".to_string(), order_date: "2026-06-05".to_string(), delivery_date: "2026-06-20".to_string(), status: "Confirmed".to_string(), total_amount: 67_890.50, item_count: 2 },
        SalesOrder { id: 3, order_no: "SO-2026-0003".to_string(), customer_name: "Gamma Supplies".to_string(), order_date: "2026-06-10".to_string(), delivery_date: "2026-06-25".to_string(), status: "Processing".to_string(), total_amount: 234_500.00, item_count: 8 },
        SalesOrder { id: 4, order_no: "SO-2026-0004".to_string(), customer_name: "Delta Corp".to_string(), order_date: "2026-06-12".to_string(), delivery_date: "2026-06-28".to_string(), status: "Shipped".to_string(), total_amount: 98_765.00, item_count: 6 },
        SalesOrder { id: 5, order_no: "SO-2026-0005".to_string(), customer_name: "Epsilon LLC".to_string(), order_date: "2026-06-15".to_string(), delivery_date: "2026-07-05".to_string(), status: "Delivered".to_string(), total_amount: 312_450.00, item_count: 15 },
        SalesOrder { id: 6, order_no: "SO-2026-0006".to_string(), customer_name: "Zeta Enterprises".to_string(), order_date: "2026-06-18".to_string(), delivery_date: "2026-07-03".to_string(), status: "Cancelled".to_string(), total_amount: 12_450.00, item_count: 1 },
        SalesOrder { id: 7, order_no: "SO-2026-0007".to_string(), customer_name: "Eta Manufacturing".to_string(), order_date: "2026-06-20".to_string(), delivery_date: "2026-07-10".to_string(), status: "Confirmed".to_string(), total_amount: 56_780.00, item_count: 3 },
        SalesOrder { id: 8, order_no: "SO-2026-0008".to_string(), customer_name: "Theta Retail".to_string(), order_date: "2026-06-22".to_string(), delivery_date: "2026-07-12".to_string(), status: "Draft".to_string(), total_amount: 178_900.00, item_count: 10 },
    ]
}

// ============================================================================
// Summary
// ============================================================================

struct OrderSummary {
    total_count: usize,
    total_amount: f64,
    draft_count: usize,
    confirmed_count: usize,
    processing_count: usize,
    shipped_count: usize,
    delivered_count: usize,
}

fn compute_summary(orders: &[SalesOrder]) -> OrderSummary {
    let mut s = OrderSummary {
        total_count: orders.len(),
        total_amount: 0.0,
        draft_count: 0, confirmed_count: 0, processing_count: 0,
        shipped_count: 0, delivered_count: 0,
    };
    for o in orders {
        s.total_amount += o.total_amount;
        match o.status.as_str() {
            "Draft" => s.draft_count += 1,
            "Confirmed" => s.confirmed_count += 1,
            "Processing" => s.processing_count += 1,
            "Shipped" => s.shipped_count += 1,
            "Delivered" => s.delivered_count += 1,
            _ => {}
        }
    }
    s
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesOrderListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let orders_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_orders().await
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = orders_resource.read().is_none();
    let orders = orders_resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&orders);

    let columns: Vec<ColumnDef<SalesOrder>> = vec![
        ColumnDef::text("so_no", "Order #", |o: &SalesOrder| o.order_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("customer", "Customer", |o: &SalesOrder| o.customer_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("order_date", "Order Date", |o: &SalesOrder| o.order_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("delivery_date", "Delivery Date", |o: &SalesOrder| o.delivery_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |o: &SalesOrder| o.status.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Yellow),
                    ("Confirmed", BadgeColor::Blue),
                    ("Processing", BadgeColor::Purple),
                    ("Shipped", BadgeColor::Cyan),
                    ("Delivered", BadgeColor::Green),
                    ("Cancelled", BadgeColor::Gray),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Confirmed".to_string(), "Processing".to_string(), "Shipped".to_string(), "Delivered".to_string(), "Cancelled".to_string()],
            }),
        ColumnDef::text("total", "Total", |o: &SalesOrder| o.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("items", "Items", |o: &SalesOrder| o.item_count.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(70))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
    ];

    let on_row_click = move |(_idx, o): (usize, SalesOrder)| {
        tracing::info!("Navigate to sales order detail: {}", o.id);
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/sales/orders/new"); } };

    let on_refresh = {
        let mut cnt = refresh_counter.clone();
        move |_| cnt += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Sales Orders" }
                    p { class: "page-subtitle", "Manage customer sales orders from creation through delivery." }
                }
            }

            div { class: "invoice-summary-bar",
                if is_loading {
                    {(0..6).map(|_| {
                        rsx! {
                            div { class: "summary-item summary-skeleton",
                                div { class: "skeleton-text", style: "width: 60%; height: 10px;" }
                                div { class: "skeleton-text", style: "width: 80%; height: 20px; margin-top: 6px;" }
                            }
                        }
                    })}
                } else {
                    div { class: "summary-item", span { class: "summary-label", "Total" } span { class: "summary-value", "{summary.total_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Amount" } span { class: "summary-value summary-amount", "PKR {summary.total_amount:.0}" } }
                    div { class: "summary-item", span { class: "summary-label", "Draft" } span { class: "summary-value", "{summary.draft_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Confirmed" } span { class: "summary-value", "{summary.confirmed_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Processing" } span { class: "summary-value", "{summary.processing_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Shipped" } span { class: "summary-value", "{summary.shipped_count}" } }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Sales Order" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, "📥 Export" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
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
            }
        }
    }
}
