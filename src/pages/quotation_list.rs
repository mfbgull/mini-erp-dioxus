//! Quotation List Page — DataGrid-backed list for quotations with status badges,
//! summary bar, toolbar, and row click navigation.

use crate::auth::use_auth;
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
pub struct Quotation {
    pub id: i64,
    pub quotation_no: String,
    pub customer_name: String,
    pub date: String,
    pub valid_until: String,
    pub status: String,
    pub total_amount: f64,
    pub item_count: i32,
}



// ============================================================================
// Summary
// ============================================================================

struct QuotationSummary {
    total_count: usize,
    total_amount: f64,
    draft_count: usize,
    sent_count: usize,
    accepted_count: usize,
    rejected_count: usize,
    expired_count: usize,
}

fn compute_summary(quotes: &[Quotation]) -> QuotationSummary {
    let mut s = QuotationSummary {
        total_count: quotes.len(),
        total_amount: 0.0,
        draft_count: 0,
        sent_count: 0,
        accepted_count: 0,
        rejected_count: 0,
        expired_count: 0,
    };
    for q in quotes {
        s.total_amount += q.total_amount;
        match q.status.as_str() {
            "Draft" => s.draft_count += 1,
            "Sent" => s.sent_count += 1,
            "Accepted" => s.accepted_count += 1,
            "Rejected" => s.rejected_count += 1,
            "Expired" => s.expired_count += 1,
            _ => {}
        }
    }
    s
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationListPage() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let quotations_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        let api = auth.api.read();
        let client = api.clone();
        drop(api);
        client.list_quotations().await.unwrap_or_default().into_iter().map(|q| Quotation {
            id: q.id,
            quotation_no: q.quotation_no,
            customer_name: q.customer_name.unwrap_or_default(),
            date: q.quotation_date,
            valid_until: q.expiry_date,
            status: q.status,
            total_amount: q.total_amount,
            item_count: 0,
        }).collect::<Vec<_>>()
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = quotations_resource.read().is_none();
    let quotations = quotations_resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&quotations);

    let columns: Vec<ColumnDef<Quotation>> = vec![
        ColumnDef::text("qot_no", "Quotation #", |q: &Quotation| q.quotation_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("customer", "Customer", |q: &Quotation| q.customer_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("date", "Date", |q: &Quotation| q.date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("valid_until", "Valid Until", |q: &Quotation| q.valid_until.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |q: &Quotation| q.status.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Yellow),
                    ("Sent", BadgeColor::Blue),
                    ("Accepted", BadgeColor::Green),
                    ("Rejected", BadgeColor::Red),
                    ("Expired", BadgeColor::Gray),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Sent".to_string(), "Accepted".to_string(), "Rejected".to_string(), "Expired".to_string()],
            }),
        ColumnDef::text("total", "Total", |q: &Quotation| q.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("items", "Items", |q: &Quotation| q.item_count.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(70))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
    ];

    let on_row_click = move |(_idx, q): (usize, Quotation)| {
        tracing::info!("Navigate to quotation detail: {}", q.id);
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/sales/quotations/new"); } };

    let on_refresh = {
        let mut cnt = refresh_counter.clone();
        move |_| cnt += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Quotations" }
                    p { class: "page-subtitle", "Manage sales quotations — create, send, and track acceptances." }
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
                    div { class: "summary-item", span { class: "summary-label", "Sent" } span { class: "summary-value", "{summary.sent_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Accepted" } span { class: "summary-value", "{summary.accepted_count}" } }
                    div { class: "summary-item", span { class: "summary-label", "Rejected" } span { class: "summary-value", "{summary.rejected_count}" } }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Quotation" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, "📥 Export" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: quotations.clone(),
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
