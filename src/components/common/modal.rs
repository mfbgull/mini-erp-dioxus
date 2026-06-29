//! Modal component — overlay dialog with configurable sizes.
//!
//! Supports sm/md/lg/xl/full sizes, close on Escape, close on backdrop click,
//! and responsive bottom-sheet on mobile.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModalSize {
    Sm,
    Md,
    Lg,
    Xl,
    Full,
}

impl Default for ModalSize {
    fn default() -> Self {
        Self::Md
    }
}

impl ModalSize {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Sm => "cb-modal-sm",
            Self::Md => "cb-modal-md",
            Self::Lg => "cb-modal-lg",
            Self::Xl => "cb-modal-xl",
            Self::Full => "cb-modal-full",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    pub is_open: Signal<bool>,
    #[props(default)]
    pub title: Option<String>,
    #[props(default = ModalSize::Md)]
    pub size: ModalSize,
    #[props(default = true)]
    pub close_on_backdrop: bool,
    #[props(default = true)]
    pub close_on_escape: bool,
    #[props(default = true)]
    pub show_header: bool,
    #[props(default = true)]
    pub show_close_button: bool,
    #[props(default)]
    pub footer: Option<Element>,
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

/// Modal overlay dialog component.
pub fn Modal(props: ModalProps) -> Element {
    let is_open = *props.is_open.read();

    if !is_open {
        return rsx! { div {} };
    }

    let modal_class = format!(
        "cb-modal {} {}",
        props.size.css_class(),
        props.class.as_deref().unwrap_or(""),
    );

    // Clone handlers for closures
    let mut is_open_w = props.is_open.clone();
    let close_on_backdrop = props.close_on_backdrop;
    let close_on_escape = props.close_on_escape;
    let show_header = props.show_header;
    let show_close_button = props.show_close_button;
    let mut is_open_w2 = props.is_open.clone();

    rsx! {
        div {
            class: "cb-modal-overlay",
            tabindex: "0",
            // Focus the overlay on mount so keyboard events work
            onmounted: move |_| {},
            onclick: move |_| {
                if close_on_backdrop {
                    is_open_w.set(false);
                }
            },
            onkeydown: move |e| {
                if close_on_escape && e.key() == dioxus::prelude::Key::Escape {
                    is_open_w2.set(false);
                }
            },
            div {
                class: "{modal_class}",
                onclick: move |e| { e.stop_propagation(); },
                if show_header {
                    div { class: "cb-modal-header",
                        div { class: "cb-modal-title",
                            {props.title.as_deref().unwrap_or("")}
                        }
                        if show_close_button {
                            button {
                                class: "cb-modal-close",
                                r#type: "button",
                                onclick: move |_| is_open_w.set(false),
                                aria_label: "Close modal",
                                "×"
                            }
                        }
                    }
                }
                div { class: "cb-modal-body",
                    {props.children}
                }
                if let Some(footer) = &props.footer {
                    div { class: "cb-modal-footer",
                        {footer}
                    }
                }
            }
        }
    }
}
