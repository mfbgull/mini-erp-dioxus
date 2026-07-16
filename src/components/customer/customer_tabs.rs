use dioxus::prelude::*;

const TAB_CSS: &str = r#"
.customer-tabs { display: flex; gap: 0; border-bottom: 2px solid var(--border-color); margin-bottom: 16px; }
.customer-tab { padding: 10px 20px; font-size: 13px; font-weight: 500; color: var(--text-secondary); cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -2px; background: none; border-top: none; border-left: none; border-right: none; }
.customer-tab:hover { color: var(--text-primary); }
.customer-tab.active { color: var(--accent); border-bottom-color: var(--accent); font-weight: 600; }
"#;

#[derive(Clone, Copy, PartialEq)]
pub enum CustomerTab {
    Overview,
    Invoices,
    Payments,
    Ledger,
}

impl CustomerTab {
    pub fn label(&self) -> &'static str {
        match self {
            CustomerTab::Overview => "Overview",
            CustomerTab::Invoices => "Invoices",
            CustomerTab::Payments => "Payments",
            CustomerTab::Ledger => "Ledger",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CustomerTabsProps {
    pub active: CustomerTab,
    pub on_change: EventHandler<CustomerTab>,
}

#[component]
pub fn CustomerTabs(props: CustomerTabsProps) -> Element {
    rsx! {
        style { "{TAB_CSS}" }
        div { class: "customer-tabs",
            for tab in [CustomerTab::Overview, CustomerTab::Invoices, CustomerTab::Payments, CustomerTab::Ledger] {
                button {
                    class: if props.active == tab { "customer-tab active" } else { "customer-tab" },
                    onclick: {
                        let on_change = props.on_change;
                        move |_| on_change.call(tab)
                    },
                    "{tab.label()}"
                }
            }
        }
    }
}
