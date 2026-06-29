//! Expense Category List Page — DataGrid-backed list view for expense categories.

use crate::components::data_grid::{
    BadgeColor, CellClassRule, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType,
    PaginationMode, RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct ExpenseCategory {
    pub id: i64,
    pub category_name: String,
    pub description: String,
    pub budget_amount: f64,
    pub spent_amount: f64,
    pub is_active: bool,
}

async fn fetch_categories() -> Vec<ExpenseCategory> {
    crate::utils::sleep(std::time::Duration::from_millis(400)).await;
    sample_categories()
}

fn sample_categories() -> Vec<ExpenseCategory> {
    vec![
        ExpenseCategory { id: 1, category_name: "Travel".to_string(), description: "Business travel and transportation".to_string(), budget_amount: 300_000.00, spent_amount: 63_500.00, is_active: true },
        ExpenseCategory { id: 2, category_name: "Office Supplies".to_string(), description: "Stationery, furniture, and office consumables".to_string(), budget_amount: 150_000.00, spent_amount: 83_500.00, is_active: true },
        ExpenseCategory { id: 3, category_name: "Utilities".to_string(), description: "Electricity, water, gas, internet, phone".to_string(), budget_amount: 500_000.00, spent_amount: 128_200.00, is_active: true },
        ExpenseCategory { id: 4, category_name: "Maintenance".to_string(), description: "Repairs and facility maintenance".to_string(), budget_amount: 200_000.00, spent_amount: 22_000.00, is_active: true },
        ExpenseCategory { id: 5, category_name: "Salary".to_string(), description: "Staff salaries and wages".to_string(), budget_amount: 3_000_000.00, spent_amount: 450_000.00, is_active: true },
        ExpenseCategory { id: 6, category_name: "Marketing".to_string(), description: "Advertising and promotional activities".to_string(), budget_amount: 250_000.00, spent_amount: 145_000.00, is_active: true },
        ExpenseCategory { id: 7, category_name: "Rent".to_string(), description: "Office rent and lease payments".to_string(), budget_amount: 400_000.00, spent_amount: 200_000.00, is_active: true },
        ExpenseCategory { id: 8, category_name: "IT & Software".to_string(), description: "Software subscriptions and IT services".to_string(), budget_amount: 180_000.00, spent_amount: 95_000.00, is_active: false },
    ]
}

#[component]
pub fn ExpenseCategoryListPage() -> Element {
    let navigator = use_navigator();
    let counter = use_signal(|| 0u32);
    let resource = use_resource(move || async move { let _ = *counter.read(); fetch_categories().await });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let categories = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<ExpenseCategory>> = vec![
        ColumnDef::text("name", "Category", |c: &ExpenseCategory| c.category_name.clone())
            .with_width(ColumnWidth::Fr(0.8))
            .with_filter(FilterType::Text),
        ColumnDef::text("desc", "Description", |c: &ExpenseCategory| c.description.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("budget", "Budget", |c: &ExpenseCategory| c.budget_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_filter(FilterType::Number),
        ColumnDef::text("spent", "Spent", |c: &ExpenseCategory| c.spent_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_cell_class(CellClassRule::new(|c: &ExpenseCategory| {
                if c.spent_amount > c.budget_amount { "text-danger fw-bold".to_string() }
                else if c.spent_amount > c.budget_amount * 0.8 { "text-warning".to_string() }
                else { String::new() }
            })),
        ColumnDef::text("remaining", "Remaining", |c: &ExpenseCategory| (c.budget_amount - c.spent_amount).to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_cell_class(CellClassRule::new(|c: &ExpenseCategory| {
                if c.spent_amount > c.budget_amount { "text-danger".to_string() }
                else { "text-success".to_string() }
            })),
        ColumnDef::text("active", "Active", |c: &ExpenseCategory| if c.is_active { "Active" } else { "Inactive" }.into())
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
                div { h1 { "Expense Categories" } p { class: "page-subtitle", "Manage expense categories, budgets, and track spending vs budget." } }
            }
            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", onclick: move |_| { navigator.push("/expenses/categories/new"); }, "＋ New Category" }
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }
            DataGrid {
                columns: columns.clone(),
                rows: categories.clone(),
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
