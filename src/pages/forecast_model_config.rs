//! Forecast Model Config Page — Configure ARIMA, ETS, Prophet, and Neural models.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast,
};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.mc-page { max-width: 800px; margin: 0 auto; }
.mc-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.mc-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.mc-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.mc-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }

.mc-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.mc-form-row > * { flex: 1; min-width: 180px; }
@media (max-width: 768px) { .mc-form-row { flex-direction: column; } .mc-form-row > * { min-width: 100%; } }

.mc-params-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 12px; }
.mc-param-group { padding: 12px; background: #f8f9fa; border-radius: 6px; border: 1px solid var(--border-color, #e0e0e0); }
.mc-param-group label { display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.3px; }
.mc-param-group input { width: 100%; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; box-sizing: border-box; }

.mc-toggle { display: flex; align-items: center; gap: 10px; padding: 10px 14px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; cursor: pointer; font-size: 13px; color: var(--text-primary); background: #fff; user-select: none; transition: all 0.15s ease; }
.mc-toggle:hover { border-color: var(--accent, #4a90d9); }
.mc-toggle-active { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.06); font-weight: 600; }

.mc-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }

.mc-status { display: flex; align-items: center; gap: 8px; padding: 10px 16px; border-radius: 6px; font-size: 13px; margin-top: 12px; }
.mc-status-ready { background: rgba(40, 167, 69, 0.08); color: #28a745; }
.mc-status-training { background: rgba(74, 144, 217, 0.08); color: #4a90d9; }
"##;

// ============================================================================
// Types
// ============================================================================

fn algorithm_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "arima".to_string(), label: "ARIMA".to_string() },
        SelectOption { value: "ets".to_string(), label: "ETS".to_string() },
        SelectOption { value: "prophet".to_string(), label: "Prophet".to_string() },
        SelectOption { value: "neural".to_string(), label: "Neural (LSTM)".to_string() },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ForecastModelConfigPage() -> Element {
    let mut toast = use_toast();
    let mut model_name = use_signal(|| "Demand Forecast ARIMA".to_string());
    let mut algorithm = use_signal(|| "arima".to_string());
    let mut training_start = use_signal(|| "2024-01-01".to_string());
    let mut training_end = use_signal(|| "2026-06-27".to_string());
    let mut auto_tune = use_signal(|| true);
    let mut seasonality = use_signal(|| true);
    let is_saving = use_signal(|| false);
    let is_testing = use_signal(|| false);
    let status = use_signal(|| "ready".to_string()); // ready, training, saved

    let mut arima_p = use_signal(|| "2".to_string());
    let mut arima_d = use_signal(|| "1".to_string());
    let mut arima_q = use_signal(|| "2".to_string());

    let mut ets_error = use_signal(|| "A".to_string());
    let mut ets_trend = use_signal(|| "A".to_string());
    let mut ets_seasonal = use_signal(|| "A".to_string());

    let mut prophet_growth = use_signal(|| "linear".to_string());
    let mut prophet_seasonality = use_signal(|| "weekly".to_string());

    let mut neural_layers = use_signal(|| "2".to_string());
    let mut neural_units = use_signal(|| "64".to_string());
    let mut neural_epochs = use_signal(|| "100".to_string());

    let t_save = toast.clone();
    let on_save = {
        let mut saving = is_saving.clone();
        let mut s = status.clone();
        let name = model_name.clone();
        move |_| {
            let mut t = t_save.clone();
            saving.set(true);
            s.set("training".to_string());
            let n = name.read().clone();
            let mut t2 = t.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(1500)).await;
                saving.set(false);
                s.set("saved".to_string());
                t2.success("Model Saved", &format!("\"{}\" configuration saved and model trained.", n));
            });
        }
    };

    let on_test = {
        let mut testing = is_testing.clone();
        let mut s = status.clone();
        let name = model_name.clone();
        move |_| {
            let mut t = toast.clone();
            testing.set(true);
            s.set("training".to_string());
            let n = name.read().clone();
            let mut t2 = t.clone();
            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(2000)).await;
                testing.set(false);
                s.set("ready".to_string());
                t2.success("Test Run Complete", &format!("\"{}\" test run completed. MAPE: 11.2%", n));
            });
        }
    };

    let status_cls = match status.read().as_str() {
        "saved" => "mc-status mc-status-ready",
        "training" => "mc-status mc-status-training",
        _ => "mc-status mc-status-ready",
    };

    let status_text = match status.read().as_str() {
        "saved" => "✅ Model trained and saved — last trained Jun 27, 2026",
        "training" => "⏳ Training in progress…",
        _ => "🟢 Model ready for configuration",
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page mc-page",

            div { class: "mc-header",
                h1 { "Forecast Model Configuration" }
            }

            // Basic info
            div { class: "mc-section",
                h2 { "Model Information" }
                div { class: "mc-form-row",
                    FormInput {
                        label: Some("Model Name".to_string()),
                        value: model_name.read().clone(),
                        oninput: move |v: String| { model_name.set(v); },
                        r#type: InputType::Text,
                        placeholder: Some("Enter model name".to_string()),
                        required: true,
                    }
                    div {
                        SearchableSelect {
                            options: algorithm_options(),
                            selected_value: Some(algorithm.read().clone()),
                            on_select: move |v: String| { algorithm.set(v); },
                            placeholder: "Select algorithm…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            // Training period
            div { class: "mc-section",
                h2 { "Training Period" }
                div { class: "mc-form-row",
                    FormInput {
                        label: Some("Start Date".to_string()),
                        value: training_start.read().clone(),
                        oninput: move |v: String| { training_start.set(v); },
                        r#type: InputType::Date,
                    }
                    FormInput {
                        label: Some("End Date".to_string()),
                        value: training_end.read().clone(),
                        oninput: move |v: String| { training_end.set(v); },
                        r#type: InputType::Date,
                    }
                }
            }

            // Parameters section
            div { class: "mc-section",
                h2 { "Algorithm Parameters" }

                if *algorithm.read() == "arima" {
                    div { class: "mc-params-grid",
                        div { class: "mc-param-group",
                            label { "p (AR Order)" }
                            input { r#type: "number", value: "{arima_p}",
                                oninput: move |e| { arima_p.set(e.value()); }
                            }
                        }
                        div { class: "mc-param-group",
                            label { "d (Differences)" }
                            input { r#type: "number", value: "{arima_d}",
                                oninput: move |e| { arima_d.set(e.value()); }
                            }
                        }
                        div { class: "mc-param-group",
                            label { "q (MA Order)" }
                            input { r#type: "number", value: "{arima_q}",
                                oninput: move |e| { arima_q.set(e.value()); }
                            }
                        }
                    }
                }

                if *algorithm.read() == "ets" {
                    div { class: "mc-params-grid",
                        div { class: "mc-param-group",
                            label { "Error Type" }
                            input { value: "{ets_error}",
                                oninput: move |e| { ets_error.set(e.value()); }
                            }
                            span { style: "font-size: 11px; color: var(--text-secondary);", "A, M, or N" }
                        }
                        div { class: "mc-param-group",
                            label { "Trend Type" }
                            input { value: "{ets_trend}",
                                oninput: move |e| { ets_trend.set(e.value()); }
                            }
                            span { style: "font-size: 11px; color: var(--text-secondary);", "A, M, or N" }
                        }
                        div { class: "mc-param-group",
                            label { "Seasonal Type" }
                            input { value: "{ets_seasonal}",
                                oninput: move |e| { ets_seasonal.set(e.value()); }
                            }
                            span { style: "font-size: 11px; color: var(--text-secondary);", "A, M, or N" }
                        }
                    }
                }

                if *algorithm.read() == "prophet" {
                    div { class: "mc-params-grid",
                        div { class: "mc-param-group",
                            label { "Growth" }
                            input { value: "{prophet_growth}",
                                oninput: move |e| { prophet_growth.set(e.value()); }
                            }
                            span { style: "font-size: 11px; color: var(--text-secondary);", "linear or logistic" }
                        }
                        div { class: "mc-param-group",
                            label { "Seasonality Mode" }
                            input { value: "{prophet_seasonality}",
                                oninput: move |e| { prophet_seasonality.set(e.value()); }
                            }
                            span { style: "font-size: 11px; color: var(--text-secondary);", "weekly, monthly, yearly" }
                        }
                    }
                }

                if *algorithm.read() == "neural" {
                    div { class: "mc-params-grid",
                        div { class: "mc-param-group",
                            label { "Hidden Layers" }
                            input { r#type: "number", value: "{neural_layers}",
                                oninput: move |e| { neural_layers.set(e.value()); }
                            }
                        }
                        div { class: "mc-param-group",
                            label { "Units per Layer" }
                            input { r#type: "number", value: "{neural_units}",
                                oninput: move |e| { neural_units.set(e.value()); }
                            }
                        }
                        div { class: "mc-param-group",
                            label { "Epochs" }
                            input { r#type: "number", value: "{neural_epochs}",
                                oninput: move |e| { neural_epochs.set(e.value()); }
                            }
                        }
                    }
                }

                // Toggles
                div { style: "display: flex; gap: 12px; margin-top: 16px; flex-wrap: wrap;",
                    div {
                        class: if *auto_tune.read() { "mc-toggle mc-toggle-active" } else { "mc-toggle" },
                        onclick: move |_| { let v = !*auto_tune.read(); auto_tune.set(v); },
                        "🎯 Auto-Tune"
                    }
                    div {
                        class: if *seasonality.read() { "mc-toggle mc-toggle-active" } else { "mc-toggle" },
                        onclick: move |_| { let v = !*seasonality.read(); seasonality.set(v); },
                        "🔄 Seasonality"
                    }
                }
            }

            // Status
            div { class: "{status_cls}",
                "{status_text}"
            }

            // Action bar
            div { class: "mc-action-bar",
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: on_test,
                    loading: *is_testing.read(),
                    icon: Some("▶".to_string()),
                    "Test Run"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_save,
                    loading: *is_saving.read(),
                    icon: Some("💾".to_string()),
                    "Save Model"
                }
            }
        }
    }
}
