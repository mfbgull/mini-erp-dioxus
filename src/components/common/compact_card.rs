use dioxus::prelude::*;

const COMPACT_CARD_CSS: &str = r#"
.compact-card-list { display: flex; flex-direction: column; gap: 8px; padding: 0 16px 80px; }
.compact-card { display: flex; align-items: center; justify-content: space-between; padding: 14px 16px; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 10px; cursor: pointer; transition: box-shadow 0.15s; }
.compact-card:hover { box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.compact-card:active { transform: scale(0.99); }
.compact-card-main { display: flex; flex-direction: column; gap: 3px; flex: 1; min-width: 0; }
.compact-card-header { display: flex; align-items: center; gap: 8px; }
.compact-card-label { font-size: 14px; font-weight: 600; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.compact-card-badge { display: inline-flex; align-items: center; padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; white-space: nowrap; }
.compact-card-badge.badge-green { background: rgba(40,167,69,0.1); color: #28a745; }
.compact-card-badge.badge-yellow { background: rgba(255,193,7,0.15); color: #d4a017; }
.compact-card-badge.badge-red { background: rgba(220,53,69,0.12); color: #dc3545; }
.compact-card-badge.badge-blue { background: rgba(74,144,217,0.1); color: #4a90d9; }
.compact-card-badge.badge-gray { background: rgba(108,117,125,0.1); color: #6c757d; }
.compact-card-subtitle { font-size: 12px; color: var(--text-secondary, #666); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.compact-card-value { font-size: 13px; color: var(--text-primary); font-weight: 500; }
.compact-card-chevron { font-size: 16px; color: var(--text-secondary, #999); margin-left: 8px; flex-shrink: 0; }
.compact-card-actions { display: flex; gap: 4px; margin-left: 8px; }
.compact-card-action { padding: 4px 8px; font-size: 12px; border: 1px solid var(--border-color); border-radius: 4px; background: #fff; cursor: pointer; color: var(--text-primary); }
.compact-card-action:hover { background: var(--bg-hover, #f5f5f5); }
.compact-card-search { padding: 0 16px 12px; }
.compact-card-search input { width: 100%; padding: 10px 14px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; font-size: 14px; background: #fff; }
.compact-card-empty { text-align: center; padding: 48px 16px; color: var(--text-secondary, #666); font-size: 14px; }
"#;

#[derive(Clone, PartialEq)]
pub enum BadgeColor {
    Green,
    Yellow,
    Red,
    Blue,
    Gray,
}

impl BadgeColor {
    pub fn class(&self) -> &'static str {
        match self {
            BadgeColor::Green => "badge-green",
            BadgeColor::Yellow => "badge-yellow",
            BadgeColor::Red => "badge-red",
            BadgeColor::Blue => "badge-blue",
            BadgeColor::Gray => "badge-gray",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CompactCardProps {
    pub label: String,
    #[props(default)]
    pub subtitle: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub badge_text: Option<String>,
    #[props(default)]
    pub badge_color: Option<BadgeColor>,
    pub onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn CompactCard(props: CompactCardProps) -> Element {
    let badge_class = props.badge_color.as_ref().map(|c| c.class()).unwrap_or("badge-gray");
    rsx! {
        div { class: "compact-card", onclick: move |e| props.onclick.call(e),
            div { class: "compact-card-main",
                div { class: "compact-card-header",
                    span { class: "compact-card-label", "{props.label}" }
                    if let Some(badge) = &props.badge_text {
                        span { class: "compact-card-badge {badge_class}", "{badge}" }
                    }
                }
                if let Some(sub) = &props.subtitle {
                    span { class: "compact-card-subtitle", "{sub}" }
                }
                if let Some(val) = &props.value {
                    span { class: "compact-card-value", "{val}" }
                }
            }
            span { class: "compact-card-chevron", "›" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CompactCardListProps {
    pub search_placeholder: Option<String>,
    pub empty_message: Option<String>,
    pub children: Element,
}

#[component]
pub fn CompactCardList(props: CompactCardListProps) -> Element {
    let mut search = use_signal(|| String::new());
    let placeholder = props.search_placeholder.unwrap_or_else(|| "Search…".to_string());
    let empty_msg = props.empty_message.unwrap_or_else(|| "No items found.".to_string());

    rsx! {
        style { "{COMPACT_CARD_CSS}" }
        div { class: "compact-card-search",
            input {
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search.read()}",
                oninput: move |e| search.set(e.value()),
            }
        }
        div { class: "compact-card-list",
            {props.children.clone()}
        }
    }
}
