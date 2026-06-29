//! Quotation List Page — DataGrid-backed list for quotations with status badges,
//! summary bar, toolbar, and row click navigation.

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
// Sample Data
// ============================================================================

async fn fetch_quotations() -> Vec<Quotation> {
    sleep(Duration::from_millis(800)).await;
    sample_quotations_data()
}

fn sample_quotations_data() -> Vec<Quotation> {
    vec![
        Quotation { id: 1, quotation_no: "QOT-2026-0001".to_string(), customer_name: "Alpha Traders".to_string(), date: "2026-06-01".to_string(), valid_until: "2026-07-01".to_string(), status: "Draft".to_string(), total_amount: 125_400.00, item_count: 4 },
        Quotation { id: 2, quotation_no: "QOT-2026-0002".to_string(), customer_name: "Beta Industries".to_string(), date: "2026-06-05".to_string(), valid_until: "2026-07-05".to_string(), status: "Sent".to_string(), total_amount: 67_890.50, item_count: 2 },
        Quotation { id: 3, quotation_no: "QOT-2026-0003".to_string(), customer_name: "Gamma Supplies".to_string(), date: "2026-06-10".to_string(), valid_until: "2026-07-10".to_string(), status: "Accepted".to_string(), total_amount: 234_500.00, item_count: 8 },
        Quotation { id: 4, quotation_no: "QOT-2026-0004".to_string(), customer_name: "Delta Corp".to_string(), date: "2026-06-12".to_string(), valid_until: "2026-07-12".to_string(), status: "Rejected".to_string(), total_amount: 45_600.00, item_count: 3 },
        Quotation { id: 5, quotation_no: "QOT-2026-0005".to_string(), customer_name: "Epsilon LLC".to_string(), date: "2026-06-15".to_string(), valid_until: "2026-07-15".to_string(), status: "Expired".to_string(), total_amount: 98_765.00, item_count: 6 },
        Quotation { id: 6, quotation_no: "QOT-2026-0006".to_string(), customer_name: "Zeta Enterprises".to_string(), date: "2026-06-18".to_string(), valid_until: "2026-07-18".to_string(), status: "Sent".to_string(), total_amount: 12_450.00, item_count: 1 },
        Quotation { id: 7, quotation_no: "QOT-2026-0007".to_string(), customer_name: "Eta Manufacturing".to_string(), date: "2026-06-20".to_string(), valid_until: "2026-07-20".to_string(), status: "Draft".to_string(), total_amount: 312_450.00, item_count: 15 },
        Quotation { id: 8, quotation_no: "QOT-2026-0008".to_string(), customer_name: "Theta Retail".to_string(), date: "2026-06-22".to_string(), valid_until: "2026-07-22".to_string(), status: "Accepted".to_string(), total_amount: 56_780.00, item_count: 3 },
    ]
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
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let quotations_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_quotations().await
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
