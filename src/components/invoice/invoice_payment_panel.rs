use dioxus::prelude::*;

const PAYMENT_PANEL_CSS: &str = r#"
.payment-panel { background: var(--surface); border: 1px solid var(--border-color); border-radius: 8px; padding: 16px; margin-bottom: 16px; }
.payment-panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
.payment-panel-header h3 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }
.payment-toggle { display: flex; align-items: center; gap: 8px; cursor: pointer; }
.payment-toggle-track { width: 36px; height: 20px; border-radius: 10px; background: #ccc; position: relative; transition: background 0.2s; }
.payment-toggle-track.active { background: var(--accent); }
.payment-toggle-thumb { width: 16px; height: 16px; border-radius: 50%; background: var(--surface); position: absolute; top: 2px; left: 2px; transition: transform 0.2s; }
.payment-toggle-track.active .payment-toggle-thumb { transform: translateX(16px); }
.payment-toggle-label { font-size: 13px; color: var(--text-primary); }
.payment-form { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.payment-field { display: flex; flex-direction: column; gap: 4px; }
.payment-field.full-width { grid-column: 1 / -1; }
.payment-field label { font-size: 12px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.payment-field input, .payment-field select { padding: 8px 10px; border: 1px solid var(--border-color); border-radius: 6px; font-size: 13px; }
.payment-field input:focus, .payment-field select:focus { outline: none; border-color: var(--accent); }
.payment-summary { display: flex; justify-content: space-between; padding: 12px; background: var(--surface-secondary); border-radius: 6px; margin-top: 12px; font-size: 13px; }
.payment-summary-amount { font-weight: 600; }
.payment-summary-change { color: #28a745; }
.payment-summary-short { color: #dc3545; }
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentInfo {
    pub record_payment: bool,
    pub amount: f64,
    pub method: String,
    pub reference: String,
    pub notes: String,
}

impl Default for PaymentInfo {
    fn default() -> Self {
        Self {
            record_payment: false,
            amount: 0.0,
            method: "Cash".to_string(),
            reference: String::new(),
            notes: String::new(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InvoicePaymentPanelProps {
    pub total: f64,
    pub payment: PaymentInfo,
    pub on_change: EventHandler<PaymentInfo>,
}

#[component]
pub fn InvoicePaymentPanel(props: InvoicePaymentPanelProps) -> Element {
    let change = props.payment.amount - props.total;
    let is_overpaid = change > 0.01;
    let is_underpaid = props.payment.amount > 0.0 && props.payment.amount < props.total - 0.01;

    rsx! {
        style { "{PAYMENT_PANEL_CSS}" }
        div { class: "payment-panel",
            div { class: "payment-panel-header",
                h3 { "Payment" }
                div {
                    class: "payment-toggle",
                    onclick: {
                        let on_change = props.on_change;
                        let base = props.payment.clone();
                        let total = props.total;
                        move |_| {
                            let mut p = base.clone();
                            p.record_payment = !p.record_payment;
                            if p.record_payment && p.amount == 0.0 {
                                p.amount = total;
                            }
                            on_change.call(p);
                        }
                    },
                    div { class: if props.payment.record_payment { "payment-toggle-track active" } else { "payment-toggle-track" },
                        div { class: "payment-toggle-thumb" }
                    }
                    span { class: "payment-toggle-label",
                        if props.payment.record_payment { "Record Payment" } else { "Skip Payment" }
                    }
                }
            }
            if props.payment.record_payment {
                div { class: "payment-form",
                    div { class: "payment-field",
                        label { "Amount" }
                        input {
                            r#type: "number",
                            value: "{props.payment.amount}",
                            min: "0",
                            step: "0.01",
                            oninput: {
                                let on_change = props.on_change;
                                let base = props.payment.clone();
                                move |e| {
                                    let mut p = base.clone();
                                    p.amount = e.value().parse().unwrap_or(0.0);
                                    on_change.call(p);
                                }
                            },
                        }
                    }
                    div { class: "payment-field",
                        label { "Method" }
                        select {
                            value: "{props.payment.method}",
                            onchange: {
                                let on_change = props.on_change;
                                let base = props.payment.clone();
                                move |e| {
                                    let mut p = base.clone();
                                    p.method = e.value();
                                    on_change.call(p);
                                }
                            },
                            option { value: "Cash", "Cash" }
                            option { value: "Bank Transfer", "Bank Transfer" }
                            option { value: "Cheque", "Cheque" }
                            option { value: "Credit Card", "Credit Card" }
                            option { value: "Online", "Online Payment" }
                        }
                    }
                    div { class: "payment-field",
                        label { "Reference" }
                        input {
                            r#type: "text",
                            value: "{props.payment.reference}",
                            placeholder: "Cheque #, transaction ID…",
                            oninput: {
                                let on_change = props.on_change;
                                let base = props.payment.clone();
                                move |e| {
                                    let mut p = base.clone();
                                    p.reference = e.value();
                                    on_change.call(p);
                                }
                            },
                        }
                    }
                    div { class: "payment-field",
                        label { "Notes" }
                        input {
                            r#type: "text",
                            value: "{props.payment.notes}",
                            placeholder: "Optional notes",
                            oninput: {
                                let on_change = props.on_change;
                                let base = props.payment.clone();
                                move |e| {
                                    let mut p = base.clone();
                                    p.notes = e.value();
                                    on_change.call(p);
                                }
                            },
                        }
                    }
                }
                div { class: "payment-summary",
                    span { {format!("Total: Rs. {:.2}", props.total)} }
                    span { {format!("Paid: Rs. {:.2}", props.payment.amount)} }
                    if is_overpaid {
                        span { class: "payment-summary-change",
                            {format!("Change: Rs. {:.2}", change)}
                        }
                    } else if is_underpaid {
                        span { class: "payment-summary-short",
                            {format!("Remaining: Rs. {:.2}", props.total - props.payment.amount)}
                        }
                    }
                }
            }
        }
    }
}
