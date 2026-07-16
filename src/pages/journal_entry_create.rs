//! Journal Entry Create Page — Form for creating journal entries with balanced debits/credits.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, use_toast};
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.jec-page { max-width: 900px; margin: 0 auto; padding: 20px; }
.jec-header { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; }
.jec-header h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.jec-back { font-size: 13px; color: var(--accent, #4a90d9); cursor: pointer; background: none; border: none; padding: 0; }
.jec-back:hover { text-decoration: underline; }
.jec-form { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; padding: 20px; }
.jec-row { display: flex; gap: 16px; margin-bottom: 16px; flex-wrap: wrap; }
.jec-field { display: flex; flex-direction: column; gap: 4px; flex: 1; min-width: 150px; }
.jec-field label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
.jec-field input, .jec-field select { padding: 8px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; font-size: 13px; background: #fff; color: var(--text-primary); }
.jec-lines-header { display: flex; align-items: center; justify-content: space-between; margin: 20px 0 10px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color); }
.jec-lines-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.jec-lines-table { width: 100%; border-collapse: collapse; font-size: 13px; margin-bottom: 10px; }
.jec-lines-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color); }
.jec-lines-table thead th.text-right { text-align: right; }
.jec-lines-table tbody td { padding: 6px 8px; border-bottom: 1px solid var(--border-color); }
.jec-lines-table tbody td.text-right { text-align: right; font-family: monospace; }
.jec-lines-table input, .jec-lines-table select { width: 100%; padding: 6px 8px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; font-size: 12px; }
.jec-lines-table input[type="number"] { text-align: right; font-family: monospace; }
.jec-lines-table .remove-btn { background: none; border: none; color: #dc3545; cursor: pointer; font-size: 16px; padding: 4px; }
.jec-lines-table .remove-btn:hover { color: #a71d2a; }
.jec-totals { display: flex; gap: 24px; padding: 10px 0; font-size: 13px; border-top: 2px solid var(--border-color); }
.jec-totals-item { display: flex; gap: 6px; }
.jec-totals-label { color: var(--text-secondary); }
.jec-totals-value { font-weight: 600; font-family: monospace; }
.jec-totals-balanced { color: #28a745; }
.jec-totals-unbalanced { color: #dc3545; }
.jec-actions { display: flex; gap: 10px; margin-top: 20px; justify-content: flex-end; }
"#;

#[derive(Clone, Debug)]
struct JournalLine {
    account_id: i64,
    debit: String,
    credit: String,
    description: String,
}

#[derive(Clone, Debug)]
struct AccountOption {
    id: i64,
    code: String,
    name: String,
}

#[component]
pub fn JournalEntryCreatePage() -> Element {
    let navigator = use_navigator();
    let mut toast = use_toast();
    let api = use_auth().api;

    let mut entry_date = use_signal(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let mut reference_type = use_signal(|| String::new());
    let mut reference_id = use_signal(|| String::new());
    let mut lines = use_signal(|| vec![
        JournalLine { account_id: 0, debit: String::new(), credit: String::new(), description: String::new() },
        JournalLine { account_id: 0, debit: String::new(), credit: String::new(), description: String::new() },
    ]);
    let mut is_submitting = use_signal(|| false);

    // Fetch chart of accounts
    let accounts_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.read().clone();
            match client.list_accounts().await {
                Ok(accounts) => accounts.into_iter().map(|a| AccountOption {
                    id: a.id,
                    code: a.code,
                    name: a.name,
                }).collect(),
                Err(_) => vec![],
            }
        }
    });
    let accounts = accounts_resource.read().cloned().unwrap_or_default();

    let total_debit: f64 = lines.read().iter().filter_map(|l| l.debit.parse::<f64>().ok()).sum();
    let total_credit: f64 = lines.read().iter().filter_map(|l| l.credit.parse::<f64>().ok()).sum();
    let is_balanced = (total_debit - total_credit).abs() < 0.01 && total_debit > 0.0;

    let add_line = move |_| {
        lines.write().push(JournalLine {
            account_id: 0,
            debit: String::new(),
            credit: String::new(),
            description: String::new(),
        });
    };

    let mut remove_line = move |idx: usize| {
        let mut l = lines.write();
        if l.len() > 2 {
            l.remove(idx);
        }
    };

    let submit = move |_| {
        if !is_balanced {
            toast.error("Error", "Debits must equal credits.");
            return;
        }
        is_submitting.set(true);
        let entry_date = entry_date.read().clone();
        let ref_type = reference_type.read().clone();
        let ref_id: Option<i64> = reference_id.read().parse().ok();
        let lines_data: Vec<(i64, f64, f64, String)> = lines.read().iter().filter_map(|l| {
            let debit: f64 = l.debit.parse().ok().unwrap_or(0.0);
            let credit: f64 = l.credit.parse().ok().unwrap_or(0.0);
            if l.account_id > 0 && (debit > 0.0 || credit > 0.0) {
                Some((l.account_id, debit, credit, l.description.clone()))
            } else {
                None
            }
        }).collect();

        if lines_data.is_empty() {
            toast.error("Error", "Add at least one valid line.");
            is_submitting.set(false);
            return;
        }

        let api = api.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        spawn(async move {
            let client = api.read().clone();
            match client.create_journal_entry(&entry_date, ref_type.as_str(), ref_id, &lines_data).await {
                Ok(_) => {
                    toast.success("Created", "Journal entry created.");
                    nav.push("/accounting/journal-entries");
                }
                Err(e) => {
                    toast.error("Error", &e);
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "jec-page",
            div { class: "jec-header",
                Button { variant: ButtonVariant::Ghost, onclick: move |_| { navigator.push("/accounting/journal-entries"); }, "← Back" }
                h1 { "New Journal Entry" }
            }

            div { class: "jec-form",
                div { class: "jec-row",
                    div { class: "jec-field",
                        label { "Entry Date" }
                        input { r#type: "date", value: "{entry_date}", onchange: move |e| entry_date.set(e.value()) }
                    }
                    div { class: "jec-field",
                        label { "Reference Type (optional)" }
                        select {
                            value: "{reference_type}",
                            onchange: move |e| reference_type.set(e.value()),
                            option { value: "", "None" }
                            option { value: "invoice", "Invoice" }
                            option { value: "payment", "Payment" }
                            option { value: "purchase_order", "Purchase Order" }
                            option { value: "expense", "Expense" }
                            option { value: "salary", "Salary" }
                        }
                    }
                    div { class: "jec-field",
                        label { "Reference ID (optional)" }
                        input { r#type: "number", value: "{reference_id}", onchange: move |e| reference_id.set(e.value()) }
                    }
                }

                div { class: "jec-lines-header",
                    h2 { "Journal Lines" }
                    Button { variant: ButtonVariant::Secondary, onclick: add_line, "＋ Add Line" }
                }

                table { class: "jec-lines-table",
                    thead { tr {
                        th { style: "width: 30%;", "Account" }
                        th { style: "width: 15%;", "Debit" }
                        th { style: "width: 15%;", "Credit" }
                        th { style: "width: 30%;", "Description" }
                        th { style: "width: 10%;" }
                    }}
                    tbody {
                        for (idx, line) in lines.read().iter().enumerate() {{
                            let idx = idx;
                            rsx! { tr {
                                td {
                                    select {
                                        value: "{line.account_id}",
                                        onchange: { let mut lines = lines.clone(); move |e| {
                                            if let Ok(v) = e.value().parse::<i64>() {
                                                lines.write()[idx].account_id = v;
                                            }
                                        }},
                                        option { value: "0", "Select account..." }
                                        for acct in accounts.iter() {
                                            option { value: "{acct.id}", "{acct.code} - {acct.name}" }
                                        }
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        min: "0",
                                        placeholder: "0.00",
                                        value: "{line.debit}",
                                        onchange: { let mut lines = lines.clone(); move |e| { lines.write()[idx].debit = e.value(); }},
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        min: "0",
                                        placeholder: "0.00",
                                        value: "{line.credit}",
                                        onchange: { let mut lines = lines.clone(); move |e| { lines.write()[idx].credit = e.value(); }},
                                    }
                                }
                                td {
                                    input {
                                        r#type: "text",
                                        placeholder: "Description",
                                        value: "{line.description}",
                                        onchange: { let mut lines = lines.clone(); move |e| { lines.write()[idx].description = e.value(); }},
                                    }
                                }
                                td {
                                    if lines.read().len() > 2 {
                                        button { class: "remove-btn", r#type: "button", onclick: { let mut lines = lines.clone(); move |_| remove_line(idx) }, "✕" }
                                    }
                                }
                            }}
                        }}
                    }
                }

                div { class: "je-totals",
                    div { class: "je-totals-item",
                        span { class: "je-totals-label", "Total Debit:" }
                        span { class: "je-totals-value", "PKR {total_debit:.2}" }
                    }
                    div { class: "je-totals-item",
                        span { class: "je-totals-label", "Total Credit:" }
                        span { class: "je-totals-value", "PKR {total_credit:.2}" }
                    }
                    div { class: "je-totals-item",
                        span { class: "je-totals-label", "Status:" }
                        if is_balanced {
                            span { class: "je-totals-value je-totals-balanced", "✓ Balanced" }
                        } else {
                            span { class: "je-totals-value je-totals-unbalanced", "✗ Unbalanced" }
                        }
                    }
                }

                div { class: "jec-actions",
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| { navigator.push("/accounting/journal-entries"); }, "Cancel" }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: *is_submitting.read() || !is_balanced,
                        onclick: submit,
                        if *is_submitting.read() { "Creating..." } else { "Create Entry" }
                    }
                }
            }
        }
    }
}
