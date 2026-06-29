use dioxus::prelude::*;

const LEDGER_CSS: &str = r#"
.ledger-container { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; overflow: hidden; }
.ledger-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 16px; border-bottom: 1px solid var(--border-color); }
.ledger-header h3 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }
.ledger-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.ledger-table th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.ledger-table th.text-right { text-align: right; }
.ledger-table td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); }
.ledger-table td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.ledger-table tr:last-child td { border-bottom: none; }
.ledger-table .balance-positive { color: #dc3545; font-weight: 600; }
.ledger-table .balance-zero { color: #28a745; }
.ledger-table .type-badge { display: inline-block; padding: 2px 6px; border-radius: 4px; font-size: 11px; font-weight: 600; }
.ledger-table .type-invoice { background: rgba(74,144,217,0.1); color: #4a90d9; }
.ledger-table .type-payment { background: rgba(40,167,69,0.1); color: #28a745; }
.ledger-table .type-opening { background: rgba(108,117,125,0.1); color: #6c757d; }
.ledger-summary { display: flex; gap: 24px; padding: 12px 16px; background: var(--bg-muted, #f8f9fa); border-top: 2px solid var(--border-color); font-size: 13px; }
.ledger-summary-item { display: flex; gap: 6px; }
.ledger-summary-label { color: var(--text-secondary); }
.ledger-summary-value { font-weight: 600; }
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct LedgerEntry {
    pub date: String,
    pub reference: String,
    pub description: String,
    pub transaction_type: String,
    pub debit: f64,
    pub credit: f64,
    pub balance: f64,
}

#[derive(Props, Clone, PartialEq)]
pub struct CustomerLedgerProps {
    pub entries: Vec<LedgerEntry>,
}

#[component]
pub fn CustomerLedger(props: CustomerLedgerProps) -> Element {
    let total_debit: f64 = props.entries.iter().map(|e| e.debit).sum();
    let total_credit: f64 = props.entries.iter().map(|e| e.credit).sum();
    let final_balance = props.entries.last().map(|e| e.balance).unwrap_or(0.0);

    rsx! {
        style { "{LEDGER_CSS}" }
        div { class: "ledger-container",
            div { class: "ledger-header",
                h3 { "Customer Ledger" }
            }
            table { class: "ledger-table",
                thead { tr {
                    th { "Date" }
                    th { "Reference" }
                    th { "Description" }
                    th { "Type" }
                    th { class: "text-right", "Debit" }
                    th { class: "text-right", "Credit" }
                    th { class: "text-right", "Balance" }
                }}
                tbody {
                    if props.entries.is_empty() {
                        tr { td { colspan: "7", style: "text-align: center; padding: 24px; color: var(--text-secondary);", "No ledger entries." } }
                    }
                    for entry in props.entries.iter() {
                        tr {
                            td { "{entry.date}" }
                            td { style: "font-family: monospace; font-size: 12px;", "{entry.reference}" }
                            td { "{entry.description}" }
                            td {
                                span { class: "type-badge type-{entry.transaction_type}", "{entry.transaction_type}" }
                            }
                            td { class: "text-right",
                                if entry.debit > 0.0 { {format!("Rs. {:.2}", entry.debit)} }
                            }
                            td { class: "text-right",
                                if entry.credit > 0.0 { {format!("Rs. {:.2}", entry.credit)} }
                            }
                            td { class: if entry.balance > 0.0 { "text-right balance-positive" } else { "text-right balance-zero" },
                                {format!("Rs. {:.2}", entry.balance)}
                            }
                        }
                    }
                }
            }
            div { class: "ledger-summary",
                div { class: "ledger-summary-item",
                    span { class: "ledger-summary-label", "Total Debit:" }
                    span { class: "ledger-summary-value", {format!("Rs. {:.2}", total_debit)} }
                }
                div { class: "ledger-summary-item",
                    span { class: "ledger-summary-label", "Total Credit:" }
                    span { class: "ledger-summary-value", {format!("Rs. {:.2}", total_credit)} }
                }
                div { class: "ledger-summary-item",
                    span { class: "ledger-summary-label", "Balance:" }
                    span { class: "ledger-summary-value",
                        {format!("Rs. {:.2}", final_balance)}
                    }
                }
            }
        }
    }
}
