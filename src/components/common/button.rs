//! Button component — primary, secondary, danger, ghost, success, warning variants.
//!
//! Supports sizes (sm, md, lg), loading state with spinner, icon prefix,
//! full-width (block) mode, and disabled state.
//!
//! # Usage
//!
//! ```ignore
//! Button {
//!     variant: ButtonVariant::Primary,
//!     size: ButtonSize::Md,
//!     loading: false,
//!     disabled: false,
//!     block: false,
//!     icon: "★",
//!     onclick: move |_| { /* action */ },
//!     "Submit"
//! }
//! ```

use dioxus::prelude::*;

/// Button visual variant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
    Success,
    Warning,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

impl ButtonVariant {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Primary => "cb-btn-primary",
            Self::Secondary => "cb-btn-secondary",
            Self::Danger => "cb-btn-danger",
            Self::Ghost => "cb-btn-ghost",
            Self::Success => "cb-btn-success",
            Self::Warning => "cb-btn-warning",
        }
    }
}

/// Button size preset.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Md
    }
}

impl ButtonSize {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Sm => "cb-btn-sm",
            Self::Md => "cb-btn-md",
            Self::Lg => "cb-btn-lg",
        }
    }
}

/// Button component properties.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Visual variant.
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,

    /// Size preset.
    #[props(default = ButtonSize::Md)]
    pub size: ButtonSize,

    /// Whether the button is in a loading state (shows spinner).
    #[props(default = false)]
    pub loading: bool,

    /// Whether the button is disabled.
    #[props(default = false)]
    pub disabled: bool,

    /// Whether the button takes full width of its container.
    #[props(default = false)]
    pub block: bool,

    /// Optional icon displayed before the label.
    #[props(default)]
    pub icon: Option<String>,

    /// Optional CSS class to append.
    #[props(default)]
    pub class: Option<String>,

    /// Click handler.
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    /// Button type attribute.
    #[props(default = "button".to_string())]
    pub r#type: String,

    /// Button label (children).
    pub children: Element,
}

/// Primary button component.
///
/// # Examples
/// ```ignore
/// // Primary button
/// Button { "Save" }
///
/// // Danger button with icon and loading state
/// Button {
///     variant: ButtonVariant::Danger,
///     loading: true,
///     icon: "🗑",
///     "Delete"
/// }
/// ```
pub fn Button(props: ButtonProps) -> Element {
    let mut classes = vec![
        "cb-btn",
        props.variant.css_class(),
        props.size.css_class(),
    ];

    if props.block {
        classes.push("cb-btn-block");
    }

    if let Some(extra) = &props.class {
        classes.push(extra);
    }

    let class_str = classes.join(" ");

    let spinner_class = if matches!(props.variant, ButtonVariant::Ghost | ButtonVariant::Warning | ButtonVariant::Secondary) {
        "cb-btn-spinner cb-btn-spinner-dark"
    } else {
        "cb-btn-spinner"
    };

    let onclick = props.onclick;

    rsx! {
        button {
            class: "{class_str}",
            r#type: "{props.r#type}",
            disabled: props.disabled || props.loading,
            onclick: move |evt: MouseEvent| {
                if let Some(cb) = onclick {
                    cb.call(evt);
                }
            },
            // Show spinner when loading
            if props.loading {
                div { class: "{spinner_class}" }
            }
            // Show icon when not loading
            if !props.loading {
                if let Some(icon) = &props.icon {
                    span { "{icon}" }
                }
            }
            // Label
            {props.children}
        }
    }
}
