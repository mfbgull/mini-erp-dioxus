use dioxus::prelude::*;

const STAT_BLOCKS_CSS: &str = r#"
.dashboard-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.dashboard-stat { display: flex; align-items: center; gap: 12px; padding: 14px; background: var(--surface-secondary); border-radius: 8px; }
.dashboard-stat-icon { width: 40px; height: 40px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 18px; flex-shrink: 0; }
.dashboard-stat-icon.blue { background: rgba(59, 130, 246, 0.10); color: var(--info); }
.dashboard-stat-icon.green { background: var(--success-light); color: var(--success); }
.dashboard-stat-icon.yellow { background: var(--warning-light); color: var(--warning); }
.dashboard-stat-icon.red { background: var(--danger-light); color: var(--danger); }
.dashboard-stat-icon.purple { background: rgba(139, 92, 246, 0.10); color: #7C3AED; }
.dashboard-stat-info { display: flex; flex-direction: column; gap: 2px; }
.dashboard-stat-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
.dashboard-stat-label { font-size: 12px; color: var(--text-secondary); }
.dashboard-stat-trend { font-size: 11px; font-weight: 600; }
.dashboard-stat-trend.up { color: var(--success); }
.dashboard-stat-trend.down { color: var(--danger); }
"#;

#[derive(Clone, PartialEq)]
pub enum StatColor { Blue, Green, Yellow, Red, Purple }

#[derive(Clone, PartialEq)]
pub struct DashboardStatItem {
    pub icon: String,
    pub label: String,
    pub value: String,
    pub color: StatColor,
    pub trend: Option<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct DashboardStatsProps {
    pub items: Vec<DashboardStatItem>,
}

#[component]
pub fn DashboardStats(props: DashboardStatsProps) -> Element {
    let color_fn = |c: &StatColor| match c {
        StatColor::Blue => "blue",
        StatColor::Green => "green",
        StatColor::Yellow => "yellow",
        StatColor::Red => "red",
        StatColor::Purple => "purple",
    };
    rsx! {
        style { "{STAT_BLOCKS_CSS}" }
        div { class: "dashboard-stats",
            for item in props.items.iter() {
                div { class: "dashboard-stat",
                    div { class: "dashboard-stat-icon {color_fn(&item.color)}", "{item.icon}" }
                    div { class: "dashboard-stat-info",
                        span { class: "dashboard-stat-value", "{item.value}" }
                        span { class: "dashboard-stat-label", "{item.label}" }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MiniBarChartProps {
    pub data: Vec<(String, f64)>,
    #[props(default = 120)]
    pub bar_height: u32,
}

#[component]
pub fn MiniBarChart(props: MiniBarChartProps) -> Element {
    let max_val = props.data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max).max(1.0);
    rsx! {
        div { style: "display: flex; align-items: flex-end; gap: 4px; height: {props.bar_height}px; padding-top: 8px;",
            for (label, value) in props.data.iter() {
                div { style: "display: flex; flex-direction: column; align-items: center; gap: 2px; flex: 1;",
                    div {
                        style: "width: 100%; max-width: 40px; background: var(--accent); border-radius: 4px 4px 0 0; height: {((value / max_val) * (props.bar_height as f64 - 20.0)) as u32}px; min-height: 2px;",
                    }
                    span { style: "font-size: 10px; color: var(--text-secondary); white-space: nowrap;", "{label}" }
                }
            }
        }
    }
}
