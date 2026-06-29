//! Dropdown Menu component — trigger button with positioned menu.
//!
//! Supports custom trigger, 4 positions, danger items, dividers, click-away close.
//!
//! # Usage
//!
//! ```ignore
//! DropdownMenu {
//!     trigger: rsx! { button { "☰ Menu" } },
//!     DropdownItem { label: "Edit", onclick: move |_| {} }
//!     DropdownDivider {}
//!     DropdownItem { label: "Delete", variant: DropdownItemVariant::Danger, onclick: move |_| {} }
//! }
//! ```

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DropdownPosition {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl Default for DropdownPosition {
    fn default() -> Self {
        Self::BottomLeft
    }
}

impl DropdownPosition {
    fn css_class(&self) -> &'static str {
        match self {
            Self::BottomLeft => "cb-dropdown-bottom-left",
            Self::BottomRight => "cb-dropdown-bottom-right",
            Self::TopLeft => "cb-dropdown-top-left",
            Self::TopRight => "cb-dropdown-top-right",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DropdownItemVariant {
    Normal,
    Danger,
}

impl Default for DropdownItemVariant {
    fn default() -> Self {
        Self::Normal
    }
}

// ============================================================================
// Dropdown Menu
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuProps {
    pub trigger: Element,
    #[props(default = DropdownPosition::BottomLeft)]
    pub position: DropdownPosition,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut toggle = move || { if !props.disabled { let val = *is_open.read(); is_open.set(!val); } };

    rsx! {
        div {
            class: "cb-dropdown-wrapper {props.class.as_deref().unwrap_or_default()}",
            div {
                onclick: move |e| { e.stop_propagation(); toggle(); },
                {props.trigger}
            }
            if *is_open.read() {
                div {
                    class: "cb-dropdown-menu {props.position.css_class()}",
                    onclick: move |e| { e.stop_propagation(); },
                    {props.children}
                }
            }
            if *is_open.read() {
                div {
                    style: "position: fixed; inset: 0; z-index: 499;",
                    onclick: move |_| { is_open.set(false); },
                }
            }
        }
    }
}

// ============================================================================
// Dropdown Item
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownItemProps {
    #[props(default)]
    pub icon: Option<String>,
    pub label: String,
    #[props(default = DropdownItemVariant::Normal)]
    pub variant: DropdownItemVariant,
    #[props(default)]
    pub shortcut: Option<String>,
    pub onclick: EventHandler<Event<MouseData>>,
}

pub fn DropdownItem(props: DropdownItemProps) -> Element {
    let item_class = match props.variant {
        DropdownItemVariant::Normal => "cb-dropdown-item",
        DropdownItemVariant::Danger => "cb-dropdown-item cb-dropdown-item-danger",
    };
    let onclick = props.onclick.clone();

    rsx! {
        button {
            class: "{item_class}",
            r#type: "button",
            onclick: move |e| onclick.call(e),
            if let Some(icon) = &props.icon {
                span { class: "cb-dropdown-item-icon", "{icon}" }
            }
            span { class: "cb-dropdown-item-label", "{props.label}" }
            if let Some(shortcut) = &props.shortcut {
                span { class: "cb-dropdown-item-shortcut", "{shortcut}" }
            }
        }
    }
}

// ============================================================================
// Dropdown Divider
// ============================================================================

#[component]
pub fn DropdownDivider() -> Element {
    rsx! { div { class: "cb-dropdown-divider" } }
}
