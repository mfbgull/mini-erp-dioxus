use dioxus::prelude::*;

const DASHBOARD_CSS: &str = r#"
.dashboard-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--space-4); padding: 0; }
.dashboard-block { background: var(--surface); border: 1px solid var(--border-color); border-radius: var(--radius-md); overflow: hidden; transition: box-shadow 0.2s; }
.dashboard-block:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
.dashboard-block-header { display: flex; align-items: center; justify-content: space-between; padding: var(--space-3) var(--space-4); border-bottom: 1px solid var(--border-color); }
.dashboard-block-title { font-size: var(--text-sm); font-weight: 600; color: var(--text-primary); margin: 0; }
.dashboard-block-actions { display: flex; gap: var(--space-1); }
.dashboard-block-action { padding: 4px 6px; border: none; background: transparent; cursor: pointer; font-size: var(--text-base); color: var(--text-secondary); border-radius: 4px; }
.dashboard-block-action:hover { background: var(--surface-tertiary); }
.dashboard-block-body { padding: var(--space-4); min-height: 120px; }
.dashboard-block-loading { display: flex; align-items: center; justify-content: center; height: 120px; color: var(--text-secondary); font-size: var(--text-sm); }
.dashboard-block-wide { grid-column: span 2; }
.dashboard-block-full { grid-column: 1 / -1; }
@media (max-width: 768px) {
    .dashboard-grid { grid-template-columns: 1fr; }
    .dashboard-block-wide { grid-column: span 1; }
}
"#;

#[derive(Clone, PartialEq)]
pub enum BlockSize {
    Normal,
    Wide,
    Full,
}

impl BlockSize {
    pub fn class(&self) -> &'static str {
        match self {
            BlockSize::Normal => "",
            BlockSize::Wide => "dashboard-block-wide",
            BlockSize::Full => "dashboard-block-full",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DashboardBlockProps {
    pub title: String,
    #[props(default = BlockSize::Normal)]
    pub size: BlockSize,
    #[props(default)]
    pub loading: bool,
    pub children: Element,
}

#[component]
pub fn DashboardBlock(props: DashboardBlockProps) -> Element {
    let size_class = props.size.class();
    rsx! {
        style { "{DASHBOARD_CSS}" }
        div { class: "dashboard-block {size_class}",
            div { class: "dashboard-block-header",
                h3 { class: "dashboard-block-title", "{props.title}" }
            }
            div { class: "dashboard-block-body",
                if props.loading {
                    div { class: "dashboard-block-loading", "Loading…" }
                } else {
                    {props.children.clone()}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DashboardGridProps {
    pub children: Element,
}

#[component]
pub fn DashboardGrid(props: DashboardGridProps) -> Element {
    rsx! {
        style { "{DASHBOARD_CSS}" }
        div { class: "dashboard-grid",
            {props.children.clone()}
        }
    }
}
