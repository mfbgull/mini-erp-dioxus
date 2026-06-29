//! Searchable Select component — typeahead dropdown with keyboard navigation.
//!
//! Supports single-selection and multi-selection (tag chips).
//! Features search filtering, arrow key navigation, and Enter to select.

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct SearchableSelectProps {
    pub options: Vec<SelectOption>,
    #[props(default)]
    pub selected_value: Option<String>,
    #[props(default)]
    pub selected_values: Vec<String>,
    #[props(default)]
    pub on_select: Option<EventHandler<String>>,
    #[props(default)]
    pub on_select_multi: Option<EventHandler<Vec<String>>>,
    #[props(default = "Select…".to_string())]
    pub placeholder: String,
    #[props(default = false)]
    pub multi: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = true)]
    pub searchable: bool,
    #[props(default)]
    pub class: Option<String>,
}

pub fn SearchableSelect(props: SearchableSelectProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut search_text = use_signal(String::new);
    let mut highlighted_idx = use_signal(|| 0usize);

    // Clone handlers before closures
    let on_select = props.on_select.clone();
    let on_select_multi = props.on_select_multi.clone();
    let multi = props.multi;
    let placeholder = props.placeholder.clone();
    let disabled = props.disabled;
    let searchable = props.searchable;
    let options = props.options.clone();
    let selected_value = props.selected_value.clone();
    let selected_values = props.selected_values.clone();

    let filtered = {
        let search = search_text.read().to_lowercase();
        if search.is_empty() {
            options.clone()
        } else {
            options.iter()
                .filter(|o| o.label.to_lowercase().contains(&search))
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    let close = {
        let mut is_open = is_open.clone();
        let mut search_text = search_text.clone();
        move || {
            is_open.set(false);
            search_text.set(String::new());
        }
    };

    let chevron_class = if *is_open.read() { "cb-select-chevron cb-select-chevron-open" } else { "cb-select-chevron" };

    rsx! {
        div {
            class: "cb-select-wrapper {props.class.as_deref().unwrap_or_default()}",
            div {
                class: "cb-select-input",
                onclick: move |e| {
                    e.stop_propagation();
                    if !disabled {
                        let current = *is_open.read();
                        is_open.set(!current);
                    }
                },
                if multi {
                    {selected_values.iter().map(|tag| {
                        let tag_val = tag.clone();
                        let label = options.iter()
                            .find(|o| o.value == *tag)
                            .map(|o| o.label.clone())
                            .unwrap_or_default();
                        let on_remove = {
                            let cb = on_select_multi.clone();
                            let current = selected_values.clone();
                            move |_| {
                                let mut new_vals = current.clone();
                                new_vals.retain(|v| v != &tag_val);
                                if let Some(ref cb) = cb { cb.call(new_vals); }
                            }
                        };
                        rsx! {
                            span { class: "cb-select-tag",
                                "{label}"
                                button {
                                    class: "cb-select-tag-remove",
                                    r#type: "button",
                                    onclick: on_remove,
                                    "×"
                                }
                            }
                        }
                    })}
                    if selected_values.is_empty() {
                        span { class: "cb-select-placeholder", "{placeholder}" }
                    }
                } else {
                    if let Some(label) = selected_value.as_ref().and_then(|v| {
                        options.iter().find(|o| o.value == *v).map(|o| o.label.clone())
                    }) {
                        span { class: "cb-select-value", "{label}" }
                    } else {
                        span { class: "cb-select-placeholder", "{placeholder}" }
                    }
                }
                span {
                    class: "{chevron_class}",
                    "▾"
                }
            }

            if *is_open.read() {
                div {
                    class: "cb-select-dropdown",
                    onclick: move |e| e.stop_propagation(),
                    if searchable {
                        div { class: "cb-select-search",
                            input {
                                r#type: "text",
                                placeholder: "Type to search…",
                                value: "{search_text}",
                                autofocus: "true",
                                oninput: {
                                    let mut st = search_text.clone();
                                    let mut hi = highlighted_idx.clone();
                                    move |e| { st.set(e.value()); hi.set(0); }
                                },
                                onkeydown: {
                                    let f = filtered.clone();
                                    let mut hi = highlighted_idx.clone();
                                    let mut close_fn = close.clone();
                                    let os = on_select.clone();
                                    let osm = on_select_multi.clone();
                                    let multi = multi;
                                    let sv = selected_values.clone();
                                    move |e| {
                                        match e.key() {
                                            Key::ArrowDown => {
                                                let len = f.len();
                                                if len > 0 {
                                                    let current = *hi.read();
                                                    hi.set((current + 1).min(len - 1));
                                                }
                                            }
                                            Key::ArrowUp => {
                                                if *hi.read() > 0 {
                                                    let current = *hi.read();
                                                    hi.set(current - 1);
                                                }
                                            }
                                            Key::Enter => {
                                                if !f.is_empty() {
                                                    let idx = *hi.read();
                                                    if idx < f.len() {
                                                        let opt = &f[idx];
                                                        if multi {
                                                            let mut current = sv.clone();
                                                            if !current.contains(&opt.value) { current.push(opt.value.clone()); }
                                                            if let Some(ref cb) = osm { cb.call(current); }
                                                        } else {
                                                            if let Some(ref cb) = os { cb.call(opt.value.clone()); }
                                                            close_fn();
                                                        }
                                                    }
                                                }
                                            }
                                            Key::Escape => { close_fn(); }
                                            _ => {}
                                        }
                                    }
                                },
                            }
                        }
                    }
                    div { class: "cb-select-options",
                        if filtered.is_empty() {
                            div { class: "cb-select-no-results", "No results found" }
                        } else {
                            {filtered.iter().enumerate().map(|(idx, opt)| {
                                let is_selected = if multi {
                                    selected_values.contains(&opt.value)
                                } else {
                                    selected_value.as_ref().map_or(false, |v| *v == opt.value)
                                };
                                let is_highlighted = idx == *highlighted_idx.read();
                                let mut opt_classes = vec!["cb-select-option"];
                                if is_highlighted { opt_classes.push("cb-select-option-highlighted"); }
                                if is_selected { opt_classes.push("cb-select-option-selected"); }

                                let opt_val = opt.value.clone();
                                let opt_label = opt.label.clone();
                                let mut close_fn = close.clone();
                                let os = on_select.clone();
                                let osm = on_select_multi.clone();
                                let multi = multi;
                                let sv = selected_values.clone();

                                rsx! {
                                    div {
                                        class: "{opt_classes.join(\" \")}",
                                        onclick: move |_| {
                                            if multi {
                                                let mut current = sv.clone();
                                                if !current.contains(&opt_val) { current.push(opt_val.clone()); }
                                                if let Some(ref cb) = osm { cb.call(current); }
                                            } else {
                                                if let Some(ref cb) = os { cb.call(opt_val.clone()); }
                                                close_fn();
                                            }
                                        },
                                        if is_selected { span { "✓ " } }
                                        "{opt_label}"
                                    }
                                }
                            })}
                        }
                    }
                }
            }

            // Click-away overlay (closes dropdown when clicking outside)
            if *is_open.read() {
                div {
                    style: "position: fixed; inset: 0; z-index: 499;",
                    onclick: move |_| {
                        is_open.set(false);
                        search_text.set(String::new());
                    },
                }
            }
        }
    }
}
