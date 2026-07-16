//! Stat Card component — KPI display card with value, icon, and trend.
//!
//! Variants: Default, Primary, Success, Danger, Warning with colour accents.
//! Trend indicators: Up (↑), Down (↓), Flat (→) with colour coding.

use dioxus::prelude::*;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StatCardVariant {
    Default,
    Primary,
    Success,
    Danger,
    Warning,
}

impl Default for StatCardVariant {
    fn default() -> Self {
        Self::Default
    }
}

impl StatCardVariant {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Primary => "cb-stat-primary cb-stat-accent",
            Self::Success => "cb-stat-success cb-stat-accent-success",
            Self::Danger => "cb-stat-danger cb-stat-accent-danger",
            Self::Warning => "cb-stat-warning cb-stat-accent-warning",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TrendDirection {
    Up,
    Down,
    Flat,
}

impl TrendDirection {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Up => "cb-stat-trend-up",
            Self::Down => "cb-stat-trend-down",
            Self::Flat => "cb-stat-trend-flat",
        }
    }
    fn arrow(&self) -> &'static str {
        match self {
            Self::Up => "↑",
            Self::Down => "↓",
            Self::Flat => "→",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StatTrend {
    pub direction: TrendDirection,
    pub label: String,
}

// ============================================================================
// Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct StatCardProps {
    pub title: String,
    pub value: String,
    #[props(default)]
    pub icon: Option<String>,
    #[props(default = StatCardVariant::Default)]
    pub variant: StatCardVariant,
    #[props(default)]
    pub trend: Option<StatTrend>,
    #[props(default)]
    pub footer: Option<String>,
    #[props(default)]
    pub onclick: Option<EventHandler<Event<MouseData>>>,
    #[props(default)]
    pub class: Option<String>,
    /// Show shimmer skeleton placeholder
    #[props(default = false)]
    pub loading: bool,
}

/// KPI stat card component.
pub fn StatCard(props: StatCardProps) -> Element {
    let variant_class = props.variant.css_class();
    let card_class = format!(
        "cb-stat {} {}",
        variant_class,
        if props.loading { "cb-stat-loading" } else { "" },
    );

    let onclick = props.onclick.clone();

    if props.loading {
        return rsx! {
            div { class: "{card_class}",
                div { class: "cb-stat-header",
                    span { class: "cb-stat-title skeleton", style: "width: 60%; height: 10px;" }
                }
                div { class: "cb-stat-value skeleton", style: "width: 45%; height: 28px; margin-top: 4px;" }
                div { class: "cb-stat-footer",
                    span { class: "skeleton", style: "width: 35%; height: 10px;" }
                }
            }
        };
    }

    rsx! {
        div {
            class: "{card_class}",
            onclick: move |e| {
                if let Some(cb) = &onclick {
                    cb.call(e);
                }
            },
            div { class: "cb-stat-header",
                span { class: "cb-stat-title", "{props.title}" }
                if let Some(icon) = &props.icon {
                    span { class: "cb-stat-icon", "{icon}" }
                }
            }
            div { class: "cb-stat-value", "{props.value}" }
            div { class: "cb-stat-footer",
                if let Some(trend) = &props.trend {
                    span { class: "cb-stat-trend {trend.direction.css_class()}",
                        "{trend.direction.arrow()} {trend.label}"
                    }
                }
                if let Some(footer) = &props.footer {
                    span { "{footer}" }
                }
            }
        }
    }
}
