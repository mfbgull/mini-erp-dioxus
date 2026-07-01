//! Dashboard page — KPI overview, sales chart, recent activity, and alerts.
//!
//! Uses StaticCard components for KPIs, inline SVG for the sales chart bar
//! visualization, and styled sections for activity feed and low-stock alerts.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, StatCard, StatCardVariant, StatTrend, TrendDirection,
};
use dioxus::prelude::*;

// ============================================================================
// Dashboard CSS
// ============================================================================

pub const DASHBOARD_CSS: &str = r##"
/* ── Dashboard Layout ── */
.dashboard {
    padding: 0;
    max-width: 1200px;
    margin: 0 auto;
}

.dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    flex-wrap: wrap;
    gap: 12px;
}

.dashboard-header h1 {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary, #1a1a2e);
    margin: 0;
}

.dashboard-header p {
    font-size: 14px;
    color: var(--text-secondary, #6c757d);
    margin: 2px 0 0 0;
}

.dashboard-date {
    font-size: 13px;
    color: var(--text-secondary, #6c757d);
    background: #f8f9fa;
    padding: 6px 14px;
    border-radius: 6px;
    border: 1px solid var(--border-color, #e0e0e0);
}

/* ── KPI Grid ── */
.dashboard-kpi-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
}

/* ── Two-column section ── */
.dashboard-columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-bottom: 24px;
}

@media (max-width: 900px) {
    .dashboard-columns {
        grid-template-columns: 1fr;
    }
}

/* ── Section Cards ── */
.dashboard-section {
    background: #ffffff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    box-shadow: var(--shadow, 0 1px 3px rgba(0,0,0,0.08));
    overflow: hidden;
}

.dashboard-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.dashboard-section-header h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary, #1a1a2e);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
}

.dashboard-section-header a {
    font-size: 12px;
    color: var(--accent, #4a90d9);
    text-decoration: none;
    font-weight: 500;
}

.dashboard-section-header a:hover {
    text-decoration: underline;
}

.dashboard-section-body {
    padding: 16px 20px;
}

/* ── SVG Bar Chart ── */
.dashboard-chart {
    width: 100%;
    height: 180px;
}

.dashboard-chart svg {
    width: 100%;
    height: 100%;
}

.dashboard-chart-bar {
    fill: var(--accent, #4a90d9);
    transition: fill 0.15s ease;
    cursor: pointer;
}

.dashboard-chart-bar:hover {
    fill: #357abd;
}

.dashboard-chart-bar-label {
    font-size: 10px;
    fill: var(--text-secondary, #6c757d);
    text-anchor: middle;
}

.dashboard-chart-bar-value {
    font-size: 9px;
    fill: var(--text-secondary, #6c757d);
    text-anchor: middle;
    font-weight: 600;
}

/* ── Activity Feed ── */
.dashboard-activity {
    margin-bottom: 24px;
}

.dashboard-activity-item {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid #f0f0f0;
}

.dashboard-activity-item:last-child {
    border-bottom: none;
}

.dashboard-activity-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    flex-shrink: 0;
    background: #f0f2f5;
}

.dashboard-activity-content {
    flex: 1;
    min-width: 0;
}

.dashboard-activity-text {
    font-size: 13px;
    color: var(--text-primary, #1a1a2e);
    line-height: 1.4;
    margin: 0;
}

.dashboard-activity-text strong {
    font-weight: 600;
}

.dashboard-activity-time {
    font-size: 11px;
    color: var(--text-secondary, #adb5bd);
    margin-top: 2px;
}

/* ── Low Stock Alert List ── */
.dashboard-alert-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid #f0f0f0;
}

.dashboard-alert-item:last-child {
    border-bottom: none;
}

.dashboard-alert-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
}

.dashboard-alert-dot-critical { background: var(--danger, #dc3545); }
.dashboard-alert-dot-warning { background: var(--warning, #ffc107); }
.dashboard-alert-dot-info { background: var(--accent, #4a90d9); }

.dashboard-alert-info {
    flex: 1;
    min-width: 0;
}

.dashboard-alert-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary, #1a1a2e);
    margin: 0;
}

.dashboard-alert-detail {
    font-size: 12px;
    color: var(--text-secondary, #6c757d);
    margin-top: 1px;
}

.dashboard-alert-stock {
    font-size: 13px;
    font-weight: 600;
    color: var(--danger, #dc3545);
    white-space: nowrap;
}

/* ── Loading State ── */
.dashboard-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    color: var(--text-secondary, #6c757d);
    font-size: 14px;
}

/* ── Quick Actions ── */
.dashboard-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}
"##;

// ============================================================================
// View Types
// ============================================================================

#[derive(Clone, PartialEq)]
struct KpiData {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

#[derive(Clone, PartialEq)]
struct MonthlySales {
    month: String,
    amount: f64,
}

#[derive(Clone, PartialEq)]
struct ActivityItem {
    icon: String,
    icon_bg: String,
    text: String,
    time: String,
}

#[derive(Clone, PartialEq)]
struct LowStockItem {
    name: String,
    code: String,
    current_stock: f64,
    reorder_level: f64,
    unit: String,
    severity: Severity,
}

#[derive(Clone, PartialEq)]
enum Severity {
    Critical,
    Warning,
    Info,
}

// ============================================================================
// Formatting Helpers
// ============================================================================

fn format_pkr(amount: f64) -> String {
    if amount >= 1_000_000.0 {
        let m = amount / 1_000_000.0;
        if (m - m.round()).abs() < 0.01 {
            format!("PKR {:.0}M", m)
        } else {
            format!("PKR {:.1}M", m)
        }
    } else if amount >= 1_000.0 {
        let thousands = amount / 1_000.0;
        format!("PKR {:.0}K", thousands)
    } else {
        format!("PKR {:.0}", amount)
    }
}

fn activity_icon_for(action: &str) -> (&'static str, &'static str) {
    match action.to_lowercase().as_str() {
        a if a.contains("create") => ("🧾", "#e8f0fe"),
        a if a.contains("invoice") => ("🧾", "#e8f0fe"),
        a if a.contains("payment") || a.contains("receive") => ("💰", "#f0f0ff"),
        a if a.contains("purchase") || a.contains("receipt") => ("📥", "#e8f8f0"),
        a if a.contains("customer") => ("👤", "#fff8e8"),
        a if a.contains("production") || a.contains("manufacture") => ("⚙", "#f8f0ff"),
        a if a.contains("login") => ("🔑", "#e8f8f0"),
        _ => ("📋", "#f0f2f5"),
    }
}

fn relative_time(iso_str: &str) -> String {
    if iso_str.is_empty() {
        return "Unknown".to_string();
    }
    iso_str.split('T').next().unwrap_or(iso_str).to_string()
}

// ============================================================================
// Helpers
// ============================================================================

fn severity_class(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical => "dashboard-alert-dot-critical",
        Severity::Warning => "dashboard-alert-dot-warning",
        Severity::Info => "dashboard-alert-dot-info",
    }
}

fn severity_label(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical => "Out of Stock",
        Severity::Warning => "Critically Low",
        Severity::Info => "Below Reorder",
    }
}

fn today_formatted() -> String {
    "June 26, 2026".to_string()
}

// ============================================================================
// SVG Bar Chart Component
// ============================================================================

#[derive(Clone, PartialEq, Props)]
pub struct SalesChartProps {
    data: Vec<MonthlySales>,
}

#[component]
fn SalesChart(props: SalesChartProps) -> Element {
    let max_amount = props.data.iter().map(|s| s.amount).fold(0.0_f64, f64::max);
    let bar_count = props.data.len();
    let bar_width_pct = 100.0 / bar_count as f64;
    let bar_gap_pct = bar_width_pct * 0.25;
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 160.0;

    rsx! {
        div { class: "dashboard-chart",
            svg {
                view_box: "0 0 100 180",
                preserve_aspect_ratio: "xMidYMid meet",
                line { x1: "0", y1: "10", x2: "100", y2: "10", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "50", x2: "100", y2: "50", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "90", x2: "100", y2: "90", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "130", x2: "100", y2: "130", stroke: "#f0f0f0", stroke_width: "0.5" }

                {props.data.into_iter().enumerate().map(|(i, s)| {
                    let bar_height = if max_amount > 0.0 {
                        (s.amount / max_amount) * chart_height
                    } else {
                        0.0
                    };
                    let x = i as f64 * bar_width_pct + bar_gap_pct / 2.0;
                    let y = 170.0 - bar_height;
                    let label_y = 178.0;

                    rsx! {
                        rect {
                            key: "{i}",
                            class: "dashboard-chart-bar",
                            x: "{x:.1}",
                            y: "{y:.1}",
                            width: "{bar_inner_pct:.1}",
                            height: "{bar_height:.1}",
                            rx: "2",
                        }
                        text {
                            class: "dashboard-chart-bar-value",
                            x: "{x + bar_inner_pct / 2.0:.1}",
                            y: "{y - 4.0:.1}",
                            "{s.amount:.0}"
                        }
                        text {
                            class: "dashboard-chart-bar-label",
                            x: "{x + bar_inner_pct / 2.0:.1}",
                            y: "{label_y:.1}",
                            "{s.month}"
                        }
                    }
                })}
            }
        }
    }
}

// ============================================================================
// Dashboard Page Component
// ============================================================================

#[component]
pub fn DashboardPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let summary_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_dashboard_summary().await.unwrap_or_default()
        }
    });

    let sales_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_sales_summary().await.unwrap_or_default()
        }
    });

    let ar_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_ar_summary().await.unwrap_or_default()
        }
    });

    let activity_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_activity_logs().await.unwrap_or_default()
        }
    });

    let items_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_items().await.unwrap_or_else(|_| vec![])
        }
    });

    let any_loading = summary_resource.read().is_none()
        || sales_resource.read().is_none()
        || activity_resource.read().is_none();

    let summary = summary_resource.read().clone().unwrap_or_default();
    let sales_data = sales_resource.read().clone().unwrap_or_default();
    let ar_data = ar_resource.read().clone().unwrap_or_default();
    let activity_logs = activity_resource.read().clone().unwrap_or_default();
    let items = items_resource.read().clone().unwrap_or_default();

    let total_revenue = summary.get("total_revenue").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let _total_expenses = summary.get("total_expenses").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_invoices = summary.get("total_invoices").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_customers = summary.get("total_customers").and_then(|v| v.as_i64()).unwrap_or(0);
    let low_stock_count = summary.get("low_stock_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let outstanding_ar = ar_data.get("current").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let unpaid_invoices = summary.get("total_invoices").and_then(|v| v.as_i64()).unwrap_or(0);
    let today_sales = sales_data.get("today").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let this_month_sales = sales_data.get("this_month").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let kpis = vec![
        KpiData {
            title: "Total Revenue".to_string(),
            value: format_pkr(total_revenue),
            icon: "💰".to_string(),
            variant: StatCardVariant::Primary,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: format!("PKR {:.0} today", today_sales),
            }),
            footer: Some("All time".to_string()),
        },
        KpiData {
            title: "Invoices".to_string(),
            value: total_invoices.to_string(),
            icon: "🧾".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some(format!("{} unpaid", unpaid_invoices)),
        },
        KpiData {
            title: "Active Customers".to_string(),
            value: total_customers.to_string(),
            icon: "👥".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: Some("Currently active".to_string()),
        },
        KpiData {
            title: "Low Stock Items".to_string(),
            value: low_stock_count.to_string(),
            icon: "⚠".to_string(),
            variant: StatCardVariant::Danger,
            trend: None,
            footer: Some(format!("Receivables: {}", format_pkr(outstanding_ar))),
        },
    ];

    let sales_chart: Vec<MonthlySales> = vec![
        MonthlySales { month: "Today".to_string(), amount: today_sales },
        MonthlySales { month: "Week".to_string(), amount: sales_data.get("this_week").and_then(|v| v.as_f64()).unwrap_or(0.0) },
        MonthlySales { month: "Month".to_string(), amount: this_month_sales },
    ];

    let activity: Vec<ActivityItem> = activity_logs.into_iter().take(5).map(|log| {
        let (icon, bg) = activity_icon_for(&log.action);
        let username = log.username.unwrap_or_else(|| "System".to_string());
        let entity = log.entity_type;
        let meta = log.metadata.unwrap_or_default();
        let time = relative_time(&log.created_at);
        let text = if meta.is_empty() {
            format!("<strong>{}</strong> {} <strong>{}</strong>", username, log.action, entity)
        } else {
            format!("<strong>{}</strong> {} <strong>{}</strong> — {}", username, log.action, entity, meta)
        };
        ActivityItem {
            icon: icon.to_string(),
            icon_bg: bg.to_string(),
            text,
            time,
        }
    }).collect();

    let alerts: Vec<LowStockItem> = items.into_iter().filter_map(|item| {
        if item.current_stock <= item.reorder_level && item.is_active {
            let severity = if item.current_stock == 0.0 {
                Severity::Critical
            } else if item.current_stock <= item.reorder_level * 0.25 {
                Severity::Critical
            } else if item.current_stock <= item.reorder_level {
                Severity::Warning
            } else {
                return None;
            };
            Some(LowStockItem {
                name: item.item_name,
                code: item.item_code,
                current_stock: item.current_stock,
                reorder_level: item.reorder_level,
                unit: item.unit_of_measure,
                severity,
            })
        } else {
            None
        }
    }).collect();

    if any_loading {
        rsx! {
            div { class: "dashboard",
                div { class: "dashboard-header",
                    div {
                        h1 { "Dashboard" }
                        p { "Overview of your business performance" }
                    }
                    div { class: "dashboard-date", "{today_formatted()}" }
                }
                div { class: "dashboard-loading", "Loading dashboard data..." }
            }
        }
    } else {
        rsx! {
            div { class: "dashboard",

                div { class: "dashboard-header",
                    div {
                        h1 { "Dashboard" }
                        p { "Overview of your business performance" }
                    }
                    div { class: "dashboard-date", "{today_formatted()}" }
                }

                div { class: "dashboard-kpi-grid",
                    {kpis.into_iter().map(|kpi| {
                        rsx! {
                            StatCard {
                                key: "{kpi.title}",
                                title: kpi.title,
                                value: kpi.value,
                                icon: kpi.icon,
                                variant: kpi.variant,
                                trend: kpi.trend,
                                footer: kpi.footer,
                            }
                        }
                    })}
                }

                div { class: "dashboard-columns",

                    div { class: "dashboard-section",
                        div { class: "dashboard-section-header",
                            h2 { "📈 Sales Summary" }
                            a { href: "/reports/sales", "View Report →" }
                        }
                        div { class: "dashboard-section-body",
                            SalesChart { data: sales_chart.clone() }
                        }
                    }

                    div { class: "dashboard-section",
                        div { class: "dashboard-section-header",
                            h2 { "⚠ Low Stock Alerts" }
                            a { href: "/inventory/items", "Manage Inventory →" }
                        }
                        div { class: "dashboard-section-body",
                            if alerts.is_empty() {
                                div { class: "dashboard-loading", "No low stock alerts" }
                            } else {
                                {alerts.into_iter().map(|item| {
                                    let dot_class = severity_class(&item.severity);
                                    let label = severity_label(&item.severity);
                                    let stock_text = if item.current_stock == 0.0 {
                                        "OUT OF STOCK".to_string()
                                    } else {
                                        format!("{} {}", item.current_stock, item.unit)
                                    };

                                    rsx! {
                                        div { key: "{item.code}", class: "dashboard-alert-item",
                                            div { class: "dashboard-alert-dot {dot_class}" }
                                            div { class: "dashboard-alert-info",
                                                p { class: "dashboard-alert-name", "{item.name}" }
                                                p { class: "dashboard-alert-detail",
                                                    "{item.code}  •  Reorder at {item.reorder_level} {item.unit}  •  {label}"
                                                }
                                            }
                                            div { class: "dashboard-alert-stock", "{stock_text}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }

                div { class: "dashboard-section dashboard-activity",
                    div { class: "dashboard-section-header",
                        h2 { "🕐 Recent Activity" }
                        a { href: "/activity-log", "View All →" }
                    }
                    div { class: "dashboard-section-body",
                        if activity.is_empty() {
                            div { class: "dashboard-loading", "No recent activity" }
                        } else {
                            {activity.into_iter().map(|a| {
                                rsx! {
                                    div { key: "{a.time}-{a.text}", class: "dashboard-activity-item",
                                        div {
                                            class: "dashboard-activity-icon",
                                            style: "background: {a.icon_bg};",
                                            "{a.icon}"
                                        }
                                        div { class: "dashboard-activity-content",
                                            p { class: "dashboard-activity-text",
                                                dangerous_inner_html: "{a.text}",
                                            }
                                            p { class: "dashboard-activity-time", "{a.time}" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }

                div { class: "dashboard-actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        icon: Some("🧾".to_string()),
                        onclick: move |_| { navigator.push("/sales/invoices/new"); },
                        "New Invoice"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        icon: Some("📦".to_string()),
                        onclick: move |_| { navigator.push("/inventory/items/new"); },
                        "New Item"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        icon: Some("👤".to_string()),
                        onclick: move |_| { navigator.push("/customers/new"); },
                        "New Customer"
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        icon: Some("📊".to_string()),
                        onclick: move |_| { navigator.push("/reports"); },
                        "View Reports"
                    }
                }
            }
        }
    }
}
