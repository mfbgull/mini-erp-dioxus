//! FIFO Report Page — Shows FIFO batch costing, P&L by item, valuation, and stock history.

use crate::auth::use_auth;
use crate::components::common::{use_toast, StatCard, StatCardVariant};
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.fr-page { max-width: 1200px; margin: 0 auto; }
.fr-header { margin-bottom: 20px; }
.fr-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.fr-header p { font-size: 13px; color: var(--text-secondary); margin: 4px 0 0; }

.fr-tabs { display: flex; gap: 2px; background: var(--bg-secondary, #f5f5f5); border-radius: 8px; padding: 3px; margin-bottom: 20px; width: fit-content; }
.fr-tab { padding: 8px 16px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; border: none; background: transparent; color: var(--text-secondary); transition: all 0.15s; }
.fr-tab:hover { color: var(--text-primary); }
.fr-tab.active { background: #fff; color: var(--text-primary); box-shadow: 0 1px 3px rgba(0,0,0,0.08); }

.fr-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }

.fr-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 16px; }
.fr-section h2 { font-size: 15px; font-weight: 600; margin: 0 0 12px; color: var(--text-primary); }

.fr-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.fr-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.fr-table thead th.text-right { text-align: right; }
.fr-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.fr-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.fr-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.fr-badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
.fr-badge.correct { background: #dcfce7; color: #166534; }
.fr-badge.incorrect { background: #fef2f2; color: #991b1b; }
.fr-badge.in { background: #dbeafe; color: #1e40af; }
.fr-badge.out { background: #fef3c7; color: #92400e; }
.fr-badge.adjustment { background: #f3e8ff; color: #6b21a8; }

.fr-loading { text-align: center; padding: 60px; color: var(--text-secondary); font-size: 14px; }
.fr-empty { text-align: center; padding: 40px; color: var(--text-secondary); font-size: 13px; }

.fr-item-select { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
.fr-item-select label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.fr-item-select select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; min-width: 250px; }

.fr-btn { padding: 8px 16px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; border: 1px solid var(--border-color, #e0e0e0); background: #fff; color: var(--text-primary); transition: all 0.15s; }
.fr-btn:hover { background: var(--bg-secondary, #f5f5f5); }
.fr-btn.primary { background: #3b82f6; color: #fff; border-color: #3b82f6; }
.fr-btn.primary:hover { background: #2563eb; }
.fr-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.fr-trace-step { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.fr-trace-step:last-child { border-bottom: none; }
.fr-step-num { width: 28px; height: 28px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700; color: #fff; flex-shrink: 0; }
.fr-step-num.buy { background: #3b82f6; }
.fr-step-num.sell { background: #f59e0b; }
.fr-step-detail { flex: 1; }
.fr-step-detail .label { font-size: 12px; color: var(--text-secondary); }
.fr-step-detail .value { font-size: 14px; font-weight: 600; color: var(--text-primary); }
"##;

#[component]
pub fn FifoReportPage() -> Element {
    let _toast = use_toast();
    let api = use_auth().api;
    let mut active_tab = use_signal(|| "test".to_string());
    let mut test_result = use_signal(|| Option::<serde_json::Value>::None);
    let mut test_loading = use_signal(|| false);
    let mut by_item_data = use_signal(|| Option::<serde_json::Value>::None);
    let mut by_item_loading = use_signal(|| false);
    let mut history_data = use_signal(|| Option::<serde_json::Value>::None);
    let mut history_loading = use_signal(|| false);
    let mut history_item_id = use_signal(|| "1".to_string());

    // Use use_resource for initial valuation load (runs once, not on every render)
    let valuation_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_stock_valuation_fifo().await.unwrap_or_default()
        }
    });

    let valuation_data = valuation_resource.read().clone().unwrap_or_default();
    let valuation_loading = valuation_resource.read().is_none();

    // Extract item list for stock history dropdown
    let item_options: Vec<(i64, String)> = valuation_data
        .get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().filter_map(|i| {
            let id = i.get("id")?.as_i64()?;
            let code = i.get("item_code")?.as_str()?;
            let name = i.get("item_name")?.as_str()?;
            Some((id, format!("{} - {}", code, name)))
        }).collect();

    rsx! {
        style { {PAGE_CSS} }
        div { class: "fr-page",
            div { class: "fr-header",
                h1 { "FIFO Inventory Reports" }
                p { "Batch-level FIFO costing, valuation, and P&L analysis" }
            }

            div { class: "fr-tabs",
                button { class: if *active_tab.read() == "test" { "fr-tab active" } else { "fr-tab" },
                    onclick: move |_| active_tab.set("test".to_string()),
                    "Test Scenario"
                }
                button { class: if *active_tab.read() == "valuation" { "fr-tab active" } else { "fr-tab" },
                    onclick: move |_| active_tab.set("valuation".to_string()),
                    "FIFO Valuation"
                }
                button { class: if *active_tab.read() == "by-item" { "fr-tab active" } else { "fr-tab" },
                    onclick: move |_| active_tab.set("by-item".to_string()),
                    "P&L by Item"
                }
                button { class: if *active_tab.read() == "history" { "fr-tab active" } else { "fr-tab" },
                    onclick: move |_| active_tab.set("history".to_string()),
                    "Stock History"
                }
            }

            if *active_tab.read() == "test" {
                div { class: "fr-section",
                    div { style: "display:flex; align-items:center; justify-content:space-between; margin-bottom:16px;",
                        div {
                            h2 { "FIFO Test Scenario" }
                            p { style: "font-size:13px; color:var(--text-secondary); margin:2px 0 0;",
                                "Buy at 100, sell at 110, buy at 105, sell at 115"
                            }
                        }
                        button { class: "fr-btn primary",
                            onclick: move |_| {
                                let api = api.clone();
                                test_loading.set(true);
                                spawn(async move {
                                    let client = api.with(|c| c.clone());
                                    match client.test_fifo_scenario().await {
                                        Ok(data) => { test_result.set(Some(data)); }
                                        Err(e) => { tracing::error!("FIFO test failed: {}", e); }
                                    }
                                    test_loading.set(false);
                                });
                            },
                            disabled: *test_loading.read(),
                            { if *test_loading.read() { "Running..." } else { "Run Test" } }
                        }
                    }
                    if let Some(result) = test_result.read().as_ref() {
                        if let Some(summary) = result.get("summary") {
                            div { class: "fr-kpi-grid",
                                StatCard { title: "Total Revenue".to_string(), value: format!("${:.2}", summary.get("total_revenue").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Success }
                                StatCard { title: "Total COGS".to_string(), value: format!("${:.2}", summary.get("total_cogs").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Warning }
                                StatCard { title: "Gross Profit".to_string(), value: format!("${:.2}", summary.get("gross_profit").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                                StatCard { title: "FIFO Correct".to_string(),
                                    value: if result.get("fifo_correct").and_then(|v| v.as_bool()).unwrap_or(false) { "YES".to_string() } else { "NO".to_string() },
                                    variant: if result.get("fifo_correct").and_then(|v| v.as_bool()).unwrap_or(false) { StatCardVariant::Success } else { StatCardVariant::Danger } }
                            }
                        }
                        div { class: "fr-section",
                            h2 { "Transaction Trace" }
                            if let Some(trace) = result.get("trace").and_then(|v| v.as_array()) {
                                for step in trace.iter() {
                                    {
                                        let action = step.get("action").and_then(|v| v.as_str()).unwrap_or("");
                                        let step_num = step.get("step").and_then(|v| v.as_i64()).unwrap_or(0);
                                        let qty = step.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let cost = step.get("fifo_unit_cost").or(step.get("unit_cost")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let expected = step.get("expected_cost").and_then(|v| v.as_f64());
                                        let correct = step.get("correct").and_then(|v| v.as_bool());
                                        rsx! {
                                            div { class: "fr-trace-step",
                                                div { class: format!("fr-step-num {}", action.to_lowercase()),
                                                    { format!("{}", step_num) }
                                                }
                                                div { class: "fr-step-detail",
                                                    div { class: "label", { if action == "BUY" { "Purchase (IN)".to_string() } else { "Sale (OUT)".to_string() } } }
                                                    div { class: "value", { format!("Qty: {} x ${:.2} = ${:.2}", qty, cost, qty * cost) } }
                                                }
                                                if let Some(exp) = expected {
                                                    if let Some(ok) = correct {
                                                        span { class: if ok { "fr-badge correct" } else { "fr-badge incorrect" },
                                                            { if ok { "CORRECT".to_string() } else { format!("EXPECTED ${:.2}", exp) } }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if *test_loading.read() {
                        div { class: "fr-loading", "Running FIFO test scenario..." }
                    } else {
                        div { class: "fr-empty", "Click Run Test to execute the FIFO scenario" }
                    }
                }
            }

            if *active_tab.read() == "valuation" {
                if valuation_loading {
                    div { class: "fr-loading", "Loading FIFO valuation..." }
                } else {
                    if let Some(summary) = valuation_data.get("summary") {
                        div { class: "fr-kpi-grid",
                            StatCard { title: "Total FIFO Value".to_string(), value: format!("${:.2}", summary.get("total_value").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                            StatCard { title: "Total Quantity".to_string(), value: format!("{:.0}", summary.get("total_quantity").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Success }
                            StatCard { title: "Active Batches".to_string(), value: format!("{}", summary.get("total_batches").and_then(|v| v.as_i64()).unwrap_or(0)), variant: StatCardVariant::Warning }
                            StatCard { title: "Avg FIFO Cost".to_string(), value: format!("${:.2}", summary.get("avg_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                        }
                    }
                    div { class: "fr-section",
                        h2 { "Stock Value by Item (FIFO)" }
                        table { class: "fr-table",
                            thead { tr { th { "Item Code" } th { "Item Name" } th { class: "text-right", "Stock Qty" } th { class: "text-right", "FIFO Value" } th { class: "text-right", "Avg Cost" } th { class: "text-right", "Batches" } } }
                            tbody {
                                if let Some(items) = valuation_data.get("items").and_then(|v| v.as_array()) {
                                    for item in items.iter() {
                                        tr {
                                            td { { item.get("item_code").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { { item.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { class: "text-right", { format!("{:.0}", item.get("stock_qty").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", item.get("fifo_value").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", item.get("avg_fifo_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("{}", item.get("batch_count").and_then(|v| v.as_i64()).unwrap_or(0)) } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "fr-section",
                        h2 { "Batch Detail" }
                        table { class: "fr-table",
                            thead { tr { th { "Batch No" } th { "Item" } th { "Warehouse" } th { class: "text-right", "Original" } th { class: "text-right", "Remaining" } th { class: "text-right", "Unit Cost" } th { class: "text-right", "Value" } th { "Source" } } }
                            tbody {
                                if let Some(batches) = valuation_data.get("batches").and_then(|v| v.as_array()) {
                                    for b in batches.iter() {
                                        tr {
                                            td { { b.get("batch_no").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { { b.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { { b.get("warehouse_name").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { class: "text-right", { format!("{:.0}", b.get("quantity_original").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("{:.0}", b.get("quantity_remaining").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", b.get("unit_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", b.get("batch_value").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { { b.get("source_type").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *active_tab.read() == "by-item" {
                div { class: "fr-section",
                    div { style: "display:flex; align-items:center; justify-content:space-between; margin-bottom:16px;",
                        h2 { "Profit & Loss by Item (FIFO COGS)" }
                        button { class: "fr-btn primary",
                            onclick: move |_| {
                                let api = api.clone();
                                by_item_loading.set(true);
                                spawn(async move {
                                    let client = api.with(|c| c.clone());
                                    match client.get_profit_loss_by_item().await {
                                        Ok(data) => { by_item_data.set(Some(data)); }
                                        Err(e) => { tracing::error!("Failed to load P&L by item: {}", e); }
                                    }
                                    by_item_loading.set(false);
                                });
                            },
                            disabled: *by_item_loading.read(),
                            { if *by_item_loading.read() { "Loading..." } else { "Load Report" } }
                        }
                    }
                    if *by_item_loading.read() {
                        div { class: "fr-loading", "Loading P&L by item..." }
                    } else if let Some(data) = by_item_data.read().as_ref() {
                        if let Some(summary) = data.get("summary") {
                            div { class: "fr-kpi-grid",
                                StatCard { title: "Total Revenue".to_string(), value: format!("${:.2}", summary.get("total_revenue").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Success }
                                StatCard { title: "Total COGS".to_string(), value: format!("${:.2}", summary.get("total_cogs").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Warning }
                                StatCard { title: "Gross Profit".to_string(), value: format!("${:.2}", summary.get("total_gross_profit").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                                StatCard { title: "Net Profit".to_string(), value: format!("${:.2}", summary.get("net_profit").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                            }
                        }
                        table { class: "fr-table",
                            thead { tr { th { "Item Code" } th { "Item Name" } th { class: "text-right", "Qty Sold" } th { class: "text-right", "Revenue" } th { class: "text-right", "COGS (FIFO)" } th { class: "text-right", "Gross Profit" } th { class: "text-right", "Margin %" } } }
                            tbody {
                                if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
                                    for item in items.iter() {
                                        tr {
                                            td { { item.get("item_code").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { { item.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                            td { class: "text-right", { format!("{:.0}", item.get("qty_sold").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", item.get("revenue").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", item.get("cogs").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("${:.2}", item.get("gross_profit").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                            td { class: "text-right", { format!("{:.1}%", item.get("margin_pct").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "fr-empty", "Click Load Report to view P&L by item" }
                    }
                }
            }

            if *active_tab.read() == "history" {
                div { class: "fr-section",
                    div { class: "fr-item-select",
                        label { "Select Item:" }
                        select { onchange: move |e: Event<FormData>| { history_item_id.set(e.value()); },
                            for opt in item_options.iter() {
                                option { value: opt.0.to_string(), { opt.1.clone() } }
                            }
                        }
                        button { class: "fr-btn primary",
                            onclick: move |_| {
                                let api = api.clone();
                                let item_id_str = history_item_id.read().clone();
                                let item_id: i64 = item_id_str.parse().unwrap_or(1);
                                history_loading.set(true);
                                spawn(async move {
                                    let client = api.with(|c| c.clone());
                                    match client.get_stock_history(item_id).await {
                                        Ok(data) => { history_data.set(Some(data)); }
                                        Err(e) => { tracing::error!("Failed to load stock history: {}", e); }
                                    }
                                    history_loading.set(false);
                                });
                            },
                            disabled: *history_loading.read(),
                            { if *history_loading.read() { "Loading..." } else { "Load History" } }
                        }
                    }
                    if *history_loading.read() {
                        div { class: "fr-loading", "Loading stock history..." }
                    } else if let Some(data) = history_data.read().as_ref() {
                        if let Some(item) = data.get("item") {
                            div { class: "fr-kpi-grid",
                                StatCard { title: format!("{} - {}", item.get("item_code").and_then(|v| v.as_str()).unwrap_or(""), item.get("item_name").and_then(|v| v.as_str()).unwrap_or("")), value: format!("Stock: {:.0}", item.get("current_stock").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                            }
                        }
                        if let Some(summary) = data.get("summary") {
                            div { class: "fr-kpi-grid",
                                StatCard { title: "Total Received".to_string(), value: format!("{:.0}", summary.get("total_received").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Success }
                                StatCard { title: "Total Issued".to_string(), value: format!("{:.0}", summary.get("total_issued").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Warning }
                                StatCard { title: "Current Stock".to_string(), value: format!("{:.0}", summary.get("current_stock").and_then(|v| v.as_f64()).unwrap_or(0.0)), variant: StatCardVariant::Primary }
                            }
                        }
                        div { class: "fr-section",
                            h2 { "Movement History (with Running Balance)" }
                            table { class: "fr-table",
                                thead { tr { th { "Date" } th { "Movement No" } th { "Type" } th { class: "text-right", "Qty" } th { class: "text-right", "Unit Cost" } th { class: "text-right", "Total Cost" } th { "Warehouse" } th { "Batch" } th { class: "text-right", "Running Balance" } } }
                                tbody {
                                    if let Some(movements) = data.get("movements").and_then(|v| v.as_array()) {
                                        for m in movements.iter() {
                                            {
                                                let mtype = m.get("movement_type").and_then(|v| v.as_str()).unwrap_or("");
                                                let badge_class = match mtype { "IN" => "fr-badge in", "OUT" => "fr-badge out", "ADJUSTMENT" => "fr-badge adjustment", _ => "fr-badge" };
                                                rsx! {
                                                    tr {
                                                        td { { m.get("created_at").and_then(|v| v.as_str()).unwrap_or("").chars().take(10).collect::<String>() } }
                                                        td { { m.get("movement_no").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                                        td { span { class: badge_class, { mtype.to_string() } } }
                                                        td { class: "text-right", { format!("{:.0}", m.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                        td { class: "text-right", { format!("${:.2}", m.get("unit_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                        td { class: "text-right", { format!("${:.2}", m.get("total_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                        td { { m.get("warehouse_name").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                                        td { { m.get("batch_no").and_then(|v| v.as_str()).unwrap_or("-").to_string() } }
                                                        td { class: "text-right", { format!("{:.0}", m.get("running_balance").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(batches) = data.get("batches").and_then(|v| v.as_array()) {
                            if !batches.is_empty() {
                                div { class: "fr-section",
                                    h2 { "Batch History" }
                                    table { class: "fr-table",
                                        thead { tr { th { "Batch No" } th { class: "text-right", "Original Qty" } th { class: "text-right", "Remaining Qty" } th { class: "text-right", "Unit Cost" } th { class: "text-right", "Original Value" } th { class: "text-right", "Remaining Value" } th { "Received Date" } th { "Source" } } }
                                        tbody {
                                            for b in batches.iter() {
                                                tr {
                                                    td { { b.get("batch_no").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                                    td { class: "text-right", { format!("{:.0}", b.get("quantity_original").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    td { class: "text-right", { format!("{:.0}", b.get("quantity_remaining").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    td { class: "text-right", { format!("${:.2}", b.get("unit_cost").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    td { class: "text-right", { format!("${:.2}", b.get("original_value").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    td { class: "text-right", { format!("${:.2}", b.get("remaining_value").and_then(|v| v.as_f64()).unwrap_or(0.0)) } }
                                                    td { { b.get("received_date").and_then(|v| v.as_str()).unwrap_or("").chars().take(10).collect::<String>() } }
                                                    td { { b.get("source_type").and_then(|v| v.as_str()).unwrap_or("").to_string() } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "fr-empty", "Select an item and click Load History" }
                    }
                }
            }
        }
    }
}
