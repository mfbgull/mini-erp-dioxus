use dioxus::prelude::*;

const LOADER_CSS: &str = r#"
.page-loader { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 400px; gap: 12px; }
.page-loader .spinner { width: 32px; height: 32px; border: 3px solid var(--border-color, #e0e0e0); border-top-color: var(--accent, #4a90d9); border-radius: 50%; animation: spin 0.8s linear infinite; }
.page-loader .spinner-sm { width: 16px; height: 16px; border-width: 2px; }
.page-loader .spinner-lg { width: 48px; height: 48px; border-width: 4px; }
.page-loader p { font-size: 13px; color: var(--text-secondary, #666); margin: 0; }
@keyframes spin { to { transform: rotate(360deg); } }
"#;

#[derive(Clone, PartialEq)]
pub enum LoaderSize {
    Small,
    Medium,
    Large,
}

#[derive(Props, Clone, PartialEq)]
pub struct PageLoaderProps {
    #[props(default = "Loading…".to_string())]
    message: String,
    #[props(default = LoaderSize::Medium)]
    size: LoaderSize,
    #[props(default = false)]
    inline: bool,
}

#[component]
pub fn PageLoader(props: PageLoaderProps) -> Element {
    let size_class = match props.size {
        LoaderSize::Small => "spinner-sm",
        LoaderSize::Medium => "",
        LoaderSize::Large => "spinner-lg",
    };
    if props.inline {
        rsx! {
            style { "{LOADER_CSS}" }
            span { class: "page-loader", style: "display: inline-flex; min-height: auto; gap: 6px;",
                div { class: "spinner {size_class}" }
                if !props.message.is_empty() {
                    span { style: "font-size: 12px; color: var(--text-secondary);", "{props.message}" }
                }
            }
        }
    } else {
        rsx! {
            style { "{LOADER_CSS}" }
            div { class: "page-loader",
                div { class: "spinner {size_class}" }
                p { "{props.message}" }
            }
        }
    }
}
