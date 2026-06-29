//! Accounting Periods Page — DataGrid-backed list view for accounting/fiscal periods.

use crate::auth::use_auth;
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
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *counter.read();
            let result = api.read().clone().list_accounting_periods().await;
            match result {
                Ok(list) => list.into_iter().map(|p| AccountingPeriod {
                    id: p.id,
                    period_name: p.period_name,
                    start_date: p.start_date,
                    end_date: p.end_date,
                    status: p.status.clone(),
                    is_active: p.status != "Closed",
                }).collect(),
                Err(_) => vec![],
            }
        }
    });
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
