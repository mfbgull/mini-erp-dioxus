//! Supplier List Page — DataGrid-backed list view for supplier management.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct Supplier {
    pub id: i64,
    pub supplier_code: String,
    pub supplier_name: String,
    pub email: String,
    pub phone: String,
    pub city: String,
    pub payment_terms: String,
    pub credit_limit: f64,
    pub current_balance: f64,
    pub status: String,
    pub supplier_type: String,
}



struct SupplierSummary {
    total_count: usize,
    active_count: usize,
    inactive_count: usize,
    total_credit_limit: f64,
    total_balance: f64,
    local_count: usize,
    international_count: usize,
}

fn compute_summary(suppliers: &[Supplier]) -> SupplierSummary {
    let mut active = 0;
    let mut inactive = 0;
    let mut total_cl = 0.0;
    let mut total_bal = 0.0;
    let mut local = 0;
    let mut international = 0;

    for s in suppliers {
        match s.status.as_str() {
            "Active" => active += 1,
            _ => inactive += 1,
        }
        total_cl += s.credit_limit;
        total_bal += s.current_balance;
        match s.supplier_type.as_str() {
            "International" => international += 1,
            _ => local += 1,
        }
    }

    SupplierSummary {
        total_count: suppliers.len(),
        active_count: active,
        inactive_count: inactive,
        total_credit_limit: total_cl,
        total_balance: total_bal,
        local_count: local,
        international_count: international,
    }
}

#[component]
pub fn SupplierListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    let suppliers_resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        let client = use_auth().api;
        let result = client.read().list_suppliers().await;
        match result {
            Ok(models) => models.into_iter().map(|m| Supplier {
                id: m.id,
                supplier_code: m.supplier_code,
                supplier_name: m.supplier_name,
                email: m.email,
                phone: m.phone,
                city: m.address.split(',').next().unwrap_or("").trim().to_string(),
                payment_terms: "Net 30".to_string(),
                credit_limit: 0.0,
                current_balance: 0.0,
                status: if m.is_active { "Active".to_string() } else { "Inactive".to_string() },
                supplier_type: "Local".to_string(),
            }).collect(),
            Err(_) => vec![],
        }
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = suppliers_resource.read().is_none();
    let suppliers = suppliers_resource.read().cloned().unwrap_or_default();

    let summary = compute_summary(&suppliers);

    let columns: Vec<ColumnDef<Supplier>> = vec![
        ColumnDef::text("code", "Code", |s: &Supplier| s.supplier_code.clone())
            .with_width(ColumnWidth::Px(110))
            .with_filter(FilterType::Text),
        ColumnDef::text("name", "Supplier Name", |s: &Supplier| s.supplier_name.clone())
            .with_width(ColumnWidth::Fr(1.2))
            .with_filter(FilterType::Text)
            .with_resizable(true),
        ColumnDef::text("city", "City", |s: &Supplier| s.city.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Select {
                options: vec!["Lahore".to_string(), "Karachi".to_string(), "Rawalpindi".to_string(), "Faisalabad".to_string(), "Sialkot".to_string(), "Multan".to_string(), "Gujranwala".to_string(), "Islamabad".to_string(), "Peshawar".to_string(), "Dubai".to_string()],
            }),
        ColumnDef::text("type", "Type", |s: &Supplier| s.supplier_type.clone())
            .with_width(ColumnWidth::Px(100))
            .with_filter(FilterType::Select {
                options: vec!["Local".to_string(), "International".to_string()],
            })
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Local", BadgeColor::Green), ("International", BadgeColor::Blue)],
                default_color: BadgeColor::Gray,
            }),
        ColumnDef::text("status", "Status", |s: &Supplier| s.status.clone())
            .with_width(ColumnWidth::Px(100))
            .with_filter(FilterType::Select {
                options: vec!["Active".to_string(), "Inactive".to_string()],
            })
            .with_renderer(CellRenderer::Badge {
                color_map: vec![("Active", BadgeColor::Green), ("Inactive", BadgeColor::Gray)],
                default_color: BadgeColor::Blue,
            }),
        ColumnDef::text("terms", "Terms", |s: &Supplier| s.payment_terms.clone())
            .with_width(ColumnWidth::Px(100))
            .with_filter(FilterType::Select {
                options: vec!["Net 15".to_string(), "Net 30".to_string(), "Net 45".to_string(), "Net 60".to_string(), "COD".to_string(), "Due on Receipt".to_string(), "LC 60".to_string()],
            }),
        ColumnDef::text("limit", "Credit Limit", |s: &Supplier| s.credit_limit.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 })
            .with_filter(FilterType::Number),
        ColumnDef::text("balance", "Balance", |s: &Supplier| s.current_balance.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 0 }),
        ColumnDef::text("phone", "Phone", |s: &Supplier| s.phone.clone())
            .with_width(ColumnWidth::Px(140))
            .with_resizable(true),
        ColumnDef::text("email", "Email", |s: &Supplier| s.email.clone())
            .with_width(ColumnWidth::Fr(0.8)),
    ];

    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, s): (usize, Supplier)| {
            nav.push(format!("/crm/suppliers/{}", s.id));
        }
    };

    let on_new = {
        let nav = navigator.clone();
        move |_| { nav.push("/crm/suppliers/new"); } };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| counter += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Suppliers" }
                    p { class: "page-subtitle", "Manage supplier accounts, credit limits, and payment terms." }
                }
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
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Suppliers" }
                        span { class: "summary-value", "{summary.total_count}" }
                    }
                    div { class: "summary-item summary-ok",
                        span { class: "summary-label", "Active" }
                        span { class: "summary-value", "{summary.active_count}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Credit" }
                        span { class: "summary-value summary-amount", "PKR {summary.total_credit_limit:.0}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Balance" }
                        span { class: "summary-value summary-balance", "PKR {summary.total_balance:.0}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Local / Intl" }
                        span { class: "summary-value", "{summary.local_count} / {summary.international_count}" }
                    }
                }
            }

            div { class: "customer-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn toolbar-btn-primary", r#type: "button", disabled: is_loading, onclick: on_new, "＋ New Supplier" }
                    button { class: "toolbar-btn", r#type: "button", disabled: is_loading, onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: suppliers.clone(),
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
