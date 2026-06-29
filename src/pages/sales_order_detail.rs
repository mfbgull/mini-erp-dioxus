//! Sales Order Detail Page — View a single sales order with header, KPI cards,
//! line items, action bar, and conversion to invoice.

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
.sodetail-page { max-width: 960px; margin: 0 auto; }

.sodetail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.sodetail-title-group { display: flex; flex-direction: column; gap: 4px; }
.sodetail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.sodetail-back:hover { text-decoration: underline; }
.sodetail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.sodetail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }

.sodetail-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.sostatus-draft { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.sostatus-confirmed { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.sostatus-processing { background: rgba(128, 0, 128, 0.1); color: #800080; }
.sostatus-shipped { background: rgba(0, 188, 212, 0.1); color: #00bcd4; }
.sostatus-delivered { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.sostatus-cancelled { background: rgba(108, 117, 125, 0.1); color: #6c757d; }

.sodetail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.sodetail-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.sodetail-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sodetail-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.sodetail-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.sodetail-field { display: flex; flex-direction: column; gap: 3px; }
.sodetail-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.sodetail-field-value { font-size: 14px; color: var(--text-primary); }

.sodetail-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }

.sodetail-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.sodetail-items-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.sodetail-items-table thead th.text-right { text-align: right; }
.sodetail-items-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.sodetail-items-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.sodetail-items-table tbody tr:last-child td { border-bottom: none; }
.sodetail-items-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.sodetail-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.sodetail-actions-left, .sodetail-actions-right { display: flex; align-items: center; gap: 8px; }

.sodetail-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.sodetail-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: sod-spin 0.8s linear infinite; }
@keyframes sod-spin { to { transform: rotate(360deg); } }

@media (max-width: 768px) {
    .sodetail-header { flex-direction: column; }
    .sodetail-title-row { flex-direction: column; align-items: flex-start; }
    .sodetail-kpis { grid-template-columns: 1fr 1fr; }
    .sodetail-info-grid { grid-template-columns: 1fr; }
    .sodetail-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct SoLineItem {
    line_no: i32,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    net_amount: f64,
}

#[derive(Clone, Debug)]
struct SalesOrderDetail {
    id: i64,
    order_no: String,
    customer_name: String,
    customer_code: String,
    order_date: String,
    delivery_date: String,
    status: String,
    subtotal: f64,
    discount_percent: f64,
    discount_amount: f64,
    tax_rate: f64,
    tax_amount: f64,
    total: f64,
    notes: String,
    items: Vec<SoLineItem>,
}

fn mock_order_detail(id: i64) -> Option<SalesOrderDetail> {
    let data = vec![
        SalesOrderDetail {
            id: 1,
            order_no: "SO-2026-0001".to_string(),
            customer_name: "Alpha Traders".to_string(),
            customer_code: "CUST-001".to_string(),
            order_date: "2026-06-01".to_string(),
            delivery_date: "2026-06-15".to_string(),
            status: "Confirmed".to_string(),
            subtotal: 125_400.00,
            discount_percent: 5.0,
            discount_amount: 6_270.00,
            tax_rate: 16.0,
            tax_amount: 19_060.80,
            total: 138_190.80,
            notes: "Delivery requested before 10 AM.".to_string(),
            items: vec![
                SoLineItem { line_no: 1, item_code: "ITM-0001".to_string(), item_name: "Premium Widget Alpha".to_string(), quantity: 50.0, unit_price: 1500.0, net_amount: 71_250.00 },
                SoLineItem { line_no: 2, item_code: "ITM-0003".to_string(), item_name: "Steel Rod 12mm x 6m".to_string(), quantity: 100.0, unit_price: 350.0, net_amount: 35_000.00 },
                SoLineItem { line_no: 3, item_code: "ITM-0005".to_string(), item_name: "Rubber Gasket Set".to_string(), quantity: 200.0, unit_price: 95.0, net_amount: 19_000.00 },
            ],
        },
    ];
    data.into_iter().find(|o| o.id == id).or_else(|| Some(SalesOrderDetail {
        id,
        order_no: format!("SO-2026-{:04}", id),
        customer_name: "Sample Customer".to_string(),
        customer_code: format!("CUST-{:03}", id),
        order_date: "2026-06-01".to_string(),
        delivery_date: "2026-06-15".to_string(),
        status: "Draft".to_string(),
        subtotal: 30_000.00,
        discount_percent: 0.0,
        discount_amount: 0.0,
        tax_rate: 16.0,
        tax_amount: 4_800.00,
        total: 34_800.00,
        notes: String::new(),
        items: vec![SoLineItem { line_no: 1, item_code: "ITM-0001".to_string(), item_name: "Sample Item".to_string(), quantity: 10.0, unit_price: 3000.0, net_amount: 30_000.00 }],
    }))
}

fn sostatus_class(status: &str) -> &'static str {
    match status {
        "Draft" => "sostatus-draft",
        "Confirmed" => "sostatus-confirmed",
        "Processing" => "sostatus-processing",
        "Shipped" => "sostatus-shipped",
        "Delivered" => "sostatus-delivered",
        "Cancelled" => "sostatus-cancelled",
        _ => "sostatus-draft",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesOrderDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();

    let resource = use_resource(move || {
        let fetch_id = id.clone();
        async move {
            sleep(Duration::from_millis(500)).await;
            let parsed = fetch_id.parse::<i64>().unwrap_or(0);
            mock_order_detail(parsed)
        }
    });

    let is_loading = resource.read().is_none();
    let so_opt = resource.read().as_ref().cloned().flatten();
    let mut show_delete_modal = use_signal(|| false);

    
    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sodetail-page",
                div { class: "sodetail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading..." }
                }
            }
        };
    }
    if so_opt.is_none() {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sodetail-page",
                div { class: "sodetail-loading",
                    div { style: "font-size: 40px;", "📦" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Sales Order Not Found" }
                    p { "No record found." }
                }
            }
        };
    }
    let so = so_opt.as_ref().unwrap();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page sodetail-page",
            div { class: "empty-state",
                p { "Sales order detail view — coming soon" }
            }
        }
    }
}