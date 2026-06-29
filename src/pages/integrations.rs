//! Integrations Page — Dashboard of integration cards showing connection
//! status for third-party services.

use crate::auth::use_auth;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.integrations-page { max-width: 960px; margin: 0 auto; }
.integrations-header { margin-bottom: 20px; }
.integrations-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.integrations-header p { font-size: 13px; color: var(--text-secondary); margin: 4px 0 0 0; }
.integrations-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px; }
.integration-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; display: flex; flex-direction: column; gap: 12px; transition: box-shadow 0.15s; }
.integration-card:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
.integration-card-header { display: flex; align-items: flex-start; gap: 12px; }
.integration-icon { width: 44px; height: 44px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 22px; flex-shrink: 0; }
.integration-icon-blue { background: rgba(74,144,217,0.1); }
.integration-icon-green { background: rgba(40,167,69,0.1); }
.integration-icon-purple { background: rgba(111,66,193,0.1); }
.integration-icon-orange { background: rgba(253,126,20,0.1); }
.integration-icon-teal { background: rgba(32,201,151,0.1); }
.integration-info { flex: 1; min-width: 0; }
.integration-info h3 { font-size: 15px; font-weight: 600; margin: 0; color: var(--text-primary); }
.integration-info p { font-size: 12px; color: var(--text-secondary); margin: 4px 0 0 0; line-height: 1.4; }
.integration-status { display: inline-flex; align-items: center; gap: 5px; padding: 3px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; margin-top: 6px; }
.integration-status-connected { background: rgba(40,167,69,0.1); color: #28a745; }
.integration-status-disconnected { background: rgba(108,117,125,0.1); color: #6c757d; }
.integration-meta { display: flex; align-items: center; justify-content: space-between; padding-top: 12px; border-top: 1px solid var(--border-color, #e0e0e0); }
.integration-meta span { font-size: 11px; color: var(--text-secondary); }
@media (max-width: 768px) { .integrations-grid { grid-template-columns: 1fr; } }
"##;

// ============================================================================
// Data Model
// ============================================================================

#[derive(Clone, Debug)]
struct Integration {
    id: i64,
    title: String,
    description: String,
    icon: &'static str,
    icon_class: &'static str,
    connected: bool,
    last_sync: String,
}

fn service_title(service: &str) -> String {
    match service {
        "email" => "Email (SMTP)".to_string(),
        "sms" => "SMS Gateway".to_string(),
        "payment" => "Payment Gateway".to_string(),
        "accounting" => "Accounting Software".to_string(),
        "ecommerce" => "E-Commerce".to_string(),
        _ => service.to_string(),
    }
}

fn service_icon(service: &str) -> &'static str {
    match service {
        "email" => "📧",
        "sms" => "📱",
        "payment" => "💳",
        "accounting" => "📊",
        "ecommerce" => "🛒",
        _ => "🔌",
    }
}

fn service_icon_class(service: &str) -> &'static str {
    match service {
        "email" => "integration-icon-blue",
        "sms" => "integration-icon-green",
        "payment" => "integration-icon-purple",
        "accounting" => "integration-icon-orange",
        "ecommerce" => "integration-icon-teal",
        _ => "integration-icon-blue",
    }
}

fn service_description(service: &str) -> &'static str {
    match service {
        "email" => "Send transactional emails, invoices, and notifications via SMTP server.",
        "sms" => "Send SMS alerts for low stock, payment reminders, and system notifications.",
        "payment" => "Process customer payments via credit card, bank transfer, and digital wallets.",
        "accounting" => "Sync invoices, expenses, and journal entries with external accounting platforms.",
        "ecommerce" => "Sync product catalog, inventory levels, and orders with online storefronts.",
        _ => "",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn IntegrationsPage() -> Element {
    // ── State ──
    let api = use_auth().api;
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let server_ints = client.list_integrations().await.unwrap_or_default();
            server_ints.into_iter().enumerate().map(|(i, si)| Integration {
                id: i as i64 + 1,
                title: service_title(&si.service),
                description: service_description(&si.service).to_string(),
                icon: service_icon(&si.service),
                icon_class: service_icon_class(&si.service),
                connected: si.is_configured,
                last_sync: String::new(), // ponytail: not in API
            }).collect::<Vec<_>>()
        }
    });
    let integrations = resource.read().as_ref().cloned().unwrap_or_default();

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page integrations-page",

            // ── Header ──
            div { class: "integrations-header",
                h1 { "Integrations" }
                p { "Connect MiniERP with third-party services and platforms." }
            }

            // ── Cards Grid ──
            div { class: "integrations-grid",
                {integrations.iter().map(|int| {
                    let status_class = if int.connected { "integration-status-connected" } else { "integration-status-disconnected" };
                    let status_text = if int.connected { "● Connected" } else { "○ Disconnected" };

                    rsx! {
                        div { class: "integration-card", key: "{int.id}",
                            div { class: "integration-card-header",
                                div { class: "integration-icon {int.icon_class}", "{int.icon}" }
                                div { class: "integration-info",
                                    h3 { "{int.title}" }
                                    p { "{int.description}" }
                                    span { class: "integration-status {status_class}", "{status_text}" }
                                }
                            }
                            div { class: "integration-meta",
                                span { "{int.last_sync}" }
                            }
                        }
                    }
                })}
            }
        }
    }
}
