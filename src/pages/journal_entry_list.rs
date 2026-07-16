//! Journal Entry List Page — List view with date range filters.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant};
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.je-page { max-width: 1100px; margin: 0 auto; padding: 20px; }
.je-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; flex-wrap: wrap; gap: 12px; }
.je-header h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.je-filters { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.je-filter-group { display: flex; flex-direction: column; gap: 4px; }
.je-filter-group label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; }
.je-filter-group input { padding: 6px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; font-size: 13px; background: #fff; color: var(--text-primary); }
.je-table-container { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; overflow: hidden; }
.je-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.je-table thead th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.je-table thead th.text-right { text-align: right; }
.je-table tbody td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); color: var(--text-primary); }
.je-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.je-table tbody tr:last-child td { border-bottom: none; }
.je-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.je-type-badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
.je-type-invoice { background: rgba(74,144,217,0.1); color: #4a90d9; }
.je-type-payment { background: rgba(40,167,69,0.1); color: #28a745; }
.je-type-purchase { background: rgba(255,193,7,0.15); color: #d4a017; }
.je-type-expense { background: rgba(220,53,69,0.12); color: #dc3545; }
.je-type-salary { background: rgba(108,117,125,0.1); color: #6c757d; }
.je-type-default { background: rgba(108,117,125,0.1); color: #6c757d; }
.je-empty { text-align: center; padding: 40px 20px; color: var(--text-secondary); font-size: 14px; }
.je-summary { display: flex; gap: 24px; padding: 12px 16px; background: var(--bg-muted, #f8f9fa); border-top: 2px solid var(--border-color); font-size: 13px; }
.je-summary-item { display: flex; gap: 6px; }
.je-summary-label { color: var(--text-secondary); }
.je-summary-value { font-weight: 600; }
"#;

#[derive(Clone, Debug)]
struct JournalEntry {
    id: i64,
    reference_type: Option<String>,
    reference_id: Option<i64>,
    entry_date: String,
}

fn type_badge_class(t: &str) -> &'static str {
    match t {
        "invoice" => "je-type-invoice",
        "payment" => "je-type-payment",
        "purchase_order" | "purchase" => "je-type-purchase",
        "expense" => "je-type-expense",
        "salary" => "je-type-salary",
        _ => "je-type-default",
    }
}

#[component]
pub fn JournalEntryListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let counter = use_signal(|| 0u32);

    let mut from_date = use_signal(|| {
        let now = chrono::Utc::now();
        now.format("%Y-01-01").to_string()
    });
    let mut to_date = use_signal(|| {
        let now = chrono::Utc::now();
        now.format("%Y-12-31").to_string()
    });

    let resource = use_resource(move || {
        let api = api.clone();
        let from = from_date.read().clone();
        let to = to_date.read().clone();
        let _ = *counter.read();
        async move {
            let client = api.read().clone();
            match client.list_journal_entries(&from, &to).await {
                Ok(entries) => entries.into_iter().map(|e| JournalEntry {
                    id: e.id,
                    reference_type: e.reference_type,
                    reference_id: e.reference_id,
                    entry_date: e.entry_date,
                }).collect(),
                Err(_) => vec![],
            }
        }
    });

    let is_loading = resource.read().is_none();
    let entries = resource.read().cloned().unwrap_or_default();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "je-page",
            div { class: "je-header",
                h1 { "Journal Entries" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| { navigator.push("/accounting/journal-entries/new"); },
                    "＋ New Entry"
                }
            }

            div { class: "je-filters",
                div { class: "je-filter-group",
                    label { "From" }
                    input {
                        r#type: "date",
                        value: "{from_date}",
                        onchange: move |e| { from_date.set(e.value()); },
                    }
                }
                div { class: "je-filter-group",
                    label { "To" }
                    input {
                        r#type: "date",
                        value: "{to_date}",
                        onchange: move |e| { to_date.set(e.value()); },
                    }
                }
            }

            if is_loading {
                div { class: "je-empty", "Loading..." }
            } else if entries.is_empty() {
                div { class: "je-empty", "No journal entries found for this date range." }
            } else {{
                let total_debit: f64 = 0.0;
                let total_credit: f64 = 0.0;
                rsx! {
                    div { class: "je-table-container",
                        table { class: "je-table",
                            thead { tr {
                                th { "Date" }
                                th { "Entry #" }
                                th { "Type" }
                                th { "Reference" }
                                th { class: "text-right", "Actions" }
                            }}
                            tbody {
                                for entry in entries.iter() {
                                    {let ref_type = entry.reference_type.as_deref().unwrap_or("manual");
                                    let badge_cls = type_badge_class(ref_type);
                                    rsx! {
                                        tr {
                                            td { "{entry.entry_date}" }
                                            td { style: "font-family: monospace; font-size: 12px;", "JE-{entry.id:05}" }
                                            td {
                                                span { class: "je-type-badge {badge_cls}", "{ref_type}" }
                                            }
                                            td {
                                                if let Some(ref rt) = entry.reference_type {
                                                    if let Some(rid) = entry.reference_id {
                                                        span { style: "font-family: monospace; font-size: 12px;", "{rt} #{rid}" }
                                                    } else {
                                                        span { "—" }
                                                    }
                                                } else {
                                                    span { "Manual" }
                                                }
                                            }
                                            td { class: "text-right",
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    onclick: { let nav = navigator.clone(); let eid = entry.id; move |_| { nav.push(format!("/accounting/journal-entries/{}", eid)); } },
                                                    "View"
                                                }
                                            }
                                        }
                                    }}
                                }
                            }
                        }
                    }
                }
            }}
        }
    }
}

// ============================================================================
// Journal Entry Detail Page
// ============================================================================

const DETAIL_CSS: &str = r#"
.jed-page { max-width: 900px; margin: 0 auto; padding: 20px; }
.jed-header { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; }
.jed-header h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.jed-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; padding: 20px; margin-bottom: 16px; }
.jed-info { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; margin-bottom: 20px; }
.jed-field { display: flex; flex-direction: column; gap: 3px; }
.jed-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; }
.jed-field-value { font-size: 14px; color: var(--text-primary); }
.jed-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.jed-table thead th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.jed-table thead th.text-right { text-align: right; }
.jed-table tbody td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); }
.jed-table tbody td.text-right { text-align: right; font-family: monospace; }
.jed-table tbody tr:last-child td { border-bottom: none; }
.jed-totals { display: flex; gap: 24px; padding: 12px 16px; background: var(--bg-muted, #f8f9fa); border-top: 2px solid var(--border-color); font-size: 13px; }
.jed-totals-item { display: flex; gap: 6px; }
.jed-totals-label { color: var(--text-secondary); }
.jed-totals-value { font-weight: 600; font-family: monospace; }
.jed-loading { text-align: center; padding: 40px; color: var(--text-secondary); }
"#;

#[derive(Clone, Debug)]
struct JournalEntryDetail {
    id: i64,
    reference_type: Option<String>,
    reference_id: Option<i64>,
    entry_date: String,
    created_by: Option<i64>,
    created_at: String,
}

#[derive(Clone, Debug)]
struct JournalLineDetail {
    account_code: Option<String>,
    account_name: Option<String>,
    debit: f64,
    credit: f64,
    description: String,
}

#[component]
pub fn JournalEntryDetailPage(id: String) -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let resource = use_resource(move || {
        let api = api.clone();
        let id = id.clone();
        async move {
            let parsed = id.parse::<i64>().unwrap_or(0);
            if parsed == 0 { return None; }
            let client = api.read().clone();
            match client.get_journal_entry(parsed).await {
                Ok(data) => {
                    let entry = &data["entry"];
                    let lines = &data["lines"];
                    let journal_lines: Vec<JournalLineDetail> = lines.as_array().unwrap_or(&vec![]).iter().map(|l| {
                        JournalLineDetail {
                            account_code: l["account_code"].as_str().map(|s| s.to_string()),
                            account_name: l["account_name"].as_str().map(|s| s.to_string()),
                            debit: l["debit"].as_f64().unwrap_or(0.0),
                            credit: l["credit"].as_f64().unwrap_or(0.0),
                            description: l["description"].as_str().unwrap_or("").to_string(),
                        }
                    }).collect();
                    Some((
                        JournalEntryDetail {
                            id: entry["id"].as_i64().unwrap_or(0),
                            reference_type: entry["reference_type"].as_str().map(|s| s.to_string()),
                            reference_id: entry["reference_id"].as_i64(),
                            entry_date: entry["entry_date"].as_str().unwrap_or("").to_string(),
                            created_by: entry["created_by"].as_i64(),
                            created_at: entry["created_at"].as_str().unwrap_or("").to_string(),
                        },
                        journal_lines,
                    ))
                }
                Err(_) => None,
            }
        }
    });

    let snap = resource.read();
    let is_loading = snap.is_none();
    let data = snap.as_ref().and_then(|d| d.clone());

    rsx! {
        style { "{DETAIL_CSS}" }
        div { class: "jed-page",
            div { class: "jed-header",
                Button { variant: ButtonVariant::Ghost, onclick: move |_| { navigator.push("/accounting/journal-entries"); }, "← Back" }
                h1 { "Journal Entry Detail" }
            }

            if is_loading {
                div { class: "jed-loading", "Loading..." }
            } else if let Some((entry, lines)) = data {{
                let total_debit: f64 = lines.iter().map(|l| l.debit).sum();
                let total_credit: f64 = lines.iter().map(|l| l.credit).sum();

                rsx! {
                    div { class: "jed-card",
                        div { class: "jed-info",
                            div { class: "jed-field",
                                span { class: "jed-field-label", "Entry #" }
                                span { class: "jed-field-value", "JE-{entry.id:05}" }
                            }
                            div { class: "jed-field",
                                span { class: "jed-field-label", "Date" }
                                span { class: "jed-field-value", "{entry.entry_date}" }
                            }
                            {let ref_type = entry.reference_type.as_deref().unwrap_or("Manual");
                            rsx! {
                                div { class: "jed-field",
                                    span { class: "jed-field-label", "Type" }
                                    span { class: "jed-field-value", "{ref_type}" }
                                }
                                div { class: "jed-field",
                                    span { class: "jed-field-label", "Reference" }
                                    span { class: "jed-field-value",
                                        if let Some(rid) = entry.reference_id {
                                            "{ref_type} #{rid}"
                                        } else {
                                            "—"
                                        }
                                    }
                                }
                            }}
                            div { class: "jed-field",
                                span { class: "jed-field-label", "Created At" }
                                span { class: "jed-field-value", "{entry.created_at}" }
                            }
                        }
                    }

                    div { class: "jed-card",
                        h2 { style: "font-size: 15px; font-weight: 600; margin: 0 0 16px 0;", "Journal Lines" }
                        table { class: "jed-table",
                            thead { tr {
                                th { "Account" }
                                th { "Description" }
                                th { class: "text-right", "Debit" }
                                th { class: "text-right", "Credit" }
                            }}
                            tbody {
                                for line in lines.iter() {{
                                    let code = line.account_code.as_deref().unwrap_or("?");
                                    let name = line.account_name.as_deref().unwrap_or("Unknown");
                                    rsx! {
                                        tr {
                                            td { style: "font-weight: 500;", "{code} — {name}" }
                                            td { style: "color: var(--text-secondary);", "{line.description}" }
                                            td { class: "text-right",
                                                if line.debit > 0.0 { "PKR {line.debit:.2}" }
                                            }
                                            td { class: "text-right",
                                                if line.credit > 0.0 { "PKR {line.credit:.2}" }
                                            }
                                        }
                                    }
                                }}
                            }
                        }
                        div { class: "jed-totals",
                            div { class: "jed-totals-item",
                                span { class: "jed-totals-label", "Total Debit:" }
                                span { class: "jed-totals-value", "PKR {total_debit:.2}" }
                            }
                            div { class: "jed-totals-item",
                                span { class: "jed-totals-label", "Total Credit:" }
                                span { class: "jed-totals-value", "PKR {total_credit:.2}" }
                            }
                            div { class: "jed-totals-item",
                                span { class: "jed-totals-label", "Balance:"
                                    if (total_debit - total_credit).abs() < 0.01 {
                                        " ✓"
                                    } else {
                                        " ✗"
                                    }
                                }
                                span { class: "jed-totals-value",
                                    if (total_debit - total_credit).abs() < 0.01 {
                                        span { style: "color: #28a745;", "Balanced" }
                                    } else {
                                        span { style: "color: #dc3545;", "Unbalanced" }
                                    }
                                }
                            }
                        }
                    }
                }
            }} else {
                div { class: "jed-loading", "Journal entry not found." }
            }
        }
    }
}
