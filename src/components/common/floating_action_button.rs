use dioxus::prelude::*;

const FAB_CSS: &str = r#"
.fab { position: fixed; bottom: 24px; right: 24px; width: 56px; height: 56px; border-radius: 16px; background: var(--accent, #4a90d9); color: #fff; border: none; font-size: 24px; cursor: pointer; box-shadow: 0 4px 12px rgba(0,0,0,0.2); z-index: 100; display: flex; align-items: center; justify-content: center; transition: transform 0.2s, box-shadow 0.2s; }
.fab:hover { transform: scale(1.05); box-shadow: 0 6px 16px rgba(0,0,0,0.25); }
.fab:active { transform: scale(0.95); }
.fab-speed-dial { position: fixed; bottom: 90px; right: 24px; display: flex; flex-direction: column-reverse; gap: 10px; z-index: 100; }
.fab-speed-dial-item { display: flex; align-items: center; gap: 8px; justify-content: flex-end; }
.fab-speed-dial-label { padding: 4px 10px; background: #333; color: #fff; border-radius: 6px; font-size: 12px; white-space: nowrap; }
.fab-speed-dial-btn { width: 44px; height: 44px; border-radius: 12px; background: var(--accent, #4a90d9); color: #fff; border: none; font-size: 18px; cursor: pointer; box-shadow: 0 2px 8px rgba(0,0,0,0.15); display: flex; align-items: center; justify-content: center; }
"#;

#[derive(Clone, PartialEq)]
pub struct FabAction {
    pub icon: String,
    pub label: String,
    pub onclick: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct FloatingActionButtonProps {
    #[props(default = "+".to_string())]
    icon: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)]
    actions: Vec<FabAction>,
}

#[component]
pub fn FloatingActionButton(props: FloatingActionButtonProps) -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        style { "{FAB_CSS}" }
        if !props.actions.is_empty() && *open.read() {
            div { class: "fab-speed-dial",
                for action in props.actions.iter() {
                    div { class: "fab-speed-dial-item",
                        span { class: "fab-speed-dial-label", "{action.label}" }
                        button { class: "fab-speed-dial-btn", "{action.icon}" }
                    }
                }
            }
        }
        button {
            class: "fab",
            onclick: move |e| {
                if props.actions.is_empty() {
                    props.onclick.call(e);
                } else {
                    open.toggle();
                }
            },
            if *open.read() { "✕" } else { "{props.icon}" }
        }
    }
}
