//! Form Input component — labeled input with validation error display.
//!
//! Supports text, number, email, password, textarea, and select field types.
//! Includes required indicator, hint text, icon prefix, and error message.

use dioxus::prelude::*;

/// The HTML input type to render.
#[derive(Clone, PartialEq, Debug)]
pub enum InputType {
    Text,
    Number,
    Email,
    Password,
    Date,
    Tel,
    Url,
    TextArea,
}

impl Default for InputType {
    fn default() -> Self {
        Self::Text
    }
}

impl InputType {
    fn html_type(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Email => "email",
            Self::Password => "password",
            Self::Date => "date",
            Self::Tel => "tel",
            Self::Url => "url",
            Self::TextArea => "text",
        }
    }
}

/// Form input component properties.
#[derive(Props, Clone, PartialEq)]
pub struct FormInputProps {
    #[props(default)]
    pub label: Option<String>,
    pub value: String,
    pub oninput: EventHandler<String>,
    #[props(default = InputType::Text)]
    pub r#type: InputType,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub error: Option<String>,
    #[props(default)]
    pub hint: Option<String>,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default)]
    pub icon: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub min: Option<f64>,
    #[props(default)]
    pub max: Option<f64>,
    #[props(default)]
    pub step: Option<f64>,
    #[props(default)]
    pub max_length: Option<usize>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub autocomplete: Option<String>,
}

/// Form input component with label, validation, and error display.
pub fn FormInput(props: FormInputProps) -> Element {
    let has_error = props.error.as_ref().map_or(false, |e| !e.is_empty());
    let is_textarea = props.r#type == InputType::TextArea;

    let input_class = format!(
        "cb-input{} {}",
        if is_textarea { " cb-input-textarea" } else { "" },
        if has_error { "cb-input-error" } else { "" },
    );

    // Clone handlers before closures to avoid capturing entire props
    let oninput = props.oninput.clone();
    let max_length = props.max_length;

    rsx! {
        div {
            class: "cb-input-group",
            if let Some(label) = &props.label {
                label { class: "cb-input-label",
                    "{label}"
                    if props.required {
                        span { class: "cb-input-required", " *" }
                    }
                }
            }
            div { class: "{input_class}",
                if let Some(icon) = &props.icon {
                    span { class: "cb-input-icon", "{icon}" }
                }
                if is_textarea {
                    textarea {
                        placeholder: props.placeholder.as_deref().unwrap_or(""),
                        value: "{props.value}",
                        disabled: props.disabled,
                        readonly: props.read_only,
                        maxlength: max_length.map(|v| v as i32).unwrap_or(-1),
                        oninput: move |e| {
                            let v = e.value();
                            if let Some(m) = max_length {
                                if v.len() > m { return; }
                            }
                            oninput.call(v);
                        },
                    }
                } else {
                    input {
                        r#type: props.r#type.html_type(),
                        placeholder: props.placeholder.as_deref().unwrap_or(""),
                        value: "{props.value}",
                        disabled: props.disabled,
                        readonly: props.read_only,
                        min: props.min.map(|v| v.to_string()).as_deref().unwrap_or(""),
                        max: props.max.map(|v| v.to_string()).as_deref().unwrap_or(""),
                        step: props.step.map(|v| v.to_string()).as_deref().unwrap_or(""),
                        maxlength: max_length.map(|v| v as i32).unwrap_or(-1),
                        name: props.name.as_deref().unwrap_or(""),
                        autocomplete: props.autocomplete.as_deref().unwrap_or(""),
                        oninput: move |e| {
                            let v = e.value();
                            if let Some(m) = max_length {
                                if v.len() > m { return; }
                            }
                            oninput.call(v);
                        },
                    }
                }
            }
            if let Some(hint) = &props.hint {
                if !has_error {
                    span { class: "cb-input-hint", "{hint}" }
                }
            }
            if let Some(err) = &props.error {
                if !err.is_empty() {
                    span { class: "cb-input-error-text", "⚠ {err}" }
                }
            }
        }
    }
}
