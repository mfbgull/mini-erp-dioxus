//! Settings Page — Tabbed settings form for company configuration,
//! user preferences, and notification preferences.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, use_toast,
};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.settings-page { max-width: 800px; margin: 0 auto; }
.settings-header { margin-bottom: 20px; }
.settings-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.settings-header p { font-size: 13px; color: var(--text-secondary); margin: 4px 0 0 0; }
.settings-tabs { display: flex; gap: 0; margin-bottom: 16px; border-bottom: 2px solid var(--border-color, #e0e0e0); }
.settings-tab { padding: 10px 20px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border: none; background: none; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.15s ease; }
.settings-tab:hover { color: var(--text-primary); background: rgba(74, 144, 217, 0.04); }
.settings-tab-active { color: var(--accent, #4a90d9); border-bottom-color: var(--accent, #4a90d9); font-weight: 600; }
.settings-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.settings-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.settings-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.settings-form-row > * { flex: 1; min-width: 180px; }
.settings-toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 10px 0; }
.settings-toggle-row + .settings-toggle-row { border-top: 1px solid var(--border-color, #e0e0e0); }
.settings-toggle-label { display: flex; flex-direction: column; gap: 2px; }
.settings-toggle-label span:first-child { font-size: 14px; font-weight: 500; color: var(--text-primary); }
.settings-toggle-label span:last-child { font-size: 12px; color: var(--text-secondary); }
.settings-toggle { position: relative; width: 44px; height: 24px; flex-shrink: 0; }
.settings-toggle input { opacity: 0; width: 0; height: 0; }
.settings-toggle-slider { position: absolute; cursor: pointer; inset: 0; background: #ccc; border-radius: 24px; transition: 0.2s; }
.settings-toggle-slider::before { content: ""; position: absolute; height: 18px; width: 18px; left: 3px; bottom: 3px; background: white; border-radius: 50%; transition: 0.2s; }
.settings-toggle input:checked + .settings-toggle-slider { background: var(--accent, #4a90d9); }
.settings-toggle input:checked + .settings-toggle-slider::before { transform: translateX(20px); }
.settings-action-bar { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
@media (max-width: 768px) { .settings-form-row { flex-direction: column; } .settings-form-row > * { min-width: 100%; } }
"##;

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SettingsPage() -> Element {
    let toast = use_toast();

    // ── Tab State ──
    let active_tab = use_signal(|| 0u32);

    // ── General Tab State ──
    let mut company_name = use_signal(|| "MiniERP Solutions (Pvt) Ltd".to_string());
    let mut tax_id = use_signal(|| "NTN-1234567-8".to_string());
    let mut address = use_signal(|| "123 Business Avenue, Block 6, Gulshan-e-Iqbal, Karachi".to_string());
    let mut email = use_signal(|| "info@minierp.pk".to_string());
    let mut phone = use_signal(|| "+92 21 111 222 333".to_string());
    let mut currency = use_signal(|| "PKR".to_string());
    let mut timezone = use_signal(|| "Asia/Karachi".to_string());

    // ── Preferences Tab State ──
    let mut items_per_page = use_signal(|| "25".to_string());
    let mut default_tax_rate = use_signal(|| "16.0".to_string());
    let mut low_stock_threshold = use_signal(|| "10".to_string());
    let mut date_format = use_signal(|| "dd-MMM-yyyy".to_string());

    // ── Notifications Tab State ──
    let mut email_alerts = use_signal(|| true);
    let mut sms_alerts = use_signal(|| false);

    // ── Saving State ──
    let is_saving = use_signal(|| false);

    // ── Tab Switch Handler ──
    let mut switch_tab = {
        let mut tab = active_tab.clone();
        move |t: u32| { tab.set(t); }
    };

    // ── Save Handler ──
    let on_save = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        move |_| {
            saving.set(true);
            let mut t = toast.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(600)).await;
                saving.set(false);
                t.success("Settings Saved", "Company settings have been updated successfully.");
            });
        }
    };

    // ── Computed tab classes ──
    let tab_classes = |tab: u32| {
        if *active_tab.read() == tab {
            "settings-tab settings-tab-active"
        } else {
            "settings-tab"
        }
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page settings-page",

            // ── Header ──
            div { class: "settings-header",
                h1 { "Settings" }
                p { "Configure company information, user preferences, and notification preferences." }
            }

            // ── Tabs ──
            div { class: "settings-tabs",
                button { class: "{tab_classes(0)}", r#type: "button", onclick: move |_| switch_tab(0), "General" }
                button { class: "{tab_classes(1)}", r#type: "button", onclick: move |_| switch_tab(1), "Preferences" }
                button { class: "{tab_classes(2)}", r#type: "button", onclick: move |_| switch_tab(2), "Notifications" }
            }

            // ════════════════════════════
            // TAB 0: General
            // ════════════════════════════
            if *active_tab.read() == 0 {
                div { class: "settings-section",
                    h2 { "Company Information" }
                    div { class: "settings-form-row",
                        FormInput {
                            label: Some("Company Name".to_string()),
                            value: company_name.read().clone(),
                            oninput: move |v| company_name.set(v),
                            r#type: InputType::Text,
                            placeholder: Some("Enter company name".to_string()),
                        }
                        FormInput {
                            label: Some("Tax ID / NTN".to_string()),
                            value: tax_id.read().clone(),
                            oninput: move |v| tax_id.set(v),
                            r#type: InputType::Text,
                            placeholder: Some("NTN-XXXXXXX-X".to_string()),
                        }
                    }
                    div { class: "settings-form-row", style: "margin-top: 12px;",
                        FormInput {
                            label: Some("Address".to_string()),
                            value: address.read().clone(),
                            oninput: move |v| address.set(v),
                            r#type: InputType::TextArea,
                            placeholder: Some("Enter company address".to_string()),
                        }
                    }
                    div { class: "settings-form-row", style: "margin-top: 12px;",
                        FormInput {
                            label: Some("Email".to_string()),
                            value: email.read().clone(),
                            oninput: move |v| email.set(v),
                            r#type: InputType::Email,
                            placeholder: Some("info@company.com".to_string()),
                        }
                        FormInput {
                            label: Some("Phone".to_string()),
                            value: phone.read().clone(),
                            oninput: move |v| phone.set(v),
                            r#type: InputType::Tel,
                            placeholder: Some("+92 21 111 222 333".to_string()),
                        }
                    }
                    div { class: "settings-form-row", style: "margin-top: 12px;",
                        FormInput {
                            label: Some("Currency".to_string()),
                            value: currency.read().clone(),
                            oninput: move |v| currency.set(v),
                            r#type: InputType::Text,
                            placeholder: Some("PKR".to_string()),
                        }
                        FormInput {
                            label: Some("Timezone".to_string()),
                            value: timezone.read().clone(),
                            oninput: move |v| timezone.set(v),
                            r#type: InputType::Text,
                            placeholder: Some("Asia/Karachi".to_string()),
                        }
                    }
                }
            }

            // ════════════════════════════
            // TAB 1: Preferences
            // ════════════════════════════
            if *active_tab.read() == 1 {
                div { class: "settings-section",
                    h2 { "Preferences" }
                    div { class: "settings-form-row",
                        FormInput {
                            label: Some("Items Per Page".to_string()),
                            value: items_per_page.read().clone(),
                            oninput: move |v| items_per_page.set(v),
                            r#type: InputType::Number,
                            placeholder: Some("25".to_string()),
                            min: Some(5.0),
                            step: Some(5.0),
                            hint: Some("Default rows per list page".to_string()),
                        }
                        FormInput {
                            label: Some("Default Tax Rate (%)".to_string()),
                            value: default_tax_rate.read().clone(),
                            oninput: move |v| default_tax_rate.set(v),
                            r#type: InputType::Number,
                            placeholder: Some("16.0".to_string()),
                            min: Some(0.0),
                            step: Some(0.1),
                            hint: Some("Default sales tax percentage".to_string()),
                        }
                    }
                    div { class: "settings-form-row", style: "margin-top: 12px;",
                        FormInput {
                            label: Some("Low Stock Threshold".to_string()),
                            value: low_stock_threshold.read().clone(),
                            oninput: move |v| low_stock_threshold.set(v),
                            r#type: InputType::Number,
                            placeholder: Some("10".to_string()),
                            min: Some(0.0),
                            step: Some(1.0),
                            hint: Some("Alert when stock drops below this".to_string()),
                        }
                        FormInput {
                            label: Some("Date Format".to_string()),
                            value: date_format.read().clone(),
                            oninput: move |v| date_format.set(v),
                            r#type: InputType::Text,
                            placeholder: Some("dd-MMM-yyyy".to_string()),
                            hint: Some("Format for all date displays".to_string()),
                        }
                    }
                }
            }

            // ════════════════════════════
            // TAB 2: Notifications
            // ════════════════════════════
            if *active_tab.read() == 2 {
                div { class: "settings-section",
                    h2 { "Notification Preferences" }
                    div { class: "settings-toggle-row",
                        div { class: "settings-toggle-label",
                            span { "Email Alerts" }
                            span { "Receive system notifications via email" }
                        }
                        label { class: "settings-toggle",
                            input {
                                r#type: "checkbox",
                                checked: *email_alerts.read(),
                                oninput: move |_| {
                                    let v = !*email_alerts.read();
                                    email_alerts.set(v);
                                },
                            }
                            span { class: "settings-toggle-slider" }
                        }
                    }
                    div { class: "settings-toggle-row",
                        div { class: "settings-toggle-label",
                            span { "SMS Alerts" }
                            span { "Receive critical alerts via SMS" }
                        }
                        label { class: "settings-toggle",
                            input {
                                r#type: "checkbox",
                                checked: *sms_alerts.read(),
                                oninput: move |_| {
                                    let v = !*sms_alerts.read();
                                    sms_alerts.set(v);
                                },
                            }
                            span { class: "settings-toggle-slider" }
                        }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "settings-action-bar",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_save,
                    loading: *is_saving.read(),
                    icon: Some("💾".to_string()),
                    "Save Settings"
                }
            }
        }
    }
}
