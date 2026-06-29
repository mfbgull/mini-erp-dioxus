use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ErrorBoundaryProps {
    children: Element,
    #[props(default)]
    fallback: Option<Element>,
}

#[component]
pub fn ErrorBoundary(props: ErrorBoundaryProps) -> Element {
    let mut error_state = use_signal(|| None::<String>);

    if let Some(err) = &*error_state.read() {
        if let Some(fb) = &props.fallback {
            return rsx! { {fb.clone()} };
        }
        return rsx! {
            div { class: "error-boundary",
                style: "padding: 24px; text-align: center; background: #fff5f5; border: 1px solid #feb2b2; border-radius: 8px; margin: 16px;",
                div { style: "font-size: 24px; margin-bottom: 8px;", "⚠️" }
                h3 { style: "color: #c53030; margin: 0 0 8px;", "Something went wrong" }
                p { style: "color: #742a2a; font-size: 13px; margin: 0 0 16px;", "{err}" }
                button {
                    style: "padding: 6px 16px; background: #c53030; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 13px;",
                    onclick: move |_| error_state.set(None),
                    "Try Again"
                }
            }
        };
    }

    rsx! { {props.children.clone()} }
}

pub fn catch_error(err: &str) -> String {
    format!("An error occurred: {}", err)
}
