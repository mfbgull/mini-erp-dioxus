//! Chart of Accounts Page — DataGrid-backed list view for the accounting chart of accounts.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct Account {
    pub id: i64,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub normal_side: String,
    pub balance: f64,
    pub is_active: bool,
}



struct AccountSummary {
    total_assets: f64,
    total_liabilities: f64,
    total_equity: f64,
    total_income: f64,
    total_expense: f64,
}

fn compute_summary(accounts: &[Account]) -> AccountSummary {
    let mut summary = AccountSummary { total_assets: 0.0, total_liabilities: 0.0, total_equity: 0.0, total_income: 0.0, total_expense: 0.0 };
    for a in accounts {
        if !a.is_active { continue; }
        match a.account_type.as_str() {
            "Asset" => summary.total_assets += a.balance,
            "Liability" => summary.total_liabilities += a.balance,
            "Equity" => summary.total_equity += a.balance,
            "Income" => summary.total_income += a.balance,
            "Expense" => summary.total_expense += a.balance,
            _ => {}
        }
    }
    summary
}

#[component]
pub fn ChartOfAccountsPage() -> Element {
    let navigator = use_navigator();
    let counter = use_signal(|| 0u32);
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *counter.read();
            let result = api.read().clone().list_account_balances().await;
            match result {
                Ok(list) => list.into_iter().map(|a| Account {
                    id: a.id,
                    account_code: a.code,
                    account_name: a.name,
                    account_type: a.account_type,
                    normal_side: a.normal_balance,
                    balance: a.balance,
                    is_active: true,
                }).collect(),
                Err(_) => vec![],
            }
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let accounts = resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&accounts);

    let columns: Vec<ColumnDef<Account>> = vec![
        ColumnDef::text("code", "Code", |a: &Account| a.account_code.clone())
            .with_width(ColumnWidth::Px(100))
            .with_filter(FilterType::Text),
        ColumnDef::text("name", "Account Name", |a: &Account| a.account_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("type", "Type", |a: &Account| a.account_type.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Asset", BadgeColor::Green),
                    ("Liability", BadgeColor::Red),
                    ("Equity", BadgeColor::Blue),
                    ("Income", BadgeColor::Purple),
                    ("Expense", BadgeColor::Yellow),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Asset".to_string(), "Liability".to_string(), "Equity".to_string(), "Income".to_string(), "Expense".to_string()],
            }),
        ColumnDef::text("side", "Normal Side", |a: &Account| a.normal_side.clone())
            .with_width(ColumnWidth::Px(100)),
        ColumnDef::text("balance", "Balance", |a: &Account| a.balance.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_filter(FilterType::Number),
        ColumnDef::text("active", "Active", |a: &Account| if a.is_active { "Yes" } else { "No" }.into())
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Yes", BadgeColor::Green), ("No", BadgeColor::Gray)],
                default_color: BadgeColor::Gray,
            }),
    ];

    let on_refresh = { let mut c = counter.clone(); move |_| c += 1 };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { h1 { "Chart of Accounts" } p { class: "page-subtitle", "Manage your chart of accounts — add, edit, and organize account codes." } }
            }

            div { class: "customer-summary-bar",
                if is_loading {
                    {[0; 5].iter().map(|_| rsx! {
                        div { class: "summary-item summary-skeleton",
                            div { class: "skeleton-text", style: "width: 60%; height: 10px;" }
                            div { class: "skeleton-text", style: "width: 80%; height: 20px; margin-top: 6px;" }
                        }
                    })}
                } else {
                    div { class: "summary-item", span { class: "summary-label", "Assets" } span { class: "summary-value summary-amount", "PKR {summary.total_assets:.0}" } }
                    div { class: "summary-item summary-warning", span { class: "summary-label", "Liabilities" } span { class: "summary-value", "PKR {summary.total_liabilities:.0}" } }
                    div { class: "summary-item summary-ok", span { class: "summary-label", "Equity" } span { class: "summary-value", "PKR {summary.total_equity:.0}" } }
                    div { class: "summary-item", span { class: "summary-label", "Income" } span { class: "summary-value summary-amount", "PKR {summary.total_income:.0}" } }
                    div { class: "summary-item", span { class: "summary-label", "Expenses" } span { class: "summary-value", "PKR {summary.total_expense:.0}" } }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: move |_| { navigator.push("/accounting/chart-of-accounts/new"); }, "＋ New Account" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: accounts.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true, hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 8,
            }
        }
    }
}
