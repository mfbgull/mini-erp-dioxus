//! Employee List Page — DataGrid-backed list view for employee management.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
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

async fn fetch_employees() -> Vec<Employee> {
    crate::utils::sleep(std::time::Duration::from_millis(600)).await;
    sample_employees_data()
}

pub fn sample_employees_data() -> Vec<Employee> {
    vec![
        Employee { id: 1, employee_code: "EMP-0001".to_string(), full_name: "Ahmad Raza".to_string(), email: "ahmad.raza@mierp.pk".to_string(), phone: "+92 300 111 0001".to_string(), department: "Sales".to_string(), designation: "Sales Manager".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2020-03-15".to_string() },
        Employee { id: 2, employee_code: "EMP-0002".to_string(), full_name: "Fatima Khan".to_string(), email: "fatima.khan@mierp.pk".to_string(), phone: "+92 300 111 0002".to_string(), department: "Finance".to_string(), designation: "Chief Accountant".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2019-07-01".to_string() },
        Employee { id: 3, employee_code: "EMP-0003".to_string(), full_name: "Usman Ali".to_string(), email: "usman.ali@mierp.pk".to_string(), phone: "+92 300 111 0003".to_string(), department: "Purchasing".to_string(), designation: "Procurement Officer".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2021-01-10".to_string() },
        Employee { id: 4, employee_code: "EMP-0004".to_string(), full_name: "Sana Tariq".to_string(), email: "sana.tariq@mierp.pk".to_string(), phone: "+92 300 111 0004".to_string(), department: "Manufacturing".to_string(), designation: "Production Supervisor".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2018-11-20".to_string() },
        Employee { id: 5, employee_code: "EMP-0005".to_string(), full_name: "Bilal Ahmed".to_string(), email: "bilal.ahmed@mierp.pk".to_string(), phone: "+92 300 111 0005".to_string(), department: "Warehouse".to_string(), designation: "Warehouse Manager".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2020-06-05".to_string() },
        Employee { id: 6, employee_code: "EMP-0006".to_string(), full_name: "Hira Javed".to_string(), email: "hira.javed@mierp.pk".to_string(), phone: "+92 300 111 0006".to_string(), department: "Admin".to_string(), designation: "HR Assistant".to_string(), employment_type: "Contract".to_string(), status: "Active".to_string(), join_date: "2022-04-12".to_string() },
        Employee { id: 7, employee_code: "EMP-0007".to_string(), full_name: "Kamran Hassan".to_string(), email: "kamran.hassan@mierp.pk".to_string(), phone: "+92 300 111 0007".to_string(), department: "Sales".to_string(), designation: "Sales Representative".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2021-09-01".to_string() },
        Employee { id: 8, employee_code: "EMP-0008".to_string(), full_name: "Nadia Shah".to_string(), email: "nadia.shah@mierp.pk".to_string(), phone: "+92 300 111 0008".to_string(), department: "Finance".to_string(), designation: "Accounts Clerk".to_string(), employment_type: "Contract".to_string(), status: "Inactive".to_string(), join_date: "2023-02-15".to_string() },
        Employee { id: 9, employee_code: "EMP-0009".to_string(), full_name: "Omar Farooq".to_string(), email: "omar.farooq@mierp.pk".to_string(), phone: "+92 300 111 0009".to_string(), department: "Manufacturing".to_string(), designation: "Machine Operator".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2017-05-20".to_string() },
        Employee { id: 10, employee_code: "EMP-0010".to_string(), full_name: "Zainab Bibi".to_string(), email: "zainab.bibi@mierp.pk".to_string(), phone: "+92 300 111 0010".to_string(), department: "Admin".to_string(), designation: "Office Assistant".to_string(), employment_type: "Intern".to_string(), status: "Active".to_string(), join_date: "2025-06-01".to_string() },
        Employee { id: 11, employee_code: "EMP-0011".to_string(), full_name: "Tariq Mahmood".to_string(), email: "tariq.mahmood@mierp.pk".to_string(), phone: "+92 300 111 0011".to_string(), department: "Purchasing".to_string(), designation: "Buyer".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2019-12-01".to_string() },
        Employee { id: 12, employee_code: "EMP-0012".to_string(), full_name: "Rashid Minhas".to_string(), email: "rashid.minhas@mierp.pk".to_string(), phone: "+92 300 111 0012".to_string(), department: "Warehouse".to_string(), designation: "Store Keeper".to_string(), employment_type: "Contract".to_string(), status: "Inactive".to_string(), join_date: "2023-08-15".to_string() },
        Employee { id: 13, employee_code: "EMP-0013".to_string(), full_name: "Asma Yousuf".to_string(), email: "asma.yousuf@mierp.pk".to_string(), phone: "+92 300 111 0013".to_string(), department: "Manufacturing".to_string(), designation: "Quality Inspector".to_string(), employment_type: "Permanent".to_string(), status: "Active".to_string(), join_date: "2020-10-10".to_string() },
        Employee { id: 14, employee_code: "EMP-0014".to_string(), full_name: "Fawad Ahmed".to_string(), email: "fawad.ahmed@mierp.pk".to_string(), phone: "+92 300 111 0014".to_string(), department: "Sales".to_string(), designation: "Sales Trainee".to_string(), employment_type: "Intern".to_string(), status: "Active".to_string(), join_date: "2026-01-05".to_string() },
        Employee { id: 15, employee_code: "EMP-0015".to_string(), full_name: "Ghulam Mustafa".to_string(), email: "ghulam.mustafa@mierp.pk".to_string(), phone: "+92 300 111 0015".to_string(), department: "Finance".to_string(), designation: "Tax Specialist".to_string(), employment_type: "Contract".to_string(), status: "Active".to_string(), join_date: "2024-03-01".to_string() },
    ]
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
    let refresh_counter = use_signal(|| 0u32);
    let resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        fetch_employees().await
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
