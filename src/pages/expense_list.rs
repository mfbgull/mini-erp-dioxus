//! Expense List Page — DataGrid-backed list view for expense tracking.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct Expense {
    pub id: i64,
    pub expense_no: String,
    pub category: String,
    pub description: String,
    pub amount: f64,
    pub expense_date: String,
    pub paid_to: String,
    pub payment_method: String,
    pub status: String,
    pub approved_by: Option<String>,
}

async fn fetch_expenses() -> Vec<Expense> {
    crate::utils::sleep(std::time::Duration::from_millis(600)).await;
    sample_expenses()
}

pub fn sample_expenses() -> Vec<Expense> {
    vec![
        Expense { id: 1, expense_no: "EXP-2026-0001".to_string(), category: "Travel".to_string(), description: "Sales team visit to Lahore office".to_string(), amount: 45_000.00, expense_date: "2026-06-20".to_string(), paid_to: "Ahmad Raza".to_string(), payment_method: "Cash".to_string(), status: "Approved".to_string(), approved_by: Some("Fatima Khan".to_string()) },
        Expense { id: 2, expense_no: "EXP-2026-0002".to_string(), category: "Office Supplies".to_string(), description: "Stationery and printer cartridges".to_string(), amount: 8_500.00, expense_date: "2026-06-18".to_string(), paid_to: "Zainab Bibi".to_string(), payment_method: "Cash".to_string(), status: "Approved".to_string(), approved_by: Some("Bilal Ahmed".to_string()) },
        Expense { id: 3, expense_no: "EXP-2026-0003".to_string(), category: "Utilities".to_string(), description: "June electricity bill".to_string(), amount: 125_000.00, expense_date: "2026-06-15".to_string(), paid_to: "IESCO".to_string(), payment_method: "Bank".to_string(), status: "Approved".to_string(), approved_by: Some("Fatima Khan".to_string()) },
        Expense { id: 4, expense_no: "EXP-2026-0004".to_string(), category: "Maintenance".to_string(), description: "AC repair - Admin floor".to_string(), amount: 22_000.00, expense_date: "2026-06-12".to_string(), paid_to: "Cool Tech Services".to_string(), payment_method: "Cash".to_string(), status: "Approved".to_string(), approved_by: Some("Bilal Ahmed".to_string()) },
        Expense { id: 5, expense_no: "EXP-2026-0005".to_string(), category: "Salary".to_string(), description: "Staff salaries for May 2026".to_string(), amount: 450_000.00, expense_date: "2026-06-01".to_string(), paid_to: "All Staff".to_string(), payment_method: "Bank".to_string(), status: "Reimbursed".to_string(), approved_by: Some("Fatima Khan".to_string()) },
        Expense { id: 6, expense_no: "EXP-2026-0006".to_string(), category: "Travel".to_string(), description: "Rawalpindi client meeting - fuel".to_string(), amount: 6_500.00, expense_date: "2026-06-10".to_string(), paid_to: "Usman Ali".to_string(), payment_method: "Cash".to_string(), status: "Draft".to_string(), approved_by: None },
        Expense { id: 7, expense_no: "EXP-2026-0007".to_string(), category: "Office Supplies".to_string(), description: "New desk chairs x 5".to_string(), amount: 75_000.00, expense_date: "2026-06-08".to_string(), paid_to: "Furniture World".to_string(), payment_method: "Credit Card".to_string(), status: "Approved".to_string(), approved_by: Some("Bilal Ahmed".to_string()) },
        Expense { id: 8, expense_no: "EXP-2026-0008".to_string(), category: "Utilities".to_string(), description: "WSSP water bill".to_string(), amount: 3_200.00, expense_date: "2026-06-05".to_string(), paid_to: "WSSP".to_string(), payment_method: "Bank".to_string(), status: "Approved".to_string(), approved_by: Some("Fatima Khan".to_string()) },
        Expense { id: 9, expense_no: "EXP-2026-0009".to_string(), category: "Other".to_string(), description: "Office security system upgrade".to_string(), amount: 95_000.00, expense_date: "2026-05-28".to_string(), paid_to: "SafeTech Security".to_string(), payment_method: "Credit Card".to_string(), status: "Rejected".to_string(), approved_by: Some("Fatima Khan".to_string()) },
        Expense { id: 10, expense_no: "EXP-2026-0010".to_string(), category: "Travel".to_string(), description: "Faisalabad supplier visit".to_string(), amount: 12_000.00, expense_date: "2026-06-22".to_string(), paid_to: "Usman Ali".to_string(), payment_method: "Cash".to_string(), status: "Draft".to_string(), approved_by: None },
    ]
}

struct ExpenseSummary {
    total: usize,
    total_amount: f64,
    by_category: Vec<(String, f64)>,
    approved: usize,
    draft: usize,
}

fn compute_summary(expenses: &[Expense]) -> ExpenseSummary {
    let total = expenses.len();
    let mut total_amount = 0.0;
    let mut cat_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut approved = 0;
    let mut draft = 0;
    for e in expenses {
        total_amount += e.amount;
        *cat_map.entry(e.category.clone()).or_default() += e.amount;
        match e.status.as_str() {
            "Approved" | "Reimbursed" => approved += 1,
            "Draft" => draft += 1,
            _ => {}
        }
    }
    let mut by_category: Vec<(String, f64)> = cat_map.into_iter().collect();
    by_category.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    ExpenseSummary { total, total_amount, by_category, approved, draft }
}

#[component]
pub fn ExpenseListPage() -> Element {
    let navigator = use_navigator();
    let counter = use_signal(|| 0u32);
    let resource = use_resource(move || async move { let _ = *counter.read(); fetch_expenses().await });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let expenses = resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&expenses);

    let columns: Vec<ColumnDef<Expense>> = vec![
        ColumnDef::text("no", "Expense #", |e: &Expense| e.expense_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("category", "Category", |e: &Expense| e.category.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Travel", BadgeColor::Blue),
                    ("Office Supplies", BadgeColor::Purple),
                    ("Utilities", BadgeColor::Yellow),
                    ("Maintenance", BadgeColor::Cyan),
                    ("Salary", BadgeColor::Green),
                    ("Other", BadgeColor::Gray),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Travel".to_string(), "Office Supplies".to_string(), "Utilities".to_string(), "Maintenance".to_string(), "Salary".to_string(), "Other".to_string()],
            }),
        ColumnDef::text("desc", "Description", |e: &Expense| e.description.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("amount", "Amount", |e: &Expense| e.amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_filter(FilterType::Number),
        ColumnDef::text("date", "Date", |e: &Expense| e.expense_date.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("paid_to", "Paid To", |e: &Expense| e.paid_to.clone())
            .with_width(ColumnWidth::Px(140)),
        ColumnDef::text("method", "Method", |e: &Expense| e.payment_method.clone())
            .with_width(ColumnWidth::Px(110))
            .with_filter(FilterType::Select {
                options: vec!["Cash".to_string(), "Bank".to_string(), "Credit Card".to_string()],
            }),
        ColumnDef::text("status", "Status", |e: &Expense| e.status.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Approved", BadgeColor::Green),
                    ("Draft", BadgeColor::Yellow),
                    ("Reimbursed", BadgeColor::Blue),
                    ("Rejected", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Approved".to_string(), "Reimbursed".to_string(), "Rejected".to_string()],
            }),
        ColumnDef::text("approved_by", "Approved By", |e: &Expense| e.approved_by.clone().unwrap_or("-".to_string()))
            .with_width(ColumnWidth::Px(130)),
    ];

    let on_new = { let nav = navigator.clone(); move |_| { nav.push("/expenses/new"); } };
    let on_refresh = { let mut c = counter.clone(); move |_| c += 1 };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { h1 { "Expenses" } p { class: "page-subtitle", "Track all business expenses and reimbursement requests." } }
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
                    div { class: "summary-item", span { class: "summary-label", "Total Expenses" } span { class: "summary-value", "{summary.total}" } }
                    div { class: "summary-item", span { class: "summary-label", "Total Amount" } span { class: "summary-value summary-amount", "PKR {summary.total_amount:.0}" } }
                    div { class: "summary-item summary-ok", span { class: "summary-label", "Approved" } span { class: "summary-value", "{summary.approved}" } }
                    div { class: "summary-item summary-warning", span { class: "summary-label", "Draft" } span { class: "summary-value", "{summary.draft}" } }
                    {summary.by_category.iter().take(3).map(|(cat, amt)| {
                        rsx! {
                            div { class: "summary-item",
                                span { class: "summary-label", "{cat}" }
                                span { class: "summary-value", "PKR {amt:.0}" }
                            }
                        }
                    })}
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Expense" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: expenses.clone(),
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
