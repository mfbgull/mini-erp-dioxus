use dioxus::prelude::*;

const SHORTCUT_BAR_CSS: &str = r#"
.shortcut-bar { display: none; position: fixed; bottom: 0; left: 0; right: 0; height: 60px; background: var(--surface); border-top: 1px solid var(--border-color); z-index: 100; padding: 0 8px; }
.shortcut-bar-inner { display: flex; align-items: center; justify-content: space-around; height: 100%; }
.shortcut-bar-item { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 6px 12px; border-radius: 8px; cursor: pointer; text-decoration: none; color: var(--text-secondary, #666); font-size: 10px; background: none; border: none; }
.shortcut-bar-item.active { color: var(--accent); }
.shortcut-bar-item:hover { background: var(--surface-tertiary); }
.shortcut-bar-icon { font-size: 20px; line-height: 1; }
.shortcut-bar-label { font-size: 10px; font-weight: 500; }
@media (max-width: 768px) { .shortcut-bar { display: block; } }
"#;

#[derive(Clone, PartialEq)]
pub struct ShortcutItem {
    pub icon: String,
    pub label: String,
    pub route: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct ShortcutBarProps {
    items: Vec<ShortcutItem>,
    active_route: String,
    on_navigate: EventHandler<String>,
}

#[component]
pub fn ShortcutBar(props: ShortcutBarProps) -> Element {
    rsx! {
        style { "{SHORTCUT_BAR_CSS}" }
        nav { class: "shortcut-bar",
            div { class: "shortcut-bar-inner",
                for item in props.items.iter() {
                    button {
                        class: if item.route == props.active_route { "shortcut-bar-item active" } else { "shortcut-bar-item" },
                        onclick: {
                            let route = item.route.clone();
                            move |_| props.on_navigate.call(route.clone())
                        },
                        span { class: "shortcut-bar-icon", "{item.icon}" }
                        span { class: "shortcut-bar-label", "{item.label}" }
                    }
                }
            }
        }
    }
}
