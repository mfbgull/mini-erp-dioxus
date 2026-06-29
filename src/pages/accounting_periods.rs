//! Accounting Periods Page — DataGrid-backed list view for accounting/fiscal periods.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct AccountingPeriod {
    pub id: i64,
    pub period_name: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub is_active: bool,
}

async fn fetch_periods() -> Vec<AccountingPeriod> {
    crate::utils::sleep(std::time::Duration::from_millis(400)).await;
    sample_periods()
}

fn sample_periods() -> Vec<AccountingPeriod> {
    vec![
        AccountingPeriod { id: 1, period_name: "January 2026".to_string(), start_date: "2026-01-01".to_string(), end_date: "2026-01-31".to_string(), status: "Closed".to_string(), is_active: false },
        AccountingPeriod { id: 2, period_name: "February 2026".to_string(), start_date: "2026-02-01".to_string(), end_date: "2026-02-28".to_string(), status: "Closed".to_string(), is_active: false },
        AccountingPeriod { id: 3, period_name: "March 2026".to_string(), start_date: "2026-03-01".to_string(), end_date: "2026-03-31".to_string(), status: "Closed".to_string(), is_active: false },
        AccountingPeriod { id: 4, period_name: "April 2026".to_string(), start_date: "2026-04-01".to_string(), end_date: "2026-04-30".to_string(), status: "Closed".to_string(), is_active: false },
        AccountingPeriod { id: 5, period_name: "May 2026".to_string(), start_date: "2026-05-01".to_string(), end_date: "2026-05-31".to_string(), status: "Closed".to_string(), is_active: false },
        AccountingPeriod { id: 6, period_name: "June 2026".to_string(), start_date: "2026-06-01".to_string(), end_date: "2026-06-30".to_string(), status: "Open".to_string(), is_active: true },
        AccountingPeriod { id: 7, period_name: "July 2026".to_string(), start_date: "2026-07-01".to_string(), end_date: "2026-07-31".to_string(), status: "Open".to_string(), is_active: true },
        AccountingPeriod { id: 8, period_name: "August 2026".to_string(), start_date: "2026-08-01".to_string(), end_date: "2026-08-31".to_string(), status: "Open".to_string(), is_active: true },
        AccountingPeriod { id: 9, period_name: "September 2026".to_string(), start_date: "2026-09-01".to_string(), end_date: "2026-09-30".to_string(), status: "Locked".to_string(), is_active: true },
        AccountingPeriod { id: 10, period_name: "FY 2025-2026".to_string(), start_date: "2025-07-01".to_string(), end_date: "2026-06-30".to_string(), status: "Open".to_string(), is_active: true },
    ]
}

fn badge_class(status: &str) -> &'static str {
    match status {
        "Open" => "customer-table-badge-green",
        "Closed" => "customer-table-badge-gray",
        "Locked" => "customer-table-badge-red",
        _ => "customer-table-badge-yellow",
    }
}

#[component]
pub fn AccountingPeriodsPage() -> Element {
    let navigator = use_navigator();
    let counter = use_signal(|| 0u32);
    let resource = use_resource(move || async move { let _ = *counter.read(); fetch_periods().await });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let periods = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<AccountingPeriod>> = vec![
        ColumnDef::text("name", "Period Name", |p: &AccountingPeriod| p.period_name.clone())
            .with_width(ColumnWidth::Fr(0.8))
            .with_filter(FilterType::Text),
        ColumnDef::text("start", "Start Date", |p: &AccountingPeriod| p.start_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("end", "End Date", |p: &AccountingPeriod| p.end_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("status", "Status", |p: &AccountingPeriod| p.status.clone())
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Open", BadgeColor::Green),
                    ("Closed", BadgeColor::Gray),
                    ("Locked", BadgeColor::Red),
                ],
                default_color: BadgeColor::Yellow,
            })
            .with_filter(FilterType::Select {
                options: vec!["Open".to_string(), "Closed".to_string(), "Locked".to_string()],
            }),
        ColumnDef::text("active", "Active", |p: &AccountingPeriod| if p.is_active { "Active" } else { "Inactive" }.into())
            .with_width(ColumnWidth::Px(90))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Active", BadgeColor::Green), ("Inactive", BadgeColor::Gray)],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Active".to_string(), "Inactive".to_string()],
            }),
    ];

    let on_refresh = { let mut c = counter.clone(); move |_| c += 1 };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { h1 { "Accounting Periods" } p { class: "page-subtitle", "Manage fiscal periods, open/close periods, and lock periods for processing." } }
            }
            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: move |_| { navigator.push("/accounting/periods/new"); }, "＋ New Period" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }
            DataGrid {
                columns: columns.clone(),
                rows: periods.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true, hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 5,
            }
        }
    }
}
