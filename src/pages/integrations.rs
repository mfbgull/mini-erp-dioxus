//! Integrations Page — Dashboard of integration cards showing connection
//! status for third-party services.

use crate::components::common::{Button, ButtonVariant, use_toast};
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

fn sample_integrations() -> Vec<Integration> {
    vec![
        Integration {
            id: 1,
            title: "Email (SMTP)".to_string(),
            description: "Send transactional emails, invoices, and notifications via SMTP server.".to_string(),
            icon: "📧",
            icon_class: "integration-icon-blue",
            connected: true,
            last_sync: "Connected — tested successfully".to_string(),
        },
        Integration {
            id: 2,
            title: "SMS Gateway".to_string(),
            description: "Send SMS alerts for low stock, payment reminders, and system notifications.".to_string(),
            icon: "📱",
            icon_class: "integration-icon-green",
            connected: false,
            last_sync: "Not configured".to_string(),
        },
        Integration {
            id: 3,
            title: "Payment Gateway".to_string(),
            description: "Process customer payments via credit card, bank transfer, and digital wallets.".to_string(),
            icon: "💳",
            icon_class: "integration-icon-purple",
            connected: true,
            last_sync: "Connected — API v3.2".to_string(),
        },
        Integration {
            id: 4,
            title: "Accounting Software".to_string(),
            description: "Sync invoices, expenses, and journal entries with external accounting platforms.".to_string(),
            icon: "📊",
            icon_class: "integration-icon-orange",
            connected: false,
            last_sync: "Not configured".to_string(),
        },
        Integration {
            id: 5,
            title: "E-Commerce".to_string(),
            description: "Sync product catalog, inventory levels, and orders with online storefronts.".to_string(),
            icon: "🛒",
            icon_class: "integration-icon-teal",
            connected: false,
            last_sync: "Not configured".to_string(),
        },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn IntegrationsPage() -> Element {
    let toast = use_toast();

    // ── State ──
    let integrations = use_signal(sample_integrations);
    let connecting_id = use_signal(|| 0i64);

    // ── Handlers ──
    let mut on_toggle = {
        let mut ints = integrations.clone();
        let mut conn = connecting_id.clone();
        let mut toast = toast.clone();
        move |id: i64| {
            conn.set(id);
            let mut t = toast.clone();
            let mut int_clone = ints.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(800)).await;
                let mut items = int_clone.write();
                if let Some(idx) = items.iter().position(|i| i.id == id) {
                    let was_connected = items[idx].connected;
                    items[idx].connected = !was_connected;
                    items[idx].last_sync = if items[idx].connected {
                        "Connected successfully".to_string()
                    } else {
                        "Disconnected".to_string()
                    };
                    let name = items[idx].title.clone();
                    if items[idx].connected {
                        t.success("Integration Connected", &format!("{} is now connected.", name));
                    } else {
                        t.info("Integration Disconnected", &format!("{} has been disconnected.", name));
                    }
                }
                conn.set(0);
            });
        }
    };

    let is_connecting = |id: i64| *connecting_id.read() == id;

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
                {integrations.read().iter().map(|int| {
                    let mut toggle = on_toggle.clone();
                    let status_class = if int.connected { "integration-status-connected" } else { "integration-status-disconnected" };
                    let status_text = if int.connected { "● Connected" } else { "○ Disconnected" };
                    let btn_variant = if int.connected { ButtonVariant::Secondary } else { ButtonVariant::Primary };
                    let btn_text = if int.connected { "Disconnect" } else { "Connect" };
                    let loading = is_connecting(int.id);
                    let id = int.id;

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
                                Button {
                                    variant: btn_variant,
                                    size: crate::components::common::ButtonSize::Sm,
                                    onclick: move |_| toggle(id),
                                    loading: loading,
                                    "{btn_text}"
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
