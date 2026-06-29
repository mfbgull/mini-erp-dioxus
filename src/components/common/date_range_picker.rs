//! Date Range Picker component.
//!
//! Two date inputs (start, end) with range validation. Fires `on_change`
//! with `(Option<NaiveDate>, Option<NaiveDate>)`.

use chrono::NaiveDate;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    pub on_change: EventHandler<(Option<NaiveDate>, Option<NaiveDate>)>,
    #[props(default = "From".to_string())]
    pub start_label: String,
    #[props(default = "To".to_string())]
    pub end_label: String,
    #[props(default = "YYYY-MM-DD".to_string())]
    pub start_placeholder: String,
    #[props(default = "YYYY-MM-DD".to_string())]
    pub end_placeholder: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
}

fn date_to_string(d: Option<NaiveDate>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
}

fn parse_date_input(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() { return None; }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Date range picker component.
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let start_str = use_signal(|| date_to_string(props.start));
    let end_str = use_signal(|| date_to_string(props.end));
    let error = use_signal(|| Option::<String>::None);

    // Sync external prop changes into local string signals only when they differ.
    // The comparison guard prevents an infinite re-render loop: Dioxus signals mark the
    // component dirty even when written with the same value, so we check first.
    {
        let mut s = start_str.clone();
        let start = props.start;
        use_effect(move || {
            let new_str = date_to_string(start);
            if *s.read() != new_str {
                s.set(new_str);
            }
        });
    }
    {
        let mut e = end_str.clone();
        let end = props.end;
        use_effect(move || {
            let new_str = date_to_string(end);
            if *e.read() != new_str {
                e.set(new_str);
            }
        });
    }

    let on_change = props.on_change.clone();
    let start_str_c = start_str.clone();
    let end_str_c = end_str.clone();
    let mut error_c = error.clone();

    let validate_and_fire = move || {
        let start = parse_date_input(&start_str_c.read());
        let end = parse_date_input(&end_str_c.read());
        let mut err = None;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                err = Some("End date must be after start date.".to_string());
            }
        }
        error_c.set(err);
        on_change.call((start, end));
    };

    rsx! {
        div {
            class: "cb-daterange {props.class.as_deref().unwrap_or_default()}",
            div { style: "flex: 1; display: flex; flex-direction: column; gap: 2px;",
                label { class: "cb-input-label", "{props.start_label}" }
                input {
                    class: "cb-daterange-input",
                    r#type: "date",
                    value: "{start_str}",
                    placeholder: "{props.start_placeholder}",
                    disabled: props.disabled,
                    oninput: {
                        let mut v = start_str.clone();
                        let mut f = validate_and_fire.clone();
                        move |e| { v.set(e.value()); f(); }
                    },
                }
            }
            span { class: "cb-daterange-separator", "→" }
            div { style: "flex: 1; display: flex; flex-direction: column; gap: 2px;",
                label { class: "cb-input-label", "{props.end_label}" }
                input {
                    class: "cb-daterange-input",
                    r#type: "date",
                    value: "{end_str}",
                    placeholder: "{props.end_placeholder}",
                    disabled: props.disabled,
                    oninput: {
                        let mut v = end_str.clone();
                        let mut f = validate_and_fire.clone();
                        move |e| { v.set(e.value()); f(); }
                    },
                }
            }
            if let Some(err) = error.read().as_ref() {
                div { class: "cb-daterange-error", "⚠ {err}" }
            }
        }
    }
}
