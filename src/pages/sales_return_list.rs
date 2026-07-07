//! Sales Return List Page — DataGrid-backed list for sales returns with status
//! badges, summary bar, toolbar, and row click navigation.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Data Model
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct SalesReturn {
    pub id: i64,
    pub return_no: String,
    pub customer_name: String,
    pub return_date: String,
    pub invoice_ref: String,
    pub status: String,
    pub total_amount: f64,
    pub reason: String,
}



// ============================================================================
// Summary
// ============================================================================

struct ReturnSummary {
    total_count: usize,
    total_amount: f64,
    draft_count: usize,
    approved_count: usize,
    processed_count: usize,
    rejected_count: usize,
}

fn compute_summary(returns: &[SalesReturn]) -> ReturnSummary {
    let mut s = ReturnSummary {
        total_count: returns.len(),
        total_amount: 0.0,
        draft_count: 0, approved_count: 0, processed_count: 0, rejected_count: 0,
    };
    for r in returns {
        s.total_amount += r.total_amount;
        match r.status.as_str() {
            "Draft" => s.draft_count += 1,
            "Approved" => s.approved_count += 1,
            "Processed" => s.processed_count += 1,
            "Rejected" => s.rejected_count += 1,
            _ => {}
        }
    }
    s
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesReturnListPage() -> Element {
    let navigator = use_navigator();
    // ponytail: no sales returns list endpoint — add when server exposes one
    let returns: Vec<SalesReturn> = vec![];
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = false;
    let summary = compute_summary(&returns);

    let columns: Vec<ColumnDef<SalesReturn>> = vec![
        ColumnDef::text("return_no", "Return #", |r: &SalesReturn| r.return_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("customer", "Customer", |r: &SalesReturn| r.customer_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("return_date", "Return Date", |r: &SalesReturn| r.return_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("invoice_ref", "Invoice Ref", |r: &SalesReturn| r.invoice_ref.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("status", "Status", |r: &SalesReturn| r.status.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Yellow),
                    ("Approved", BadgeColor::Green),
                    ("Processed", BadgeColor::Blue),
                    ("Rejected", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Approved".to_string(), "Processed".to_string(), "Rejected".to_string()],
            }),
        ColumnDef::text("total", "Amount", |r: &SalesReturn| r.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("reason", "Reason", |r: &SalesReturn| r.reason.clone())
            .with_width(ColumnWidth::Fr(1.5)),
    ];

    let on_row_click = move |(_idx, r): (usize, SalesReturn)| {
        tracing::info!("Navigate to sales return detail: {}", r.id);
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/sales/returns/new"); } };

    // ponytail: no-op — no data to refresh yet
    let on_refresh = move |_| {};

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Sales Returns" }
                    p { class: "page-subtitle", "Track and manage returned goods from customers." }
                }
            }

            div { class: "invoice-summary-bar",
                if is_loading {
                    {(0..5).map(|_| {
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
                    div { class: "summary-item", span { class: "summary-label", "Approved" } span { class: "summary-value", "{summary.approved_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Processed" } span { class: "summary-value", "{summary.processed_count}" } }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Return" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, "📥 Export" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: returns.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 6,
            }
        }
    }
}
