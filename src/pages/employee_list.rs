//! Employee List Page — DataGrid-backed list view for employee management.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct Employee {
    pub id: i64,
    pub employee_code: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub department: String,
    pub designation: String,
    pub employment_type: String,
    pub status: String,
    pub join_date: String,
}

async fn fetch_employees(client: &crate::api::ApiClient) -> Vec<Employee> {
    match client.list_employees().await {
        Ok(server_emps) => server_emps
            .into_iter()
            .map(|e| Employee {
                id: e.id,
                employee_code: e.employee_code,
                full_name: format!("{} {}", e.first_name, e.last_name),
                email: e.email,
                phone: e.phone,
                department: e.department,
                designation: e.designation,
                employment_type: e.employment_type,
                status: if e.is_active {
                    "Active".to_string()
                } else {
                    "Inactive".to_string()
                },
                join_date: e.created_at,
            })
            .collect(),
        Err(_) => vec![],
    }
}

struct EmployeeSummary {
    total: usize,
    active: usize,
    inactive: usize,
    permanent: usize,
    contract: usize,
    intern: usize,
    departments: Vec<String>,
}

fn compute_summary(employees: &[Employee]) -> EmployeeSummary {
    let mut active = 0; let mut inactive = 0;
    let mut permanent = 0; let mut contract = 0; let mut intern = 0;
    let mut depts: Vec<String> = Vec::new();
    for e in employees {
        if e.status == "Active" { active += 1; } else { inactive += 1; }
        match e.employment_type.as_str() {
            "Permanent" => permanent += 1,
            "Contract" => contract += 1,
            _ => intern += 1,
        }
        if !depts.contains(&e.department) { depts.push(e.department.clone()); }
    }
    EmployeeSummary { total: employees.len(), active, inactive, permanent, contract, intern, departments: depts }
}

#[component]
pub fn EmployeeListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let refresh_counter = use_signal(|| 0u32);
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.with(|c| c.clone());
            fetch_employees(&client).await
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let employees = resource.read().cloned().unwrap_or_default();
    let summary = compute_summary(&employees);

    let columns: Vec<ColumnDef<Employee>> = vec![
        ColumnDef::text("code", "Code", |e: &Employee| e.employee_code.clone())
            .with_width(ColumnWidth::Px(110))
            .with_filter(FilterType::Text),
        ColumnDef::text("name", "Full Name", |e: &Employee| e.full_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("department", "Department", |e: &Employee| e.department.clone())
            .with_width(ColumnWidth::Px(130))
            .with_filter(FilterType::Select {
                options: vec!["Sales".to_string(), "Purchasing".to_string(), "Warehouse".to_string(), "Manufacturing".to_string(), "Admin".to_string(), "Finance".to_string()],
            }),
        ColumnDef::text("designation", "Designation", |e: &Employee| e.designation.clone())
            .with_width(ColumnWidth::Fr(0.8))
            .with_filter(FilterType::Text),
        ColumnDef::text("type", "Employment Type", |e: &Employee| e.employment_type.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Permanent", BadgeColor::Blue), ("Contract", BadgeColor::Yellow), ("Intern", BadgeColor::Purple)],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Permanent".to_string(), "Contract".to_string(), "Intern".to_string()],
            }),
        ColumnDef::text("status", "Status", |e: &Employee| e.status.clone())
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Active", BadgeColor::Green), ("Inactive", BadgeColor::Gray)],
                default_color: BadgeColor::Blue,
            })
            .with_filter(FilterType::Select {
                options: vec!["Active".to_string(), "Inactive".to_string()],
            }),
        ColumnDef::text("join", "Join Date", |e: &Employee| e.join_date.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("email", "Email", |e: &Employee| e.email.clone())
            .with_width(ColumnWidth::Fr(0.8)),
        ColumnDef::text("phone", "Phone", |e: &Employee| e.phone.clone())
            .with_width(ColumnWidth::Px(140)),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_i, e): (usize, Employee)| { nav.push(format!("/crm/employees/{}", e.id)); }
    };

    let on_new = { let nav = navigator.clone(); move |_| { nav.push("/crm/employees/new"); } };
    let on_refresh = { let mut c = refresh_counter.clone(); move |_| c += 1 };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div { h1 { "Employees" } p { class: "page-subtitle", "Manage employee records, departments, and employment types." } }
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
                    div { class: "summary-item", span { class: "summary-label", "Total" } span { class: "summary-value", "{summary.total}" } }
                    div { class: "summary-item summary-ok", span { class: "summary-label", "Active" } span { class: "summary-value", "{summary.active}" } }
                    div { class: "summary-item", span { class: "summary-label", "Permanent" } span { class: "summary-value", "{summary.permanent}" } }
                    div { class: "summary-item", span { class: "summary-label", "Contract" } span { class: "summary-value", "{summary.contract}" } }
                    div { class: "summary-item", span { class: "summary-label", "Interns" } span { class: "summary-value", "{summary.intern}" } }
                    div { class: "summary-item", span { class: "summary-label", "Depts" } span { class: "summary-value", "{summary.departments.len()}" } }
                }
            }
            div { class: "customer-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Employee" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
                }
            }
            DataGrid {
                columns: columns.clone(),
                rows: employees.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true, hoverable: true,
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
