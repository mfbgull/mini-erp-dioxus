//! Toast notification system.
//!
//! Provides `ToastProvider` + `use_toast()` hook for auto-dismissing
//! notifications (success, error, warning, info) from anywhere in the app.
//!
//! # Setup
//!
//! ```ignore
//! // In root component:
//! ToastProvider { Router::<Route> {} }
//!
//! // Anywhere:
//! let toast = use_toast();
//! toast.success("Saved", "Invoice created.");
//! toast.error("Error", "Validation failed.");
//! ```

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for unique toast IDs.
static NEXT_TOAST_ID: AtomicU64 = AtomicU64::new(1);

/// Toast visual type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Error => "✗",
            Self::Warning => "⚠",
            Self::Info => "ℹ",
        }
    }
    fn css_class(&self) -> &'static str {
        match self {
            Self::Success => "cb-toast-success",
            Self::Error => "cb-toast-error",
            Self::Warning => "cb-toast-warning",
            Self::Info => "cb-toast-info",
        }
    }
}

/// A single toast notification entry.
#[derive(Clone)]
struct ToastEntry {
    id: u64,
    r#type: ToastType,
    title: String,
    message: String,
    duration_ms: u64,
    exiting: bool,
}

/// Internal toast state shared via context.
#[derive(Clone)]
struct ToastState {
    toasts: Signal<Vec<ToastEntry>>,
}

/// Public API for showing toasts.
#[derive(Clone)]
pub struct ToastManager {
    toasts: Signal<Vec<ToastEntry>>,
}

impl ToastManager {
    /// Show a success toast (auto-dismisses after 4s).
    pub fn success(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.show(ToastType::Success, title, message, 4000);
    }

    /// Show an error toast (auto-dismisses after 6s).
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.show(ToastType::Error, title, message, 6000);
    }

    /// Show a warning toast (auto-dismisses after 4s).
    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.show(ToastType::Warning, title, message, 4000);
    }

    /// Show an info toast (auto-dismisses after 3s).
    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.show(ToastType::Info, title, message, 3000);
    }

    fn show(&mut self, r#type: ToastType, title: impl Into<String>, message: impl Into<String>, duration_ms: u64) {
        let id = NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed);
        let effective_duration = if duration_ms > 0 { duration_ms } else { 4000 };

        self.toasts.write().push(ToastEntry {
            id,
            r#type,
            title: title.into(),
            message: message.into(),
            duration_ms: effective_duration,
            exiting: false,
        });

        let mut toasts = self.toasts.clone();
        spawn(async move {
            crate::utils::sleep(std::time::Duration::from_millis(effective_duration)).await;

            {
                let mut guard = toasts.write();
                if let Some(entry) = guard.iter_mut().find(|t| t.id == id) {
                    entry.exiting = true;
                }
            }

            crate::utils::sleep(std::time::Duration::from_millis(250)).await;
            toasts.write().retain(|t| t.id != id);
        });
    }

    /// Dismiss a specific toast by id immediately.
    pub fn dismiss(&mut self, id: u64) {
        self.toasts.write().retain(|t| t.id != id);
    }
}

/// Provides the toast context to the component tree.
///
/// Should wrap the app root, typically around the Router.
#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_signal(|| Vec::<ToastEntry>::new());
    let state = use_context_provider(|| ToastState { toasts });

    rsx! {
        {children}
        div { class: "cb-toast-container",
            {state.toasts.iter().map(|entry| {
                let type_class = entry.r#type.css_class();
                let exit_class = if entry.exiting { " cb-toast-exit" } else { "" };
                let toast_id = entry.id;
                let mut manager = ToastManager { toasts: state.toasts.clone() };

                rsx! {
                    div {
                        key: "{entry.id}",
                        class: "cb-toast {type_class}{exit_class}",
                        role: "alert",
                        div { class: "cb-toast-icon", "{entry.r#type.icon()}" }
                        div { class: "cb-toast-content",
                            if !entry.title.is_empty() {
                                div { class: "cb-toast-title", "{entry.title}" }
                            }
                            if !entry.message.is_empty() {
                                div { class: "cb-toast-message", "{entry.message}" }
                            }
                        }
                        button {
                            class: "cb-toast-close",
                            r#type: "button",
                            onclick: move |_| manager.dismiss(toast_id),
                            aria_label: "Dismiss",
                            "×"
                        }
                    }
                }
            })}
        }
    }
}

/// Hook to access the toast manager from any component.
///
/// Must be called within a `ToastProvider` subtree.
pub fn use_toast() -> ToastManager {
    let state = use_context::<ToastState>();
    ToastManager {
        toasts: state.toasts,
    }
}
