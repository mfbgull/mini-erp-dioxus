//! Forecast Model Config Page — Configure ARIMA, ETS, Prophet, and Neural models.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast,
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

.mc-selector { margin-bottom: 20px; }
.mc-selector-row { display: flex; gap: 10px; align-items: flex-end; }
.mc-selector-row > .mc-selector-dropdown { flex: 1; }
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

fn reset_form(
    model_name: &mut Signal<String>,
    algorithm: &mut Signal<String>,
    training_start: &mut Signal<String>,
    training_end: &mut Signal<String>,
    auto_tune: &mut Signal<bool>,
    seasonality: &mut Signal<bool>,
    arima_p: &mut Signal<String>,
    arima_d: &mut Signal<String>,
    arima_q: &mut Signal<String>,
    ets_error: &mut Signal<String>,
    ets_trend: &mut Signal<String>,
    ets_seasonal: &mut Signal<String>,
    prophet_growth: &mut Signal<String>,
    prophet_seasonality: &mut Signal<String>,
    neural_layers: &mut Signal<String>,
    neural_units: &mut Signal<String>,
    neural_epochs: &mut Signal<String>,
) {
    model_name.set("Demand Forecast ARIMA".to_string());
    algorithm.set("arima".to_string());
    training_start.set("2024-01-01".to_string());
    training_end.set("2026-06-27".to_string());
    auto_tune.set(true);
    seasonality.set(true);
    arima_p.set("2".to_string());
    arima_d.set("1".to_string());
    arima_q.set("2".to_string());
    ets_error.set("A".to_string());
    ets_trend.set("A".to_string());
    ets_seasonal.set("A".to_string());
    prophet_growth.set("linear".to_string());
    prophet_seasonality.set("weekly".to_string());
    neural_layers.set("2".to_string());
    neural_units.set("64".to_string());
    neural_epochs.set("100".to_string());
}

fn populate_from_config(
    cfg: &serde_json::Value,
    model_name: &mut Signal<String>,
    algorithm: &mut Signal<String>,
    arima_p: &mut Signal<String>,
    arima_d: &mut Signal<String>,
    arima_q: &mut Signal<String>,
    ets_error: &mut Signal<String>,
    ets_trend: &mut Signal<String>,
    ets_seasonal: &mut Signal<String>,
    prophet_growth: &mut Signal<String>,
    prophet_seasonality: &mut Signal<String>,
    neural_layers: &mut Signal<String>,
    neural_units: &mut Signal<String>,
    neural_epochs: &mut Signal<String>,
) {
    model_name.set(cfg["model_name"].as_str().unwrap_or("").to_string());
    algorithm.set(cfg["model_type"].as_str().unwrap_or("arima").to_string());
    let params = &cfg["params"];
    if params.is_object() {
        arima_p.set(params["p"].as_str().unwrap_or("2").to_string());
        arima_d.set(params["d"].as_str().unwrap_or("1").to_string());
        arima_q.set(params["q"].as_str().unwrap_or("2").to_string());
        ets_error.set(params["error"].as_str().unwrap_or("A").to_string());
        ets_trend.set(params["trend"].as_str().unwrap_or("A").to_string());
        ets_seasonal.set(params["seasonal"].as_str().unwrap_or("A").to_string());
        prophet_growth.set(params["growth"].as_str().unwrap_or("linear").to_string());
        prophet_seasonality.set(params["seasonality_mode"].as_str().unwrap_or("weekly").to_string());
        neural_layers.set(params["layers"].as_str().unwrap_or("2").to_string());
        neural_units.set(params["units"].as_str().unwrap_or("64").to_string());
        neural_epochs.set(params["epochs"].as_str().unwrap_or("100").to_string());
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ForecastModelConfigPage() -> Element {
    let mut toast = use_toast();
    let api = use_auth().api;

    // Config list — loaded on mount via use_resource
    let configs = use_resource(move || {
        let api = api.clone();
        async move {
            api.with(|c| c.clone()).list_forecast_configs().await.unwrap_or_default()
        }
    });

    let mut selected_config_id: Signal<Option<i64>> = use_signal(|| None);

    // Form signals
    let mut model_name = use_signal(|| "Demand Forecast ARIMA".to_string());
    let mut algorithm = use_signal(|| "arima".to_string());
    let mut training_start = use_signal(|| "2024-01-01".to_string());
    let mut training_end = use_signal(|| "2026-06-27".to_string());
    let mut auto_tune = use_signal(|| true);
    let mut seasonality = use_signal(|| true);
    let mut is_saving = use_signal(|| false);
    let mut is_testing = use_signal(|| false);
    let mut status = use_signal(|| "ready".to_string());

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

    // Build selector options from loaded configs
    let config_options = use_memo(move || {
        let snapshot = configs.read();
        let list = snapshot.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let mut opts = vec![SelectOption {
            value: "new".to_string(),
            label: "New Config…".to_string(),
        }];
        for cfg in list {
            let id = cfg["id"].as_i64().unwrap_or(0);
            let name = cfg["model_name"].as_str().unwrap_or("Unnamed");
            opts.push(SelectOption {
                value: id.to_string(),
                label: name.to_string(),
            });
        }
        opts
    });

    // Handle config selection — uses a snapshot of the config list captured at call time
    let on_select_config = {
        // Clone the current config list into the closure so it owns its data
        let snapshot: Vec<serde_json::Value> = configs
            .read()
            .as_ref()
            .map(|v| v.clone())
            .unwrap_or_default();

        move |val: String| {
            if val == "new" {
                selected_config_id.set(None);
                reset_form(
                    &mut model_name, &mut algorithm, &mut training_start, &mut training_end,
                    &mut auto_tune, &mut seasonality,
                    &mut arima_p, &mut arima_d, &mut arima_q,
                    &mut ets_error, &mut ets_trend, &mut ets_seasonal,
                    &mut prophet_growth, &mut prophet_seasonality,
                    &mut neural_layers, &mut neural_units, &mut neural_epochs,
                );
            } else if let Ok(id) = val.parse::<i64>() {
                selected_config_id.set(Some(id));
                if let Some(cfg) = snapshot.iter().find(|c| c["id"].as_i64() == Some(id)) {
                    populate_from_config(
                        cfg,
                        &mut model_name, &mut algorithm,
                        &mut arima_p, &mut arima_d, &mut arima_q,
                        &mut ets_error, &mut ets_trend, &mut ets_seasonal,
                        &mut prophet_growth, &mut prophet_seasonality,
                        &mut neural_layers, &mut neural_units, &mut neural_epochs,
                    );
                }
            }
        }
    };

    // Save handler
    let t_save = toast.clone();
    let on_save = {
        let mut is_saving = is_saving.clone();
        let mut status = status.clone();
        let api = api.clone();
        let selected_config_id = selected_config_id.clone();
        let configs = configs.clone();
        move |_| {
            let mut t = t_save.clone();
            let api = api.clone();
            is_saving.set(true);
            status.set("training".to_string());
            let n = model_name.read().clone();
            let algo = algorithm.read().clone();
            let payload = serde_json::json!({
                "model_name": n.clone(),
                "model_type": algo,
                "training_start": training_start.read().clone(),
                "training_end": training_end.read().clone(),
                "auto_tune": *auto_tune.read(),
                "seasonality": *seasonality.read(),
                "params": match algo.as_str() {
                    "arima" => serde_json::json!({"p": arima_p.read().clone(), "d": arima_d.read().clone(), "q": arima_q.read().clone()}),
                    "ets" => serde_json::json!({"error": ets_error.read().clone(), "trend": ets_trend.read().clone(), "seasonal": ets_seasonal.read().clone()}),
                    "prophet" => serde_json::json!({"growth": prophet_growth.read().clone(), "seasonality_mode": prophet_seasonality.read().clone()}),
                    "neural" => serde_json::json!({"layers": neural_layers.read().clone(), "units": neural_units.read().clone(), "epochs": neural_epochs.read().clone()}),
                    _ => serde_json::json!({}),
                },
            });
            let mut configs = configs.clone();
            let selected_id = *selected_config_id.read();
            spawn(async move {
                let client = api.with(|c| c.clone());
                let result = if let Some(id) = selected_id {
                    client.update_forecast_config(id, &payload).await
                } else {
                    client.create_forecast_config(&payload).await
                };
                match result {
                    Ok(_) => {
                        is_saving.set(false);
                        status.set("saved".to_string());
                        configs.restart();
                        t.success("Model Saved", &format!("\"{}\" configuration saved.", n));
                    }
                    Err(e) => {
                        is_saving.set(false);
                        status.set("ready".to_string());
                        t.error("Save Failed", &e);
                    }
                }
            });
        }
    };

    // Test Run handler
    let t_test = toast.clone();
    let on_test = {
        let mut is_testing = is_testing.clone();
        let mut status = status.clone();
        let api = api.clone();
        move |_| {
            let mut t = t_test.clone();
            let api = api.clone();
            is_testing.set(true);
            status.set("training".to_string());
            let n = model_name.read().clone();
            let algo = algorithm.read().clone();
            let params = serde_json::json!({
                "model_name": n,
                "algorithm": algo,
                "training_start": training_start.read().clone(),
                "training_end": training_end.read().clone(),
                "auto_tune": *auto_tune.read(),
                "seasonality": *seasonality.read(),
            });
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.run_forecast(&params).await {
                    Ok(_) => {
                        is_testing.set(false);
                        status.set("ready".to_string());
                        t.success("Test Run Complete", &format!("\"{}\" test run completed.", n));
                    }
                    Err(e) => {
                        is_testing.set(false);
                        status.set("ready".to_string());
                        t.error("Test Run Failed", &e);
                    }
                }
            });
        }
    };

    let status_cls = match status.read().as_str() {
        "saved" => "mc-status mc-status-ready",
        "training" => "mc-status mc-status-training",
        _ => "mc-status mc-status-ready",
    };

    let status_text = match status.read().as_str() {
        "saved" => "✅ Model trained and saved",
        "training" => "⏳ Training in progress…",
        _ => "🟢 Model ready for configuration",
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page mc-page",

            div { class: "mc-header",
                h1 { "Forecast Model Configuration" }
            }

            // ── Config selector ──
            div { class: "mc-section mc-selector",
                h2 { "Select Configuration" }
                div { class: "mc-selector-row",
                    div { class: "mc-selector-dropdown",
                        SearchableSelect {
                            options: config_options.read().clone(),
                            selected_value: Some(
                                (*selected_config_id.read()).map(|id| id.to_string()).unwrap_or_else(|| "new".to_string())
                            ),
                            on_select: on_select_config,
                            placeholder: "Select a config…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| {
                            selected_config_id.set(None);
                            reset_form(
                                &mut model_name, &mut algorithm, &mut training_start, &mut training_end,
                                &mut auto_tune, &mut seasonality,
                                &mut arima_p, &mut arima_d, &mut arima_q,
                                &mut ets_error, &mut ets_trend, &mut ets_seasonal,
                                &mut prophet_growth, &mut prophet_seasonality,
                                &mut neural_layers, &mut neural_units, &mut neural_epochs,
                            );
                        },
                        icon: Some("+".to_string()),
                        "New"
                    }
                }
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
                    if selected_config_id.read().is_some() { "Update Model" } else { "Save Model" }
                }
            }
        }
    }
}
