//! Chart of Accounts Page — DataGrid-backed list view for the accounting chart of accounts.

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

async fn fetch_accounts() -> Vec<Account> {
    crate::utils::sleep(std::time::Duration::from_millis(500)).await;
    sample_accounts()
}

fn sample_accounts() -> Vec<Account> {
    vec![
        Account { id: 1, account_code: "1000".to_string(), account_name: "Cash on Hand".to_string(), account_type: "Asset".to_string(), normal_side: "Debit".to_string(), balance: 450_000.00, is_active: true },
        Account { id: 2, account_code: "1100".to_string(), account_name: "Bank Accounts".to_string(), account_type: "Asset".to_string(), normal_side: "Debit".to_string(), balance: 2_800_000.00, is_active: true },
        Account { id: 3, account_code: "1200".to_string(), account_name: "Accounts Receivable".to_string(), account_type: "Asset".to_string(), normal_side: "Debit".to_string(), balance: 1_200_000.00, is_active: true },
        Account { id: 4, account_code: "1300".to_string(), account_name: "Inventory".to_string(), account_type: "Asset".to_string(), normal_side: "Debit".to_string(), balance: 650_000.00, is_active: true },
        Account { id: 5, account_code: "1400".to_string(), account_name: "Fixed Assets".to_string(), account_type: "Asset".to_string(), normal_side: "Debit".to_string(), balance: 2_100_000.00, is_active: true },
        Account { id: 6, account_code: "2000".to_string(), account_name: "Accounts Payable".to_string(), account_type: "Liability".to_string(), normal_side: "Credit".to_string(), balance: 1_200_000.00, is_active: true },
        Account { id: 7, account_code: "2100".to_string(), account_name: "Accrued Expenses".to_string(), account_type: "Liability".to_string(), normal_side: "Credit".to_string(), balance: 350_000.00, is_active: true },
        Account { id: 8, account_code: "2200".to_string(), account_name: "Short Term Loans".to_string(), account_type: "Liability".to_string(), normal_side: "Credit".to_string(), balance: 250_000.00, is_active: true },
        Account { id: 9, account_code: "3000".to_string(), account_name: "Owner's Equity".to_string(), account_type: "Equity".to_string(), normal_side: "Credit".to_string(), balance: 3_200_000.00, is_active: true },
        Account { id: 10, account_code: "3100".to_string(), account_name: "Retained Earnings".to_string(), account_type: "Equity".to_string(), normal_side: "Credit".to_string(), balance: 200_000.00, is_active: true },
        Account { id: 11, account_code: "4000".to_string(), account_name: "Sales Revenue".to_string(), account_type: "Income".to_string(), normal_side: "Credit".to_string(), balance: 6_500_000.00, is_active: true },
        Account { id: 12, account_code: "4100".to_string(), account_name: "Service Revenue".to_string(), account_type: "Income".to_string(), normal_side: "Credit".to_string(), balance: 890_000.00, is_active: true },
        Account { id: 13, account_code: "5000".to_string(), account_name: "Cost of Goods Sold".to_string(), account_type: "Expense".to_string(), normal_side: "Debit".to_string(), balance: 4_200_000.00, is_active: true },
        Account { id: 14, account_code: "5100".to_string(), account_name: "Rent Expense".to_string(), account_type: "Expense".to_string(), normal_side: "Debit".to_string(), balance: 240_000.00, is_active: true },
        Account { id: 15, account_code: "5200".to_string(), account_name: "Utilities Expense".to_string(), account_type: "Expense".to_string(), normal_side: "Debit".to_string(), balance: 128_000.00, is_active: true },
        Account { id: 16, account_code: "5300".to_string(), account_name: "Salary Expense".to_string(), account_type: "Expense".to_string(), normal_side: "Debit".to_string(), balance: 540_000.00, is_active: true },
        Account { id: 17, account_code: "5400".to_string(), account_name: "Depreciation".to_string(), account_type: "Expense".to_string(), normal_side: "Debit".to_string(), balance: 85_000.00, is_active: true },
        Account { id: 18, account_code: "6000".to_string(), account_name: "Other Income".to_string(), account_type: "Income".to_string(), normal_side: "Credit".to_string(), balance: 45_000.00, is_active: false },
    ]
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
    let resource = use_resource(move || async move { let _ = *counter.read(); fetch_accounts().await });
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
