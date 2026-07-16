use dioxus::prelude::*;

const OVERVIEW_CSS: &str = r#"
.customer-overview { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 20px; }
.customer-metric-card { background: var(--surface); border: 1px solid var(--border-color); border-radius: 8px; padding: 16px; display: flex; flex-direction: column; gap: 6px; }
.customer-metric-label { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); }
.customer-metric-value { font-size: 20px; font-weight: 700; color: var(--text-primary); }
.customer-metric-value.positive { color: #28a745; }
.customer-metric-value.negative { color: #dc3545; }
.customer-metric-sub { font-size: 12px; color: var(--text-secondary); }
.customer-recent-activity { background: var(--surface); border: 1px solid var(--border-color); border-radius: 8px; padding: 16px; }
.customer-recent-activity h3 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0 0 12px; }
.activity-item { display: flex; align-items: center; gap: 10px; padding: 8px 0; border-bottom: 1px solid var(--border-color); font-size: 13px; }
.activity-item:last-child { border-bottom: none; }
.activity-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.activity-dot.invoice { background: #4a90d9; }
.activity-dot.payment { background: #28a745; }
.activity-text { flex: 1; color: var(--text-primary); }
.activity-amount { font-family: monospace; font-weight: 500; }
.activity-date { color: var(--text-secondary); font-size: 12px; }
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomerOverviewData {
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub outstanding: f64,
    pub invoice_count: i64,
    pub overdue_count: i64,
    pub credit_utilization: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivityItem {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub activity_type: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct CustomerOverviewProps {
    pub data: CustomerOverviewData,
    pub recent_activity: Vec<ActivityItem>,
}

#[component]
pub fn CustomerOverview(props: CustomerOverviewProps) -> Element {
    rsx! {
        style { "{OVERVIEW_CSS}" }
        div { class: "customer-overview",
            div { class: "customer-metric-card",
                span { class: "customer-metric-label", "Total Invoiced" }
                span { class: "customer-metric-value", {format!("Rs. {:.0}", props.data.total_invoiced)} }
                span { class: "customer-metric-sub", "{props.data.invoice_count} invoices" }
            }
            div { class: "customer-metric-card",
                span { class: "customer-metric-label", "Total Paid" }
                span { class: "customer-metric-value positive", {format!("Rs. {:.0}", props.data.total_paid)} }
            }
            div { class: "customer-metric-card",
                span { class: "customer-metric-label", "Outstanding" }
                span { class: if props.data.outstanding > 0.0 { "customer-metric-value negative" } else { "customer-metric-value positive" },
                    {format!("Rs. {:.0}", props.data.outstanding)}
                }
            }
            div { class: "customer-metric-card",
                span { class: "customer-metric-label", "Overdue" }
                span { class: if props.data.overdue_count > 0 { "customer-metric-value negative" } else { "customer-metric-value" },
                    "{props.data.overdue_count}"
                }
                span { class: "customer-metric-sub", "invoices" }
            }
            div { class: "customer-metric-card",
                span { class: "customer-metric-label", "Credit Utilization" }
                span { class: "customer-metric-value", {format!("{:.1}%", props.data.credit_utilization)} }
            }
        }
        div { class: "customer-recent-activity",
            h3 { "Recent Activity" }
            if props.recent_activity.is_empty() {
                p { style: "color: var(--text-secondary); font-size: 13px;", "No recent activity." }
            }
            for item in props.recent_activity.iter() {
                div { class: "activity-item",
                    div { class: "activity-dot {item.activity_type}" }
                    span { class: "activity-text", "{item.description}" }
                    span { class: "activity-amount",
                        {format!("Rs. {:.2}", item.amount)}
                    }
                    span { class: "activity-date", "{item.date}" }
                }
            }
        }
    }
}
