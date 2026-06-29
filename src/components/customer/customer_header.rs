use dioxus::prelude::*;

const CUSTOMER_HEADER_CSS: &str = r#"
.customer-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 20px; gap: 16px; flex-wrap: wrap; }
.customer-header-info { display: flex; flex-direction: column; gap: 4px; }
.customer-header-title { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.customer-header-title h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.customer-code { font-size: 13px; font-family: monospace; color: var(--text-secondary); padding: 2px 8px; background: var(--bg-muted); border-radius: 4px; }
.customer-header-actions { display: flex; gap: 8px; flex-wrap: wrap; }
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomerHeaderData {
    pub customer_name: String,
    pub customer_code: String,
    pub current_balance: f64,
    pub credit_limit: f64,
    pub is_active: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct CustomerHeaderProps {
    pub data: CustomerHeaderData,
    pub on_edit: Option<EventHandler<MouseEvent>>,
    pub on_deactivate: Option<EventHandler<MouseEvent>>,
    pub on_new_invoice: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn CustomerHeader(props: CustomerHeaderProps) -> Element {
    rsx! {
        style { "{CUSTOMER_HEADER_CSS}" }
        div { class: "customer-header",
            div { class: "customer-header-info",
                div { class: "customer-header-title",
                    h1 { "{props.data.customer_name}" }
                    span { class: "customer-code", "{props.data.customer_code}" }
                    if !props.data.is_active {
                        span { class: "compact-card-badge badge-red", "Inactive" }
                    }
                }
                div { style: "display: flex; gap: 20px; font-size: 13px; color: var(--text-secondary); margin-top: 4px;",
                    span { {format!("Balance: Rs. {:.2}", props.data.current_balance)} }
                    span { {format!("Credit Limit: Rs. {:.2}", props.data.credit_limit)} }
                }
            }
            div { class: "customer-header-actions",
                for cb in props.on_new_invoice.iter() {
                    button {
                        class: "btn btn-primary",
                        onclick: {
                            let cb = cb.clone();
                            move |e| cb.call(e)
                        },
                        "＋ New Invoice"
                    }
                }
                for cb in props.on_edit.iter() {
                    button {
                        class: "btn btn-secondary",
                        onclick: {
                            let cb = cb.clone();
                            move |e| cb.call(e)
                        },
                        "✏️ Edit"
                    }
                }
                for cb in props.on_deactivate.iter() {
                    button {
                        class: "btn btn-danger",
                        onclick: {
                            let cb = cb.clone();
                            move |e| cb.call(e)
                        },
                        "Deactivate"
                    }
                }
            }
        }
    }
}
