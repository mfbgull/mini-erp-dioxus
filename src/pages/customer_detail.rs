//! Customer Detail Page — A tabbed detail view showing full customer info,
//! invoices, payments, and running-balance ledger.
//!
//! Tabs:
//! - **Overview** — KPI cards, customer details, recent activity
//! - **Invoices** — Mini invoice list table
//! - **Payments** — Payment history table
//! - **Ledger** — Running balance ledger table

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::models;
use std::collections::HashMap;
use crate::pages::customer_list::Customer;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.customer-detail-page { max-width: 1000px; margin: 0 auto; }
.customer-detail-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; gap: 16px; flex-wrap: wrap; }
.customer-detail-title-group { display: flex; flex-direction: column; gap: 4px; }
.customer-detail-back { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent, #4a90d9); text-decoration: none; margin-bottom: 6px; cursor: pointer; background: none; border: none; padding: 0; }
.customer-detail-back:hover { text-decoration: underline; }
.customer-detail-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.customer-detail-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.customer-detail-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.customer-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; line-height: 1; }
.customer-status-active { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.customer-status-inactive { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.customer-status-over-limit { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.customer-detail-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.customer-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 2px solid var(--border-color, #e0e0e0); overflow-x: auto; }
.customer-tab { display: inline-flex; align-items: center; gap: 6px; padding: 10px 18px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border: none; background: none; white-space: nowrap; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.15s ease; }
.customer-tab:hover { color: var(--text-primary); background: rgba(74, 144, 217, 0.04); }
.customer-tab-active { color: var(--accent, #4a90d9); border-bottom-color: var(--accent, #4a90d9); font-weight: 600; }
.customer-tab-count { font-size: 11px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 0 6px; border-radius: 8px; line-height: 18px; }
.customer-tab-active .customer-tab-count { background: rgba(74, 144, 217, 0.1); color: var(--accent, #4a90d9); }
.customer-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.customer-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.customer-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.customer-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.customer-field { display: flex; flex-direction: column; gap: 3px; }
.customer-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.customer-field-value { font-size: 14px; color: var(--text-primary); }
.customer-field-value.monospace { font-family: monospace; font-size: 13px; }
.customer-field-value.text-success { color: #28a745; }
.customer-field-value.text-danger { color: #dc3545; }
.customer-field-value.text-warning { color: #d4a017; }
.customer-notes { font-size: 13px; color: var(--text-secondary); line-height: 1.6; padding: 12px; background: var(--bg-muted, #f5f5f5); border-radius: 6px; margin: 0; }
.customer-activity { display: flex; flex-direction: column; gap: 10px; }
.activity-item { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border-radius: 6px; background: var(--bg-muted, #f5f5f5); }
.activity-icon { width: 32px; height: 32px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 14px; flex-shrink: 0; }
.activity-icon-green { background: rgba(40, 167, 69, 0.12); }
.activity-icon-blue { background: rgba(74, 144, 217, 0.12); }
.activity-icon-yellow { background: rgba(255, 193, 7, 0.15); }
.activity-icon-red { background: rgba(220, 53, 69, 0.12); }
.activity-content { flex: 1; font-size: 13px; line-height: 1.5; }
.activity-content strong { font-weight: 600; color: var(--text-primary); }
.activity-date { font-size: 11px; color: var(--text-secondary); white-space: nowrap; }
.customer-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.customer-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.customer-table thead th.text-right { text-align: right; }
.customer-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.customer-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.customer-table tbody td.text-success { color: #28a745; }
.customer-table tbody td.text-danger { color: #dc3545; }
.customer-table tbody td.text-warning { color: #d4a017; }
.customer-table tbody tr:last-child td { border-bottom: none; }
.customer-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.customer-table-badge { display: inline-flex; align-items: center; padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; }
.customer-table-badge-green { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.customer-table-badge-yellow { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.customer-table-badge-blue { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.customer-table-badge-red { background: rgba(220, 53, 69, 0.12); color: #dc3545; }
.customer-table-badge-gray { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.customer-table-empty { text-align: center; padding: 30px 20px; color: var(--text-secondary); font-size: 14px; }
.customer-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); flex-wrap: wrap; }
.customer-actions-left, .customer-actions-right { display: flex; align-items: center; gap: 8px; }
.customer-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.customer-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: customer-spin 0.8s linear infinite; }
@keyframes customer-spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) {
    .customer-detail-header { flex-direction: column; }
    .customer-detail-title-row { flex-direction: column; align-items: flex-start; }
    .customer-detail-kpis { grid-template-columns: 1fr 1fr; }
    .customer-info-grid { grid-template-columns: 1fr; }
    .customer-tab { padding: 10px 12px; font-size: 12px; }
    .customer-actions { flex-direction: column; align-items: stretch; }
}
"##;

// ============================================================================
// Local Data Models
// ============================================================================

#[derive(Clone, Debug)]
struct InvoiceItem {
    id: i64,
    invoice_no: String,
    date: String,
    due_date: String,
    status: String,
    total: f64,
    paid: f64,
    balance: f64,
}

#[derive(Clone, Debug)]
struct PaymentItem {
    id: i64,
    payment_no: String,
    date: String,
    amount: f64,
    method: String,
    reference: String,
}

#[derive(Clone, Debug)]
struct LedgerEntry {
    date: String,
    entry_type: String,
    reference: String,
    debit: f64,
    credit: f64,
    balance: f64,
    notes: String,
}

// ============================================================================
// Helpers
// ============================================================================

fn status_class(status: &str) -> &'static str {
    match status {
        "Active" => "customer-status-active",
        "Inactive" => "customer-status-inactive",
        "Over Limit" => "customer-status-over-limit",
        _ => "customer-status-active",
    }
}

fn badge_class(status: &str) -> &'static str {
    match status {
        "Paid" => "customer-table-badge-green",
        "Unpaid" => "customer-table-badge-yellow",
        "Partially Paid" => "customer-table-badge-blue",
        "Overdue" => "customer-table-badge-red",
        _ => "customer-table-badge-gray",
    }
}

/// Generate recent activity items from invoices and ledger entries.
fn generate_activity_items(invoices: &[InvoiceItem], ledger: &[LedgerEntry]) -> Vec<(String, String, String)> {
    let mut items: Vec<(String, String, String)> = Vec::new();

    // Latest invoices
    for inv in invoices.iter().rev().take(3) {
        let icon = match inv.status.as_str() {
            "Overdue" => "activity-icon-red",
            "Paid" => "activity-icon-green",
            "Partially Paid" => "activity-icon-yellow",
            _ => "activity-icon-blue",
        };
        items.push((
            format!("Invoice <strong>{}</strong> — PKR {:.0}", inv.invoice_no, inv.total),
            icon.to_string(),
            inv.date.clone(),
        ));
    }

    // Payment entries from ledger
    for entry in ledger.iter() {
        if items.len() >= 5 {
            break;
        }
        if entry.entry_type == "PAYMENT" {
            items.push((
                format!("Payment of <strong>PKR {:.0}</strong> received", entry.credit),
                "activity-icon-blue".to_string(),
                entry.date.clone(),
            ));
        }
    }

    if items.is_empty() {
        items.push(("No recent activity".to_string(), "activity-icon-blue".to_string(), String::new()));
    }

    items
}

// ============================================================================
// Component
// ============================================================================

/// The Customer Detail page — a tabbed view with Overview, Invoices, Payments, and Ledger.
#[component]
pub fn CustomerDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let id_display = id.clone();
    let api = use_auth().api;

    // ── Tab state ──
    let active_tab = use_signal(|| 0usize);
    let mut show_delete_modal = use_signal(|| false);
    let show_payment_modal = use_signal(|| false);
    let is_saving = use_signal(|| false);
    // invoice_id -> amount-to-pay input (as string for editable fields)
    let pay_alloc = use_signal(HashMap::<i64, String>::new);
    let pay_method = use_signal(|| "Cash".to_string());
    let pay_reference = use_signal(String::new);
    let pay_notes = use_signal(String::new);

    // ── Combined data fetch ──
    let customer_resource = use_resource(move || {
        let api = api.clone();
        let id = id.clone();
        async move {
            let parsed = id.parse::<i64>().ok()?;
            let client = api.read().clone();

            // Fetch customer
            let server_customer = client.get_customer(parsed).await.ok()?;

            // Fetch invoices and filter by customer_id
            // ponytail: filters client-side; add ?customer_id= query param server-side if pagination matters
            let all_invoices = client.list_invoices().await.unwrap_or_default();
            let customer_invoices: Vec<models::Invoice> = all_invoices
                .into_iter()
                .filter(|inv| inv.customer_id == parsed)
                .collect();

            // Fetch ledger
            let ledger_entries = client.get_customer_ledger(parsed).await.unwrap_or_default();

            // Fetch payments
            let payment_entries = client.get_customer_payments(parsed).await.unwrap_or_default();

            Some((server_customer, customer_invoices, ledger_entries, payment_entries))
        }
    });

    // ── Extract & map data, then drop the read guard ──
    let (customer_opt, invoices, payments, ledger, util, balance_remaining, overdue_count, unpaid_count) = {
        let snapshot = customer_resource.read();

        let mut invoices: Vec<InvoiceItem> = Vec::new();
        let mut payments: Vec<PaymentItem> = Vec::new();
        let mut ledger: Vec<LedgerEntry> = Vec::new();
        let mut util = 0.0_f64;
        let mut balance_remaining = 0.0_f64;
        let mut overdue_count = 0usize;
        let mut unpaid_count = 0usize;
        let mut customer_opt: Option<Customer> = None;

        // ponytail: outer Option=loading, inner Option=found/not-found
        if let Some(Some((sc, invs, ledgers, pmts))) = snapshot.as_ref() {
            // Compute invoice-level metrics
            let total_invoiced: f64 = invs.iter().map(|i| i.total_amount).sum();
            let total_paid: f64 = invs.iter().map(|i| i.paid_amount).sum();
            let overdue = invs.iter().filter(|i| i.status == "Overdue").count();
            let unpaid = invs.iter().filter(|i| i.status != "Paid").count();

            // Map server Customer -> local Customer
            let city = sc
                .billing_address
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let last_inv = invs
                .first()
                .map(|i| i.invoice_date.clone())
                .unwrap_or_default();
            let status = if sc.current_balance > sc.credit_limit && sc.credit_limit > 0.0 {
                "Over Limit".to_string()
            } else if sc.is_active {
                "Active".to_string()
            } else {
                "Inactive".to_string()
            };

            let customer = Customer {
                id: sc.id,
                customer_code: sc.customer_code.clone(),
                customer_name: sc.customer_name.clone(),
                email: sc.email.clone(),
                phone: sc.phone.clone(),
                city,
                payment_terms: sc.payment_terms.clone(),
                credit_limit: sc.credit_limit,
                current_balance: sc.current_balance,
                opening_balance: sc.opening_balance,
                total_invoiced,
                total_paid,
                last_invoice_date: last_inv,
                status,
                customer_type: "Standard".to_string(),
                notes: String::new(),
            };

            util = if customer.credit_limit > 0.0 {
                (customer.current_balance / customer.credit_limit) * 100.0
            } else {
                0.0
            };
            balance_remaining = customer.credit_limit - customer.current_balance;
            overdue_count = overdue;
            unpaid_count = unpaid;

            // Map invoices
            invoices = invs
                .iter()
                .map(|inv| InvoiceItem {
                    id: inv.id,
                    invoice_no: inv.invoice_no.clone(),
                    date: inv.invoice_date.clone(),
                    due_date: inv.due_date.clone(),
                    status: inv.status.clone(),
                    total: inv.total_amount,
                    paid: inv.paid_amount,
                    balance: inv.balance_amount,
                })
                .collect();

            payments = pmts
                .iter()
                .map(|p| PaymentItem {
                    id: p.id,
                    payment_no: p.payment_no.clone(),
                    date: p.payment_date.clone(),
                    amount: p.amount,
                    method: p.payment_method.clone(),
                    reference: p.reference.clone().unwrap_or_default(),
                })
                .collect();

            // Map ledger entries
            ledger = ledgers
                .iter()
                .map(|e| LedgerEntry {
                    date: e.transaction_date.clone(),
                    entry_type: e.transaction_type.clone(),
                    reference: e.reference_no.clone(),
                    debit: e.debit,
                    credit: e.credit,
                    balance: e.balance,
                    notes: String::new(),
                })
                .collect();

            customer_opt = Some(customer);
        }

        (
            customer_opt,
            invoices,
            payments,
            ledger,
            util,
            balance_remaining,
            overdue_count,
            unpaid_count,
        )
    };

    // ── Activity items (derived from fetched data) ──
    let activity_items = generate_activity_items(&invoices, &ledger);

    // ── Derived state ──
    let tabs = ["Overview", "Invoices", "Payments", "Ledger"];

    let on_back_not_found = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/customers");
        }
    };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page customer-detail-page",

            // ── Loading State ──
            if customer_resource.read().is_none() {
                div { class: "customer-loading",
                    div { class: "loading-spinner" }
                    span { "Loading customer details…" }
                }
            }
            // ── Not Found State ──
            else if customer_opt.is_none() {
                div { class: "customer-loading",
                    div { style: "font-size: 40px;", "👤" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Customer Not Found" }
                    p { "No customer with ID \"{id_display}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: on_back_not_found, "← Back to Customers" }
                }
            }
            // ── Detail View ──
            else {
                {{
                    let nav = navigator.clone();
                    let t = toast.clone();
                    let on_back = move |_| { nav.push("/customers"); };
                    let cust_id = customer_opt.as_ref().map(|c| c.id).unwrap_or(0);
                    let on_edit = { let nav2 = nav.clone(); move |_| { nav2.push(format!("/customers/{}/edit", cust_id)); } };
                    let on_new_invoice = { let nav2 = nav.clone(); move |_| { nav2.push("/sales/invoices/new"); } };
                    // Unpaid invoices for this customer (outstanding balance)
                    let unpaid_invoices: Vec<InvoiceItem> = invoices.iter().filter(|i| i.balance > 0.01).cloned().collect();
                    let on_record_payment = {
                        let mut m = show_payment_modal.clone();
                        let mut alloc = pay_alloc.clone();
                        let mut method = pay_method.clone();
                        let mut reference = pay_reference.clone();
                        let mut notes = pay_notes.clone();
                        move |_| {
                            alloc.set(HashMap::new());
                            method.set("Cash".to_string());
                            reference.set(String::new());
                            notes.set(String::new());
                            m.set(true);
                        }
                    };
                    let cancel_payment = { let mut m = show_payment_modal.clone(); move |_| m.set(false) };
                    let submit_payment = {
                        let api = api.clone();
                        let toast2 = t.clone();
                        let modal = show_payment_modal.clone();
                        let alloc_sig = pay_alloc.clone();
                        let method_sig = pay_method.clone();
                        let reference_sig = pay_reference.clone();
                        let notes_sig = pay_notes.clone();
                        let saving_sig = is_saving.clone();
                        let refresh = customer_resource.clone();
                        let unpaid = unpaid_invoices.clone();
                        move |_| {
                            let mut toast_v = toast2.clone();
                            // Build allocations from entered amounts, clamped to each invoice balance
                            let alloc_map = alloc_sig.read().clone();
                            let mut allocations: Vec<serde_json::Value> = Vec::new();
                            let mut total = 0.0_f64;
                            for inv in unpaid.iter() {
                                let amt = alloc_map.get(&inv.id).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                                if amt <= 0.0 { continue; }
                                if amt > inv.balance + 0.01 {
                                    toast_v.error("Validation Error", &format!("Amount for {} exceeds its balance.", inv.invoice_no));
                                    return;
                                }
                                allocations.push(serde_json::json!({ "invoice_id": inv.id, "amount": amt }));
                                total += amt;
                            }
                            if allocations.is_empty() || total <= 0.0 {
                                toast_v.error("Validation Error", "Enter a payment amount against at least one invoice.");
                                return;
                            }
                            let mut saving_v = saving_sig.clone();
                            saving_v.set(true);
                            let client = api.with(|c| c.clone());
                            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                            let body = serde_json::json!({
                                "customer_id": cust_id,
                                "invoice_id": serde_json::Value::Null,
                                "payment_date": today,
                                "amount": total,
                                "payment_method": method_sig.read().clone(),
                                "reference": reference_sig.read().clone(),
                                "notes": notes_sig.read().clone(),
                                "allocations": allocations,
                            });
                            let mut toast3 = toast2.clone();
                            let mut modal2 = modal.clone();
                            let mut saving2 = saving_sig.clone();
                            let mut refresh2 = refresh.clone();
                            spawn(async move {
                                match client.create_payment(&body).await {
                                    Ok(_) => {
                                        toast3.success("Payment Recorded", "Payment recorded successfully.");
                                        modal2.set(false);
                                        saving2.set(false);
                                        refresh2.restart();
                                    }
                                    Err(e) => {
                                        toast3.error("Payment Failed", &e);
                                        saving2.set(false);
                                    }
                                }
                            });
                        }
                    };
                    let on_delete = { let mut m = show_delete_modal.clone(); move |_| m.set(true) };
                    let cancel_delete = { let mut m = show_delete_modal.clone(); move |_| m.set(false) };
                    let confirm_delete = {
                        let mut m = show_delete_modal.clone();
                        let mut t2 = t.clone();
                        let nav2 = nav.clone();
                        move |_| { m.set(false); t2.success("Customer Deleted", "Customer has been deleted."); nav2.push("/customers"); }
                    };
                    let customer = customer_opt.as_ref().unwrap();

                    rsx! {
                        // ── Header ──
                        div { class: "customer-detail-header",
                            div { class: "customer-detail-title-group",
                                Button { class: Some("customer-detail-back".to_string()), variant: ButtonVariant::Ghost, onclick: on_back, "← Back to Customers" }
                                div { class: "customer-detail-title-row",
                                    h1 { "{customer.customer_name}" }
                                    span { class: "customer-detail-code", "{customer.customer_code}" }
                                    span { class: "customer-status-badge {status_class(&customer.status)}",
                                        match customer.status.as_str() {
                                            "Active" => "✓ Active",
                                            "Inactive" => "— Inactive",
                                            "Over Limit" => "⚠ Over Limit",
                                            _ => "{customer.status}",
                                        }
                                    }
                                }
                            }
                        }

                        // ── KPI Stat Cards ──
                        div { class: "customer-detail-kpis",
                            StatCard {
                                title: "Current Balance".to_string(),
                                value: format!("PKR {:.0}", customer.current_balance),
                                variant: if customer.status == "Over Limit" { StatCardVariant::Danger }
                                         else if util > 80.0 { StatCardVariant::Warning }
                                         else { StatCardVariant::Primary },
                                icon: Some("💰".to_string()),
                                footer: Some(format!("Credit: PKR {:.0}", customer.credit_limit)),
                            }
                            StatCard {
                                title: "Available Credit".to_string(),
                                value: format!("PKR {:.0}", balance_remaining),
                                variant: if balance_remaining <= 0.0 { StatCardVariant::Danger }
                                         else if balance_remaining < customer.credit_limit * 0.2 { StatCardVariant::Warning }
                                         else { StatCardVariant::Success },
                                icon: Some("💳".to_string()),
                                footer: Some(format!("{:.0}% utilized", util)),
                            }
                            StatCard {
                                title: "Invoices".to_string(),
                                value: format!("{} open", unpaid_count),
                                variant: if overdue_count > 0 { StatCardVariant::Warning }
                                         else { StatCardVariant::Primary },
                                icon: Some("🧾".to_string()),
                                footer: Some(format!("{} overdue", overdue_count)),
                            }
                            StatCard {
                                title: "Total Invoiced".to_string(),
                                value: format!("PKR {:.0}", customer.total_invoiced),
                                variant: StatCardVariant::Primary,
                                icon: Some("📊".to_string()),
                                footer: Some(format!("Paid: PKR {:.0}", customer.total_paid)),
                            }
                        }

                        // ── Tabs ──
                        div { class: "customer-tabs",
                            {tabs.iter().enumerate().map(|(i, tab_name)| {
                                let is_active = *active_tab.read() == i;
                                let count = match i {
                                    1 => Some(invoices.len()),
                                    2 => Some(payments.len()),
                                    3 => Some(ledger.len()),
                                    _ => None,
                                };
                                let tab_class = if is_active { "customer-tab customer-tab-active" } else { "customer-tab" };
                                let mut tab_set = active_tab.clone();
                                rsx! {
                                    button {
                                        class: "{tab_class}",
                                        r#type: "button",
                                        onclick: move |_| { tab_set.set(i); },
                                        "{tab_name}"
                                        if let Some(cnt) = count {
                                            span { class: "customer-tab-count", "{cnt}" }
                                        }
                                    }
                                }
                            })}
                        }

                        // ════════ TAB: Overview ════════
                        if *active_tab.read() == 0 {
                            div { class: "customer-section",
                                div { class: "customer-section-header", h2 { "Customer Information" } }
                                div { class: "customer-info-grid",
                                    div { class: "customer-field", span { class: "customer-field-label", "Email" } span { class: "customer-field-value", "{customer.email}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Phone" } span { class: "customer-field-value", "{customer.phone}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "City" } span { class: "customer-field-value", "{customer.city}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Type" } span { class: "customer-field-value", "{customer.customer_type}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Terms" } span { class: "customer-field-value", "{customer.payment_terms}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Credit Limit" } span { class: "customer-field-value monospace", "PKR {customer.credit_limit:.0}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Opening Balance" } span { class: "customer-field-value monospace", "PKR {customer.opening_balance:.0}" } }
                                    div { class: "customer-field", span { class: "customer-field-label", "Last Invoice" } span { class: "customer-field-value", "{customer.last_invoice_date}" } }
                                }
                            }

                            if customer.notes.is_empty() {
                                div { class: "customer-section",
                                    div { class: "customer-section-header", h2 { "Notes" } }
                                    p { class: "customer-notes", style: "font-style: italic;", "No notes recorded for this customer." }
                                }
                            } else {
                                div { class: "customer-section",
                                    div { class: "customer-section-header", h2 { "Notes" } }
                                    p { class: "customer-notes", "{customer.notes}" }
                                }
                            }

                            div { class: "customer-section",
                                div { class: "customer-section-header", h2 { "Recent Activity" } }
                                div { class: "customer-activity",
                                    {activity_items.iter().map(|(text, icon_class, date)| {
                                        rsx! {
                                            div { class: "activity-item",
                                                div { class: "activity-icon {icon_class}", "📋" }
                                                div { class: "activity-content", span { dangerous_inner_html: "{text}" } }
                                                span { class: "activity-date", "{date}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }

                        // ════════ TAB: Invoices ════════
                        if *active_tab.read() == 1 {
                            div { class: "customer-section",
                                div { class: "customer-section-header",
                                    h2 { "Invoices" }
                                    Button { variant: ButtonVariant::Primary, onclick: on_new_invoice, "＋ New Invoice" }
                                }
                                if invoices.is_empty() {
                                    div { class: "customer-table-empty", "No invoices found for this customer." }
                                } else {
                                    table { class: "customer-table",
                                        thead { tr {
                                            th { "Invoice #" } th { "Date" } th { "Due Date" } th { "Status" }
                                            th { class: "text-right", "Total" } th { class: "text-right", "Paid" } th { class: "text-right", "Balance" }
                                        }}
                                        tbody {
                                            {invoices.iter().map(|inv| {
                                                let bdg = badge_class(&inv.status);
                                                let bc = if inv.balance > 0.0 { "text-danger" } else { "text-success" };
                                                rsx! {
                                                    tr {
                                                        td { style: "font-family: monospace;", "{inv.invoice_no}" }
                                                        td { "{inv.date}" } td { "{inv.due_date}" }
                                                        td { span { class: "customer-table-badge {bdg}", "{inv.status}" } }
                                                        td { class: "text-right", "PKR {inv.total:.0}" }
                                                        td { class: "text-right", "PKR {inv.paid:.0}" }
                                                        td { class: "text-right {bc}", "PKR {inv.balance:.0}" }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            }
                        }

                        // ════════ TAB: Payments ════════
                        if *active_tab.read() == 2 {
                            div { class: "customer-section",
                                div { class: "customer-section-header",
                                    h2 { "Payments" }
                                    Button { variant: ButtonVariant::Primary, onclick: on_record_payment, icon: Some("💰".to_string()), "Record Payment" }
                                }
                                if payments.is_empty() {
                                    div { class: "customer-table-empty", "No payments recorded." }
                                } else {
                                    table { class: "customer-table",
                                        thead { tr {
                                            th { "Payment #" } th { "Date" } th { "Method" } th { "Reference" }
                                            th { class: "text-right", "Amount" }
                                        }}
                                        tbody {
                                            {payments.iter().map(|pmt| {
                                                rsx! {
                                                    tr {
                                                        td { style: "font-family: monospace;", "{pmt.payment_no}" }
                                                        td { "{pmt.date}" } td { "{pmt.method}" }
                                                        td { style: "font-family: monospace;", "{pmt.reference}" }
                                                        td { class: "text-right text-success", "PKR {pmt.amount:.0}" }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            }
                        }

                        // ════════ TAB: Ledger ════════
                        if *active_tab.read() == 3 {
                            div { class: "customer-section",
                                div { class: "customer-section-header", h2 { "Ledger" } }
                                if ledger.is_empty() {
                                    div { class: "customer-table-empty", "No ledger entries found." }
                                } else {
                                    table { class: "customer-table",
                                        thead { tr {
                                            th { "Date" } th { "Type" } th { "Reference" }
                                            th { class: "text-right", "Debit" } th { class: "text-right", "Credit" }
                                            th { class: "text-right", "Balance" } th { "Notes" }
                                        }}
                                        tbody {
                                            {ledger.iter().map(|entry| {
                                                let type_icon = match entry.entry_type.as_str() {
                                                    "INVOICE" => "📄", "PAYMENT" => "💰", "OPENING" => "📋", _ => "•",
                                                };
                                                let dc = if entry.debit > 0.0 { "text-danger" } else { "" };
                                                let cc = if entry.credit > 0.0 { "text-success" } else { "" };
                                                let bc2 = if entry.balance > 0.0 { "text-danger" } else { "text-success" };
                                                rsx! {
                                                    tr {
                                                        td { "{entry.date}" }
                                                        td { "{type_icon} {entry.entry_type}" }
                                                        td { style: "font-family: monospace;", "{entry.reference}" }
                                                        td { class: "text-right {dc}", if entry.debit > 0.0 { "PKR {entry.debit:.0}" } else { "—" } }
                                                        td { class: "text-right {cc}", if entry.credit > 0.0 { "PKR {entry.credit:.0}" } else { "—" } }
                                                        td { class: "text-right {bc2}", "PKR {entry.balance:.0}" }
                                                        td { "{entry.notes}" }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            }
                        }

                        // ── Action Bar ──
                        div { class: "customer-actions",
                            div { class: "customer-actions-left",
                                Button { variant: ButtonVariant::Primary, onclick: on_edit, icon: Some("✏️".to_string()), "Edit Customer" }
                                Button { variant: ButtonVariant::Secondary, onclick: on_new_invoice, icon: Some("🧾".to_string()), "New Invoice" }
                            }
                            div { class: "customer-actions-right",
                                Button { variant: ButtonVariant::Ghost, onclick: on_delete, icon: Some("🗑️".to_string()), "Delete" }
                            }
                        }

                        // ── Delete Confirmation Modal ──
                        Modal {
                            is_open: show_delete_modal,
                            title: Some("Delete Customer".to_string()),
                            size: ModalSize::Sm,
                            close_on_backdrop: true,
                            close_on_escape: true,
                            footer: rsx! {
                                Button { variant: ButtonVariant::Secondary, onclick: cancel_delete, "Cancel" }
                                Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete Customer" }
                            },
                            div {
                                p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;", "Delete {customer.customer_name}?" }
                                p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                                    "This action cannot be undone. Deleting \"{customer.customer_code}\" will permanently remove the customer record."
                                }
                            }
                        }

                        // ── Record Payment Modal ──
                        Modal {
                            is_open: show_payment_modal,
                            title: Some(format!("Record Payment — {}", customer.customer_name)),
                            size: ModalSize::Md,
                            close_on_backdrop: true,
                            close_on_escape: true,
                            footer: rsx! {
                                Button { variant: ButtonVariant::Secondary, onclick: cancel_payment, "Cancel" }
                                Button { variant: ButtonVariant::Primary, loading: *is_saving.read(), onclick: submit_payment, "Record Payment" }
                            },
                            if unpaid_invoices.is_empty() {
                                div { class: "customer-table-empty", "No unpaid invoices to settle." }
                            } else {
                                {{
                                    // Total being allocated (reactive on pay_alloc)
                                    let alloc_snapshot = pay_alloc.read();
                                    let entered_total: f64 = unpaid_invoices.iter()
                                        .map(|inv| alloc_snapshot.get(&inv.id).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0))
                                        .sum();
                                    rsx! {
                                        div { style: "display:flex; gap:10px; margin-bottom:14px; flex-wrap:wrap;",
                                            div { class: "payment-field", style: "flex:1; min-width:140px; display:flex; flex-direction:column; gap:4px;",
                                                label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "Method" }
                                                select {
                                                    style: "padding:8px 10px; border:1px solid var(--border-color); border-radius:6px; font-size:13px;",
                                                    value: "{pay_method.read()}",
                                                    onchange: { let mut m = pay_method.clone(); move |e: Event<FormData>| m.set(e.value()) },
                                                    option { value: "Cash", "Cash" }
                                                    option { value: "Bank Transfer", "Bank Transfer" }
                                                    option { value: "Cheque", "Cheque" }
                                                    option { value: "Credit Card", "Credit Card" }
                                                    option { value: "Online", "Online Payment" }
                                                }
                                            }
                                            div { class: "payment-field", style: "flex:1; min-width:140px; display:flex; flex-direction:column; gap:4px;",
                                                label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "Reference" }
                                                input {
                                                    style: "padding:8px 10px; border:1px solid var(--border-color); border-radius:6px; font-size:13px;",
                                                    r#type: "text", placeholder: "Cheque #, txn ID…",
                                                    value: "{pay_reference.read()}",
                                                    oninput: { let mut r = pay_reference.clone(); move |e: Event<FormData>| r.set(e.value()) },
                                                }
                                            }
                                        }
                                        table { class: "customer-table",
                                            thead { tr {
                                                th { "Invoice #" } th { "Date" }
                                                th { class: "text-right", "Balance" }
                                                th { class: "text-right", "Pay Now" }
                                            }}
                                            tbody {
                                                {unpaid_invoices.iter().map(|inv| {
                                                    let inv_id = inv.id;
                                                    let bal = inv.balance;
                                                    let current = alloc_snapshot.get(&inv_id).cloned().unwrap_or_default();
                                                    let mut alloc_set = pay_alloc.clone();
                                                    let mut alloc_fill = pay_alloc.clone();
                                                    rsx! {
                                                        tr {
                                                            td { style: "font-family: monospace;", "{inv.invoice_no}" }
                                                            td { "{inv.date}" }
                                                            td { class: "text-right",
                                                                span {
                                                                    style: "cursor:pointer; text-decoration:underline dotted;",
                                                                    title: "Click to pay full balance",
                                                                    onclick: move |_| {
                                                                        let mut m = alloc_fill.read().clone();
                                                                        m.insert(inv_id, format!("{:.2}", bal));
                                                                        alloc_fill.set(m);
                                                                    },
                                                                    "PKR {bal:.0}"
                                                                }
                                                            }
                                                            td { class: "text-right",
                                                                input {
                                                                    style: "width:110px; padding:6px 8px; border:1px solid var(--border-color); border-radius:6px; font-size:13px; text-align:right;",
                                                                    r#type: "number", min: "0", max: "{bal}", step: "0.01",
                                                                    value: "{current}",
                                                                    oninput: move |e: Event<FormData>| {
                                                                        let mut m = alloc_set.read().clone();
                                                                        m.insert(inv_id, e.value());
                                                                        alloc_set.set(m);
                                                                    },
                                                                }
                                                            }
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                        div { class: "payment-summary", style: "display:flex; justify-content:space-between; padding:12px; background:var(--bg-muted,#f5f5f5); border-radius:6px; margin-top:12px; font-size:14px; font-weight:600;",
                                            span { "Total Payment" }
                                            span { class: "text-success", "PKR {entered_total:.2}" }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                }}
            }
        }
    }
}
