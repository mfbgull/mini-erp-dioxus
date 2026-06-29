//! Quotation Detail Page — View a single quotation with header, KPI cards,
//! line items, action bar, status change, and conversion to invoice.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;
use crate::utils::sleep;
use std::time::Duration;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.qdetail-page { max-width: 960px; margin: 0 auto; }

.qdetail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.qdetail-title-group { display: flex; flex-direction: column; gap: 4px; }
.qdetail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.qdetail-back:hover { text-decoration: underline; }
.qdetail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.qdetail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }

.qdetail-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.qstatus-draft { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.qstatus-sent { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.qstatus-accepted { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.qstatus-rejected { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.qstatus-expired { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.qdetail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.qdetail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.qdetail-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.qdetail-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.qdetail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.qdetail-field { display: flex; flex-direction: column; gap: 3px; }
.qdetail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.qdetail-field-value { font-size: 14px; color: var(--text-primary); }

.qdetail-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }

.qdetail-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.qdetail-items-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.qdetail-items-table thead th.text-right { text-align: right; }
.qdetail-items-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.qdetail-items-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.qdetail-items-table tbody tr:last-child td { border-bottom: none; }
.qdetail-items-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.qdetail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.qdetail-actions-left, .qdetail-actions-right { display: flex; align-items: center; gap: 8px; }

.qdetail-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.qdetail-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: qd-spin 0.8s linear infinite; }
@keyframes qd-spin { to { transform: rotate(360deg); } }

@media (max-width: 768px) {
    .qdetail-header { flex-direction: column; }
    .qdetail-title-row { flex-direction: column; align-items: flex-start; }
    .qdetail-kpis { grid-template-columns: 1fr 1fr; }
    .qdetail-info-grid { grid-template-columns: 1fr; }
    .qdetail-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct QdetailLineItem {
    line_no: i32,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    discount: f64,
    tax_rate: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct QuotationDetail {
    id: i64,
    quotation_no: String,
    customer_name: String,
    customer_code: String,
    date: String,
    valid_until: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    notes: String,
    items: Vec<QdetailLineItem>,
}

fn mock_quotation_detail(id: i64) -> Option<QuotationDetail> {
    let data = vec![
        QuotationDetail {
            id: 1,
            quotation_no: "QOT-2026-0001".to_string(),
            customer_name: "Alpha Traders".to_string(),
            customer_code: "CUST-001".to_string(),
            date: "2026-06-01".to_string(),
            valid_until: "2026-07-01".to_string(),
            status: "Draft".to_string(),
            subtotal: 159_250.00,
            discount_percent: 5.0,
            discount_amount: 7_962.50,
            tax_rate: 16.0,
            tax_amount: 24_206.00,
            total: 156_000.00,
            notes: "Quotation valid for 30 days.".to_string(),
            items: vec![
                QdetailLineItem { line_no: 1, item_code: "ITM-0001".to_string(), item_name: "Premium Widget Alpha".to_string(), quantity: 50.0, unit_price: 1500.0, discount: 5.0, tax_rate: 16.0, net_amount: 71_250.00 },
                QdetailLineItem { line_no: 2, item_code: "ITM-0003".to_string(), item_name: "Steel Rod 12mm x 6m".to_string(), quantity: 200.0, unit_price: 350.0, discount: 0.0, tax_rate: 16.0, net_amount: 70_000.00 },
                QdetailLineItem { line_no: 3, item_code: "ITM-0005".to_string(), item_name: "Rubber Gasket Set".to_string(), quantity: 100.0, unit_price: 180.0, discount: 0.0, tax_rate: 16.0, net_amount: 18_000.00 },
            ],
        },
        QuotationDetail {
            id: 3,
            quotation_no: "QOT-2026-0003".to_string(),
            customer_name: "Gamma Supplies".to_string(),
            customer_code: "CUST-003".to_string(),
            date: "2026-06-10".to_string(),
            valid_until: "2026-07-10".to_string(),
            status: "Accepted".to_string(),
            subtotal: 234_500.00,
            discount_percent: 10.0,
            discount_amount: 23_450.00,
            tax_rate: 16.0,
            tax_amount: 33_768.00,
            total: 244_818.00,
            notes: "Accepted by customer. Ready for invoicing.".to_string(),
            items: vec![
                QdetailLineItem { line_no: 1, item_code: "ITM-0004".to_string(), item_name: "Hydraulic Pump HPD-200".to_string(), quantity: 10.0, unit_price: 12500.0, discount: 5.0, tax_rate: 16.0, net_amount: 118_750.00 },
                QdetailLineItem { line_no: 2, item_code: "ITM-0006".to_string(), item_name: "Copper Wire 2.5mm (100m)".to_string(), quantity: 500.0, unit_price: 45.0, discount: 0.0, tax_rate: 16.0, net_amount: 22_500.00 },
                QdetailLineItem { line_no: 3, item_code: "ITM-0007".to_string(), item_name: "LED Panel Light 24W".to_string(), quantity: 200.0, unit_price: 185.0, discount: 0.0, tax_rate: 16.0, net_amount: 37_000.00 },
                QdetailLineItem { line_no: 4, item_code: "ITM-0008".to_string(), item_name: "Packaging Box 40x30x20cm".to_string(), quantity: 1000.0, unit_price: 1.2, discount: 0.0, tax_rate: 16.0, net_amount: 1_200.00 },
            ],
        },
    ];
    data.into_iter().find(|q| q.id == id).or_else(|| Some(QuotationDetail {
        id,
        quotation_no: format!("QOT-2026-{:04}", id),
        customer_name: "Sample Customer".to_string(),
        customer_code: format!("CUST-{:03}", id),
        date: "2026-06-01".to_string(),
        valid_until: "2026-07-01".to_string(),
        status: "Draft".to_string(),
        subtotal: 25_000.00,
        discount_percent: 0.0,
        discount_amount: 0.0,
        tax_rate: 16.0,
        tax_amount: 4_000.00,
        total: 29_000.00,
        notes: String::new(),
        items: vec![QdetailLineItem { line_no: 1, item_code: "ITM-0001".to_string(), item_name: "Sample Item".to_string(), quantity: 10.0, unit_price: 2500.0, discount: 0.0, tax_rate: 16.0, net_amount: 25_000.00 }],
    }))
}

fn qstatus_class(status: &str) -> &'static str {
    match status {
        "Draft" => "qstatus-draft",
        "Sent" => "qstatus-sent",
        "Accepted" => "qstatus-accepted",
        "Rejected" => "qstatus-rejected",
        "Expired" => "qstatus-expired",
        _ => "qstatus-draft",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let resource = use_resource(move || {
        let fetch_id = id.clone();
        async move {
            sleep(Duration::from_millis(500)).await;
            let parsed = fetch_id.parse::<i64>().unwrap_or(0);
            mock_quotation_detail(parsed)
        }
    });

    let is_loading = resource.read().is_none();
    let q_opt = resource.read().as_ref().cloned().flatten();
    let mut show_delete_modal = use_signal(|| false);

    
    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page qdetail-page",
                div { class: "qdetail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading..." }
                }
            }
        };
    }
    if q_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page qdetail-page",
                div { class: "qdetail-loading",
                    div { style: "font-size: 40px;", "📋" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Quotation Not Found" }
                    p { "No record found." }
                }
            }
        };
    }
    let q = q_opt.as_ref().unwrap();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page qdetail-page",
            div { class: "empty-state",
                p { "Quotation detail view — coming soon" }
            }
        }
    }
}