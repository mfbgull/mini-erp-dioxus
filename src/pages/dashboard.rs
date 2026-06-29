//! Dashboard page — KPI overview, sales chart, recent activity, and alerts.
//!
//! Uses StaticCard components for KPIs, inline SVG for the sales chart bar
//! visualization, and styled sections for activity feed and low-stock alerts.

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

/* ── Quick Actions ── */
.dashboard-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}
"##;

// ============================================================================
// Demo Data Types
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
    current_stock: i32,
    reorder_level: i32,
    unit: String,
    severity: Severity,
}

#[derive(Clone, PartialEq)]
enum Severity {
    Critical, // stock = 0
    Warning,  // stock <= 25% of reorder
    Info,     // stock <= reorder level
}

// ============================================================================
// Demo Data
// ============================================================================

fn kpi_data() -> Vec<KpiData> {
    vec![
        KpiData {
            title: "Total Revenue".to_string(),
            value: "PKR 1,284,500".to_string(),
            icon: "💰".to_string(),
            variant: StatCardVariant::Primary,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "12.5% vs last month".to_string(),
            }),
            footer: Some("Last 30 days".to_string()),
        },
        KpiData {
            title: "Invoices".to_string(),
            value: "48".to_string(),
            icon: "🧾".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "8 from last month".to_string(),
            }),
            footer: Some("12 unpaid • 36 paid".to_string()),
        },
        KpiData {
            title: "Active Customers".to_string(),
            value: "124".to_string(),
            icon: "👥".to_string(),
            variant: StatCardVariant::Default,
            trend: Some(StatTrend {
                direction: TrendDirection::Flat,
                label: "Same as last month".to_string(),
            }),
            footer: Some("3 new this month".to_string()),
        },
        KpiData {
            title: "Low Stock Items".to_string(),
            value: "7".to_string(),
            icon: "⚠".to_string(),
            variant: StatCardVariant::Danger,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "2 more than last month".to_string(),
            }),
            footer: Some("2 items out of stock".to_string()),
        },
    ]
}

fn monthly_sales() -> Vec<MonthlySales> {
    vec![
        MonthlySales { month: "Jan".to_string(), amount: 185000.0 },
        MonthlySales { month: "Feb".to_string(), amount: 220000.0 },
        MonthlySales { month: "Mar".to_string(), amount: 195000.0 },
        MonthlySales { month: "Apr".to_string(), amount: 278000.0 },
        MonthlySales { month: "May".to_string(), amount: 312000.0 },
        MonthlySales { month: "Jun".to_string(), amount: 289500.0 },
    ]
}

fn recent_activity() -> Vec<ActivityItem> {
    vec![
        ActivityItem {
            icon: "🧾".to_string(),
            icon_bg: "#e8f0fe".to_string(),
            text: "Invoice <strong>INV-2026-0042</strong> was created for <strong>ABC Traders</strong> — PKR 45,000".to_string(),
            time: "2 minutes ago".to_string(),
        },
        ActivityItem {
            icon: "📥".to_string(),
            icon_bg: "#e8f8f0".to_string(),
            text: "Goods receipt recorded for <strong>Purchase Order PO-2026-0018</strong> — 150 units of Steel Rod".to_string(),
            time: "15 minutes ago".to_string(),
        },
        ActivityItem {
            icon: "👤".to_string(),
            icon_bg: "#fff8e8".to_string(),
            text: "New customer <strong>Alina Enterprises</strong> was registered".to_string(),
            time: "1 hour ago".to_string(),
        },
        ActivityItem {
            icon: "💰".to_string(),
            icon_bg: "#f0f0ff".to_string(),
            text: "Payment of <strong>PKR 22,500</strong> received from <strong>TechSource Ltd</strong> for Invoice INV-2026-0039".to_string(),
            time: "2 hours ago".to_string(),
        },
        ActivityItem {
            icon: "⚙".to_string(),
            icon_bg: "#f8f0ff".to_string(),
            text: "Production run <strong>PRD-2026-0012</strong> completed — 200 units of Premium Widget Alpha".to_string(),
            time: "3 hours ago".to_string(),
        },
    ]
}

fn low_stock_items() -> Vec<LowStockItem> {
    vec![
        LowStockItem {
            name: "Rubber Gasket Set".to_string(),
            code: "ITM-0005".to_string(),
            current_stock: 0,
            reorder_level: 50,
            unit: "pcs".to_string(),
            severity: Severity::Critical,
        },
        LowStockItem {
            name: "Assembly Robot Arm v3".to_string(),
            code: "ITM-0010".to_string(),
            current_stock: 2,
            reorder_level: 5,
            unit: "pcs".to_string(),
            severity: Severity::Critical,
        },
        LowStockItem {
            name: "Hydraulic Pump HPD-200".to_string(),
            code: "ITM-0004".to_string(),
            current_stock: 5,
            reorder_level: 10,
            unit: "pcs".to_string(),
            severity: Severity::Warning,
        },
        LowStockItem {
            name: "Safety Helmet (Yellow)".to_string(),
            code: "ITM-0009".to_string(),
            current_stock: 60,
            reorder_level: 100,
            unit: "pcs".to_string(),
            severity: Severity::Warning,
        },
        LowStockItem {
            name: "Copper Wire 2.5mm (100m)".to_string(),
            code: "ITM-0006".to_string(),
            current_stock: 25,
            reorder_level: 50,
            unit: "rolls".to_string(),
            severity: Severity::Info,
        },
        LowStockItem {
            name: "Steel Rod 12mm x 6m".to_string(),
            code: "ITM-0003".to_string(),
            current_stock: 80,
            reorder_level: 100,
            unit: "pcs".to_string(),
            severity: Severity::Info,
        },
        LowStockItem {
            name: "LED Panel Light 24W".to_string(),
            code: "ITM-0007".to_string(),
            current_stock: 200,
            reorder_level: 250,
            unit: "pcs".to_string(),
            severity: Severity::Info,
        },
    ]
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
    // Simple date formatting without chrono dependency at the page level
    // In a real implementation, use the calculations::formatting module
    "June 26, 2026".to_string()
}

// ============================================================================
// SVG Bar Chart Component
// ============================================================================

/// Props for the SalesChart component.
#[derive(Clone, PartialEq, Props)]
pub struct SalesChartProps {
    data: Vec<MonthlySales>,
}

/// Inline SVG bar chart for monthly sales.
/// No external chart library needed — pure Dioxus + SVG.
#[component]
fn SalesChart(props: SalesChartProps) -> Element {
    let max_amount = props.data.iter().map(|s| s.amount).fold(0.0_f64, f64::max);
    let bar_count = props.data.len();
    let bar_width_pct = 100.0 / bar_count as f64;
    let bar_gap_pct = bar_width_pct * 0.25; // 25% gap between bars
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 160.0;

    rsx! {
        div { class: "dashboard-chart",
            svg {
                view_box: "0 0 100 180",
                preserve_aspect_ratio: "xMidYMid meet",
                // Y-axis grid lines
                line { x1: "0", y1: "10", x2: "100", y2: "10", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "50", x2: "100", y2: "50", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "90", x2: "100", y2: "90", stroke: "#f0f0f0", stroke_width: "0.5" }
                line { x1: "0", y1: "130", x2: "100", y2: "130", stroke: "#f0f0f0", stroke_width: "0.5" }

                // Bars
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
                        // Bar
                        rect {
                            key: "{i}",
                            class: "dashboard-chart-bar",
                            x: "{x:.1}",
                            y: "{y:.1}",
                            width: "{bar_inner_pct:.1}",
                            height: "{bar_height:.1}",
                            rx: "2",
                        }
                        // Value label on top of bar
                        text {
                            class: "dashboard-chart-bar-value",
                            x: "{x + bar_inner_pct / 2.0:.1}",
                            y: "{y - 4.0:.1}",
                            "{s.amount:.0}"
                        }
                        // Month label below
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

/// Main dashboard page with KPI cards, sales chart, activity feed, and alerts.
#[component]
pub fn DashboardPage() -> Element {
    let kpis = kpi_data();
    let sales = monthly_sales();
    let activity = recent_activity();
    let alerts = low_stock_items();

    rsx! {
        div { class: "dashboard",

            // ── Header ──
            div { class: "dashboard-header",
                div {
                    h1 { "Dashboard" }
                    p { "Overview of your business performance" }
                }
                div { class: "dashboard-date", "{today_formatted()}" }
            }

            // ── KPI Cards ──
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

            // ── Two-column: Chart + Low Stock ──
            div { class: "dashboard-columns",

                // Sales Chart Section
                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "📈 Monthly Sales (2026)" }
                        a { href: "/reports/sales", "View Report →" }
                    }
                    div { class: "dashboard-section-body",
                        SalesChart { data: sales.clone() }
                    }
                }

                // Low Stock Alerts Section
                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "⚠ Low Stock Alerts" }
                        a { href: "/inventory/items", "Manage Inventory →" }
                    }
                    div { class: "dashboard-section-body",
                        {alerts.into_iter().map(|item| {
                            let dot_class = severity_class(&item.severity);
                            let label = severity_label(&item.severity);
                            let stock_text = if item.current_stock == 0 {
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

            // ── Recent Activity ──
            div { class: "dashboard-section dashboard-activity",
                div { class: "dashboard-section-header",
                    h2 { "🕐 Recent Activity" }
                    a { href: "/activity-log", "View All →" }
                }
                div { class: "dashboard-section-body",
                    {activity.into_iter().map(|a| {
                        rsx! {
                            div { key: "{a.time}", class: "dashboard-activity-item",
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

            // ── Quick Actions ──
            div { class: "dashboard-actions",
                Button {
                    variant: ButtonVariant::Primary,
                    icon: Some("🧾".to_string()),
                    onclick: move |_| {},
                    "New Invoice"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    icon: Some("📦".to_string()),
                    onclick: move |_| {},
                    "New Item"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    icon: Some("👤".to_string()),
                    onclick: move |_| {},
                    "New Customer"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    icon: Some("📊".to_string()),
                    onclick: move |_| {},
                    "View Reports"
                }
            }
        }
    }
}
