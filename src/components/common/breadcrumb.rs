//! Breadcrumb component — hierarchical navigation path with links.
//!
//! Supports a list of `BreadcrumbItem` segments, each with a label
//! and optional route path. The last item is rendered as plain text
//! (the current page), all prior items are clickable links.
//!
//! # Usage
//!
//! ```ignore
//! Breadcrumb {
//!     items: vec![
//!         BreadcrumbItem { label: "Sales".into(), path: Some("/sales".into()) },
//!         BreadcrumbItem { label: "Invoices".into(), path: Some("/sales/invoices".into()) },
//!         BreadcrumbItem { label: "INV-2026-0042".into(), path: None },
//!     ],
//! }
//! ```

use dioxus::prelude::*;

/// A single breadcrumb segment.
#[derive(Clone, PartialEq, Debug)]
pub struct BreadcrumbItem {
    /// Display label for this segment.
    pub label: String,
    /// Optional route path. `None` for the current (last) item.
    pub path: Option<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbProps {
    /// Ordered list of breadcrumb segments (most-ancestor first).
    pub items: Vec<BreadcrumbItem>,
    #[props(default)]
    pub class: Option<String>,
}

/// Breadcrumb navigation component.
#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    let len = props.items.len();
    let class = format!("breadcrumb{}", props.class.as_ref().map_or(String::new(), |c| format!(" {c}")));

    rsx! {
        nav {
            class: "{class}",
            "aria-label": "Breadcrumb",
            {props.items.into_iter().enumerate().map(|(i, item)| {
                let is_last = i == len - 1;
                rsx! {
                    if i > 0 {
                        span {
                            class: "breadcrumb-separator",
                            "aria-hidden": "true",
                            "/"
                        }
                    }
                    if is_last || item.path.is_none() {
                        span {
                            class: "breadcrumb-current",
                            "aria-current": if is_last { "page" } else { "false" },
                            "{item.label}"
                        }
                    } else if let Some(path) = item.path {
                        Link {
                            to: "{path}",
                            class: "breadcrumb-link",
                            "{item.label}"
                        }
                    }
                }
            })}
        }
    }
}
