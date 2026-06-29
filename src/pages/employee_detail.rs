//! Employee Detail Page — Single-section detail view for an employee.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.emp-detail-page { max-width: 900px; margin: 0 auto; }
.emp-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.emp-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.emp-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.emp-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.emp-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.emp-detail-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted); padding: 2px 8px; border-radius: 4px; }
.emp-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; }
.emp-status-active { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.emp-status-inactive { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.emp-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.emp-section { background: #fff; border: 1px solid var(--border-color); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.emp-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color); }
.emp-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.emp-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.emp-field { display: flex; flex-direction: column; gap: 3px; }
.emp-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.emp-field-value { font-size: 14px; color: var(--text-primary); }
.emp-actions { display: flex; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color); }
.emp-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.emp-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: emp-spin 0.8s linear infinite; }
@keyframes emp-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .emp-info-grid { grid-template-columns: 1fr; } }
"##;

fn type_badge_class(et: &str) -> &'static str {
    match et {
        "Permanent" => "customer-table-badge-blue",
        "Contract" => "customer-table-badge-yellow",
        "Intern" => "customer-table-badge-gray",
        _ => "customer-table-badge-gray",
    }
}

fn status_class(s: &str) -> &'static str {
    match s { "Active" => "emp-status-active", _ => "emp-status-inactive" }
}

#[component]
pub fn EmployeeDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let resource = use_resource(move || {
        let id = id.clone();
        async move {
            crate::utils::sleep(std::time::Duration::from_millis(600)).await;
            let employees = crate::pages::employee_list::sample_employees_data();
            let parsed = id.parse::<i64>().unwrap_or(0);
            employees.into_iter().find(|e| e.id == parsed)
        }
    });

    let snap = resource.read();
    let is_loading = snap.is_none();
    let emp_opt = snap.as_ref().and_then(|e| e.clone());
    let mut show_delete_modal = use_signal(|| false);

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page emp-detail-page",
            if is_loading {
                div { class: "emp-loading",
                    div { class: "loading-spinner" }
                    span { "Loading employee details…" }
                }
            } else if emp_opt.is_none() {
                div { class: "emp-loading",
                    h2 { style: "margin: 0; color: var(--text-primary);", "Employee Not Found" }
                    p { "No employee with ID \"{id_display}\" was found." }
                }
            } else {{
                let emp = emp_opt.as_ref().unwrap();

                rsx! {
                    div { class: "emp-detail-header",
                        div { class: "emp-detail-title-group",
                            Button { class: Some("emp-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: move |_| { navigator.push("/crm/employees"); }, "← Back to Employees" }
                            div { class: "emp-detail-title-row",
                                h1 { "{emp.full_name}" }
                                span { class: "emp-detail-code", "{emp.employee_code}" }
                                span { class: "emp-status-badge {status_class(&emp.status)}",
                                    if emp.status == "Active" { "✓ Active" } else { "— Inactive" }
                                }
                            }
                        }
                    }

                    div { class: "emp-detail-kpis",
                        StatCard {
                            title: "Department".to_string(),
                            value: emp.department.clone(),
                            variant: StatCardVariant::Primary,
                            icon: Some("🏢".to_string()),
                        }
                        StatCard {
                            title: "Designation".to_string(),
                            value: emp.designation.clone(),
                            variant: StatCardVariant::Default,
                            icon: Some("👤".to_string()),
                        }
                        StatCard {
                            title: "Tasks Assigned".to_string(),
                            value: "0".to_string(),
                            variant: StatCardVariant::Primary,
                            icon: Some("📋".to_string()),
                            footer: Some("No tasks yet".to_string()),
                        }
                        StatCard {
                            title: "Attendance %".to_string(),
                            value: "98%".to_string(),
                            variant: StatCardVariant::Success,
                            icon: Some("📊".to_string()),
                            footer: Some("This month".to_string()),
                        }
                    }

                    div { class: "emp-section",
                        div { class: "emp-section-header", h2 { "Employee Information" } }
                        div { class: "emp-info-grid",
                            div { class: "emp-field", span { class: "emp-field-label", "Employee Code" } span { class: "emp-field-value", "{emp.employee_code}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Full Name" } span { class: "emp-field-value", "{emp.full_name}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Email" } span { class: "emp-field-value", "{emp.email}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Phone" } span { class: "emp-field-value", "{emp.phone}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Department" } span { class: "emp-field-value", "{emp.department}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Designation" } span { class: "emp-field-value", "{emp.designation}" } }
                            div { class: "emp-field",
                                span { class: "emp-field-label", "Employment Type" }
                                span { class: "customer-table-badge {type_badge_class(&emp.employment_type)}", "{emp.employment_type}" }
                            }
                            div { class: "emp-field", span { class: "emp-field-label", "Status" } span { class: "emp-field-value", "{emp.status}" } }
                            div { class: "emp-field", span { class: "emp-field-label", "Join Date" } span { class: "emp-field-value", "{emp.join_date}" } }
                        }
                    }

                    div { class: "emp-actions",
                        Button { variant: ButtonVariant::Primary, onclick: { let mut t = toast.clone(); move |_| t.info("Coming Soon", "Employee editing coming soon.") }, icon: Some("✏️".to_string()), "Edit Employee" }
                        Button { variant: ButtonVariant::Ghost, onclick: move |_| show_delete_modal.set(true), icon: Some("🗑️".to_string()), "Delete" }
                    }

                    Modal {
                        is_open: show_delete_modal,
                        title: Some("Delete Employee".to_string()),
                        size: ModalSize::Sm,
                        close_on_backdrop: true, close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: move |_| show_delete_modal.set(false), "Cancel" }
                            Button { variant: ButtonVariant::Danger, onclick: { let mut t = toast.clone(); move |_| { show_delete_modal.set(false); t.success("Deleted", "Employee deleted."); navigator.push("/crm/employees"); } }, "Delete Employee" }
                        },
                        div {
                            p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;", "Delete {emp.full_name}?" }
                            p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;", "This action cannot be undone." }
                        }
                    }
                }
            }}
        }
    }
}
