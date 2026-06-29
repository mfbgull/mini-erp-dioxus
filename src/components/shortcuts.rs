use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct Shortcut {
    pub keys: String,
    pub description: String,
    pub category: String,
}

pub fn default_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut { keys: "?".into(), description: "Show keyboard shortcuts".into(), category: "General".into() },
        Shortcut { keys: "Ctrl+K".into(), description: "Quick search".into(), category: "General".into() },
        Shortcut { keys: "/".into(), description: "Focus search".into(), category: "General".into() },
        Shortcut { keys: "Ctrl+P".into(), description: "Print current page".into(), category: "Actions".into() },
        Shortcut { keys: "Ctrl+S".into(), description: "Save".into(), category: "Actions".into() },
        Shortcut { keys: "Escape".into(), description: "Close modal / Go back".into(), category: "Navigation".into() },
        Shortcut { keys: "G then D".into(), description: "Go to Dashboard".into(), category: "Navigation".into() },
        Shortcut { keys: "G then I".into(), description: "Go to Inventory".into(), category: "Navigation".into() },
        Shortcut { keys: "G then S".into(), description: "Go to Sales".into(), category: "Navigation".into() },
        Shortcut { keys: "G then P".into(), description: "Go to Purchases".into(), category: "Navigation".into() },
        Shortcut { keys: "1-9".into(), description: "Navigate sidebar sections".into(), category: "Navigation".into() },
    ]
}

#[component]
pub fn ShortcutsHelp() -> Element {
    let mut show_help = use_context::<Signal<bool>>();

    let show = *show_help.read();
    if !show {
        return rsx! {
            div {
                style: "position: fixed; top: 0; left: 0; width: 0; height: 0; overflow: hidden;",
                tabindex: "0",
                onkeydown: move |e: Event<KeyboardData>| {
                    use dioxus::prelude::keyboard_types::Key;
                    let key = e.key();
                    let mods = e.data().modifiers();
                    let ctrl = mods.ctrl() || mods.meta();
                    let is_q = matches!(&key, Key::Character(s) if s == "?");
                    let is_k = matches!(&key, Key::Character(s) if s == "k");
                    let is_esc = matches!(&key, Key::Escape);
                    if is_q || (ctrl && is_k) {
                        show_help.set(true);
                    }
                },
            }
        };
    }

    let all_shortcuts = default_shortcuts();
    let categories: Vec<String> = all_shortcuts
        .iter()
        .map(|s| s.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    rsx! {
        div {
            style: "position: fixed; inset: 0; z-index: 9999; display: flex; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);",
            onclick: move |_| show_help.set(false),
            onkeydown: move |e: Event<KeyboardData>| {
                use dioxus::prelude::keyboard_types::Key;
                if matches!(&e.key(), Key::Escape) {
                    show_help.set(false);
                }
                e.stop_propagation();
            },
            div {
                style: "background: #fff; border-radius: 12px; padding: 24px; max-width: 600px; width: 90%; max-height: 80vh; overflow-y: auto; box-shadow: 0 20px 60px rgba(0,0,0,0.3);",
                onclick: |e| e.stop_propagation(),
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    h2 { style: "margin: 0; font-size: 18px; font-weight: 700; color: #1a1a1a;", "Keyboard Shortcuts" }
                    button {
                        style: "background: none; border: none; font-size: 20px; cursor: pointer; color: #6c757d; padding: 4px 8px;",
                        onclick: move |_| show_help.set(false),
                        "✕"
                    }
                }
                {categories.iter().map(|cat| {
                    let cat_shortcuts: Vec<_> = all_shortcuts.iter().filter(|s| s.category == *cat).collect();
                    rsx! {
                        div { style: "margin-bottom: 16px;",
                            h3 { style: "font-size: 12px; font-weight: 600; color: #6c757d; text-transform: uppercase; letter-spacing: 0.5px; margin: 0 0 8px 0;", "{cat}" }
                            {cat_shortcuts.iter().map(|s| {
                                rsx! {
                                    div { style: "display: flex; justify-content: space-between; align-items: center; padding: 6px 0; border-bottom: 1px solid #f0f0f0;",
                                        span { style: "font-size: 13px; color: #333;", "{s.description}" }
                                        kbd { style: "background: #f5f5f5; border: 1px solid #ddd; border-radius: 4px; padding: 2px 8px; font-size: 12px; font-family: 'SF Mono', Monaco, Consolas, monospace; color: #333;", "{s.keys}" }
                                    }
                                }
                            })}
                        }
                    }
                })}
                div { style: "margin-top: 16px; padding-top: 12px; border-top: 1px solid #eee; font-size: 11px; color: #999; text-align: center;",
                    "Press ? or Escape to toggle"
                }
            }
        }
    }
}
