use dioxus::prelude::*;

const TOPMENU_CSS: &str = r#"
.top-menu { display: flex; align-items: center; justify-content: space-between; height: 52px; padding: 0 20px; background: var(--surface); border-bottom: 1px solid var(--border-color); position: sticky; top: 0; z-index: 50; }
.top-menu-left { display: flex; align-items: center; gap: 12px; }
.top-menu-title { font-size: 16px; font-weight: 600; color: var(--text-primary); }
.top-menu-right { display: flex; align-items: center; gap: 12px; }
.top-menu-search { display: flex; align-items: center; gap: 6px; padding: 6px 12px; border: 1px solid var(--border-color); border-radius: 8px; background: var(--surface-tertiary); cursor: pointer; font-size: 13px; color: var(--text-secondary); min-width: 200px; }
.top-menu-search kbd { padding: 1px 5px; background: var(--surface); border: 1px solid var(--border-color); border-radius: 3px; font-size: 11px; margin-left: auto; }
.top-menu-btn { display: flex; align-items: center; justify-content: center; width: 36px; height: 36px; border-radius: 8px; border: none; background: transparent; cursor: pointer; font-size: 16px; color: var(--text-secondary); }
.top-menu-btn:hover { background: var(--surface-tertiary); }
.top-menu-user { display: flex; align-items: center; gap: 8px; padding: 4px 10px; border-radius: 8px; cursor: pointer; font-size: 13px; color: var(--text-primary); }
.top-menu-user:hover { background: var(--surface-tertiary); }
.top-menu-avatar { width: 28px; height: 28px; border-radius: 50%; background: var(--accent); color: #ffffff; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 600; }
"#;

#[derive(Props, Clone, PartialEq)]
pub struct TopMenuProps {
    #[props(default)]
    title: Option<String>,
    user_name: Option<String>,
    on_search: Option<EventHandler<MouseEvent>>,
    on_notifications: Option<EventHandler<MouseEvent>>,
    on_user_menu: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn TopMenu(props: TopMenuProps) -> Element {
    let initial = props.user_name.as_deref().unwrap_or("?").chars().next().unwrap_or('?').to_uppercase().to_string();

    rsx! {
        style { "{TOPMENU_CSS}" }
        div { class: "top-menu",
            div { class: "top-menu-left",
                if let Some(title) = &props.title {
                    span { class: "top-menu-title", "{title}" }
                }
            }
            div { class: "top-menu-right",
                button {
                    class: "top-menu-search",
                    onclick: move |e| { if let Some(cb) = &props.on_search { cb.call(e); } },
                    "🔍 Search…"
                    kbd { "/" }
                }
                button {
                    class: "top-menu-btn",
                    onclick: move |e| { if let Some(cb) = &props.on_notifications { cb.call(e); } },
                    "🔔"
                }
                div {
                    class: "top-menu-user",
                    onclick: move |e| { if let Some(cb) = &props.on_user_menu { cb.call(e); } },
                    div { class: "top-menu-avatar", "{initial}" }
                    if let Some(name) = &props.user_name {
                        span { "{name}" }
                    }
                }
            }
        }
    }
}
