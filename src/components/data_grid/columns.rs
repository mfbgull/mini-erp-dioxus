//! Column header rendering with Phase 2 column filtering, width calculation,
//! sort indicators, and filter dropdowns.
//!
//! Phase 1: header text, sort indicators, width calc.
//! Phase 2: filter button, filter dropdown with input widgets, active filter indicator.

use dioxus::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

use super::filter::FilterValue;
use super::types::*;

// ---------------------------------------------------------------------------
// Resize Handle Constants
// ---------------------------------------------------------------------------

/// The width of the invisible hit area for the resize handle.
const RESIZE_HANDLE_WIDTH: f64 = 6.0;

// ---------------------------------------------------------------------------
// Width Calculation
// ---------------------------------------------------------------------------

/// Compute the final per-column pixel widths from column definitions and
/// the available container width.
///
/// Algorithm:
/// 1. Fixed (`Px`) columns get their pixel value.
/// 2. `Auto` columns get a sample-based width (header text + sample values).
/// 3. Remaining space after fixed + auto is distributed among `Fr` columns
///    proportional to their `fr` value.
pub fn calculate_column_widths<T>(
    columns: &[ColumnDef<T>],
    container_width: f64,
    sample_rows: Option<&[String]>,
) -> Vec<f64> {
    let mut widths: Vec<f64> = vec![0.0; columns.len()];
    let mut used: f64 = 0.0;
    let mut fr_total: f32 = 0.0;

    // Pass 1: collect widths for non-fr columns
    for (i, col) in columns.iter().enumerate() {
        match col.width {
            ColumnWidth::Px(px) => {
                let w = px as f64;
                widths[i] = w;
                used += w;
            }
            ColumnWidth::Auto => {
                let w = estimate_auto_width(col.header, sample_rows) as f64;
                widths[i] = w.clamp(60.0, 400.0);
                used += widths[i];
            }
            ColumnWidth::Fr(fr) => {
                fr_total += fr;
            }
        }
    }

    // Pass 2: distribute remaining space to Fr columns
    if fr_total > 0.0 {
        let remaining = (container_width - used).max(100.0);
        for (i, col) in columns.iter().enumerate() {
            if let ColumnWidth::Fr(fr) = col.width {
                let share = (fr as f64 / fr_total as f64) * remaining;
                widths[i] = share.max(60.0);
            }
        }
    }

    widths
}

fn estimate_auto_width(header: &str, _sample_rows: Option<&[String]>) -> u32 {
    let char_width = 7.5;
    let header_width = (header.len() as f64 * char_width).ceil() as u32;
    header_width + 32
}

// ---------------------------------------------------------------------------
// Sort Indicator
// ---------------------------------------------------------------------------

pub fn render_sort_indicator(direction: Option<SortDirection>) -> Element {
    match direction {
        Some(SortDirection::Ascending) => rsx! {
            span { class: "dg-sort-indicator", " ▲" }
        },
        Some(SortDirection::Descending) => rsx! {
            span { class: "dg-sort-indicator", " ▼" }
        },
        None => rsx! {
            span { class: "dg-sort-indicator dg-sort-inactive", " ⇅" }
        },
    }
}

// ---------------------------------------------------------------------------
// Filter Button / Indicator
// ---------------------------------------------------------------------------

/// Render the filter button that appears next to the header text.
/// Shows a funnel icon with active filter indicator dot.
fn render_filter_button(
    is_active: bool,
    is_open: bool,
) -> Element {
    let btn_class = format!(
        "dg-filter-btn {} {}",
        if is_active { "dg-filter-active" } else { "" },
        if is_open { "dg-filter-open" } else { "" },
    );

    rsx! {
        button {
            class: "{btn_class}",
            r#type: "button",
            tabindex: 0,
            aria_label: if is_active { "Filter active, click to change" } else { "Open filter" },
            aria_expanded: if is_open { "true" } else { "false" },
            aria_haspopup: "true",

            // Funnel icon (simple SVG)
            svg {
                width: "14",
                height: "14",
                view_box: "0 0 16 16",
                fill: "currentColor",
                path {
                    d: "M1.5 1.5A.5.5 0 0 1 2 1h12a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-.146.354L8 10.293V13.5a.5.5 0 0 1-.5.5h-1a.5.5 0 0 1-.5-.5v-3.207L1.646 3.854A.5.5 0 0 1 1.5 3.5v-2z"
                }
            }

            // Active indicator dot
            if is_active {
                span { class: "dg-filter-dot" }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Filter Dropdown
// ---------------------------------------------------------------------------

/// Render the dropdown content for a single column's filter.
fn render_filter_dropdown(
    col_key: &'static str,
    filter_type: &FilterType,
    filter_value: &FilterValue,
    on_filter_change: Rc<dyn Fn(&'static str, FilterValue)>,
) -> Element {
    let col_key_clone = col_key;

    match filter_type {
        FilterType::None => rsx! {
            div { class: "dg-filter-dropdown", "No filter available" }
        },

        FilterType::Text => {
            let current = match filter_value {
                FilterValue::Text { query } => query.clone(),
                _ => String::new(),
            };

            rsx! {
                div {
                    class: "dg-filter-dropdown",
                    role: "dialog",
                    aria_label: "Text filter for {col_key}",

                    input {
                        class: "dg-filter-input",
                        r#type: "text",
                        placeholder: "Filter…",
                        value: "{current}",
                        autofocus: true,
                        oninput: {
                            let key = col_key_clone;
                            let cb = on_filter_change.clone();
                            move |e| {
                                cb(key, FilterValue::Text { query: e.value() });
                            }
                        },
                        onkeydown: {
                            let key = col_key_clone;
                            let cb = on_filter_change.clone();
                            move |e| {
                                if e.key() == dioxus::prelude::Key::Escape {
                                    // Close handled by parent
                                }
                                if e.key() == dioxus::prelude::Key::Enter {
                                    // Keep focus, filter updates live
                                }
                            }
                        },
                    }

                    if !current.is_empty() {
                        button {
                            class: "dg-filter-clear-btn",
                            r#type: "button",
                            onclick: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                move |_| {
                                    cb(key, FilterValue::None);
                                }
                            },
                            "Clear"
                        }
                    }
                }
            }
        },

        FilterType::Number => {
            let (current_min, current_max) = match filter_value {
                FilterValue::Number { min, max } => (
                    min.map(|v| v.to_string()).unwrap_or_default(),
                    max.map(|v| v.to_string()).unwrap_or_default(),
                ),
                _ => (String::new(), String::new()),
            };

            rsx! {
                div {
                    class: "dg-filter-dropdown",
                    role: "dialog",
                    aria_label: "Number filter for {col_key}",

                    div { class: "dg-filter-range",
                        label { class: "dg-filter-label", "Min" }
                        input {
                            class: "dg-filter-input",
                            r#type: "number",
                            placeholder: "Min",
                            value: "{current_min}",
                            oninput: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                let max = current_max.clone();
                                move |e| {
                                    let min = e.value().parse::<f64>().ok();
                                    let max = max.parse::<f64>().ok();
                                    cb(key, FilterValue::Number { min, max });
                                }
                            },
                        }
                    }

                    div { class: "dg-filter-range",
                        label { class: "dg-filter-label", "Max" }
                        input {
                            class: "dg-filter-input",
                            r#type: "number",
                            placeholder: "Max",
                            value: "{current_max}",
                            oninput: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                let min = current_min.clone();
                                move |e| {
                                    let min = min.parse::<f64>().ok();
                                    let max = e.value().parse::<f64>().ok();
                                    cb(key, FilterValue::Number { min, max });
                                }
                            },
                        }
                    }

                    // Show clear button when either min or max has a value
                    if !current_min.is_empty() || !current_max.is_empty() {
                        button {
                            class: "dg-filter-clear-btn",
                            r#type: "button",
                            onclick: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                move |_| {
                                    cb(key, FilterValue::None);
                                }
                            },
                            "Clear"
                        }
                    }
                }
            }
        },

        FilterType::Date => {
            let (current_from, current_to) = match filter_value {
                FilterValue::Date { from, to } => (
                    from.clone().unwrap_or_default(),
                    to.clone().unwrap_or_default(),
                ),
                _ => (String::new(), String::new()),
            };

            rsx! {
                div {
                    class: "dg-filter-dropdown",
                    role: "dialog",
                    aria_label: "Date filter for {col_key}",

                    div { class: "dg-filter-range",
                        label { class: "dg-filter-label", "From" }
                        input {
                            class: "dg-filter-input",
                            r#type: "date",
                            value: "{current_from}",
                            oninput: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                let to = current_to.clone();
                                move |e| {
                                    let from_val = e.value();
                                    let from = if from_val.is_empty() { None } else { Some(from_val) };
                                    let to = if to.is_empty() { None } else { Some(to.clone()) };
                                    cb(key, FilterValue::Date { from, to });
                                }
                            },
                        }
                    }

                    div { class: "dg-filter-range",
                        label { class: "dg-filter-label", "To" }
                        input {
                            class: "dg-filter-input",
                            r#type: "date",
                            value: "{current_to}",
                            oninput: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                let from = current_from.clone();
                                move |e| {
                                    let to_val = e.value();
                                    let from = if from.is_empty() { None } else { Some(from.clone()) };
                                    let to = if to_val.is_empty() { None } else { Some(to_val) };
                                    cb(key, FilterValue::Date { from, to });
                                }
                            },
                        }
                    }

                    if !current_from.is_empty() || !current_to.is_empty() {
                        button {
                            class: "dg-filter-clear-btn",
                            r#type: "button",
                            onclick: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                move |_| {
                                    cb(key, FilterValue::None);
                                }
                            },
                            "Clear"
                        }
                    }
                }
            }
        },

        FilterType::Select { options } => {
            let selected = match filter_value {
                FilterValue::Select { selected } => selected.clone(),
                _ => Vec::new(),
            };

            rsx! {
                div {
                    class: "dg-filter-dropdown dg-filter-select-dropdown",
                    role: "dialog",
                    aria_label: "Select filter for {col_key}",

                    // Select all / Deselect all
                    div { class: "dg-filter-select-actions",
                        button {
                            class: "dg-filter-select-action",
                            r#type: "button",
                            onclick: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                let opts = options.clone();
                                move |_| {
                                    cb(key, FilterValue::Select { selected: opts.clone() });
                                }
                            },
                            "Select All"
                        }
                        button {
                            class: "dg-filter-select-action",
                            r#type: "button",
                            onclick: {
                                let key = col_key_clone;
                                let cb = on_filter_change.clone();
                                move |_| {
                                    cb(key, FilterValue::Select { selected: vec![] });
                                }
                            },
                            "Clear"
                        }
                    }

                    // Checkbox list
                    div { class: "dg-filter-select-options",
                        {options.iter().map(|opt| {
                            let is_checked = selected.contains(opt);
                            let opt_str = opt.clone();
                            let key = col_key_clone;
                            let cb = on_filter_change.clone();
                            let sel = selected.clone();

                            rsx! {
                                label {
                                    class: "dg-filter-select-option",
                                    input {
                                        r#type: "checkbox",
                                        checked: is_checked,
                                        oninput: {
                                            let key = key;
                                            let cb = cb.clone();
                                            let opt = opt_str.clone();
                                            let mut sel = sel.clone();
                                            move |_| {
                                                if sel.contains(&opt) {
                                                    sel.retain(|s| s != &opt);
                                                } else {
                                                    sel.push(opt.clone());
                                                }
                                                cb(key, FilterValue::Select { selected: sel.clone() });
                                            }
                                        },
                                    }
                                    span { "{opt_str}" }
                                }
                            }
                        })}
                    }
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Resize Handle
// ---------------------------------------------------------------------------

/// Render a drag handle for column resizing.
///
/// The handle is a thin vertical bar at the right edge of the header cell.
/// It's invisible until hovered (CSS controlled).
/// `on_resize_start` is called on mousedown with (col_key, client_x).
pub fn render_resize_handle(
    col_key: &'static str,
    on_resize_start: Rc<dyn Fn(&'static str, f64)>,
) -> Element {
    rsx! {
        div {
            class: "dg-resize-handle",
            "data-col": "{col_key}",
            role: "separator",
            aria_label: "Resize column",
            tabindex: 0,

            onmousedown: move |e| {
                e.prevent_default();
                e.stop_propagation();
                on_resize_start(col_key, e.data().client_coordinates().x);
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Full Header Row
// ---------------------------------------------------------------------------

/// Render the complete header row including column headers, sort controls,
/// filter buttons, and filter dropdowns.
pub fn render_header_row<T: 'static + Clone>(
    columns: &[ColumnDef<T>],
    widths: &[f64],
    active_sort: &[SortColumn],
    on_sort: Rc<dyn Fn(&'static str)>,
    // Filter state
    active_filters: &HashMap<&'static str, FilterValue>,
    open_filter_col: Option<&'static str>,
    on_toggle_filter: Rc<dyn Fn(&'static str)>,
    on_filter_change: Rc<dyn Fn(&'static str, FilterValue)>,
    // Phase 3: Column resize
    on_resize_start: Option<Rc<dyn Fn(&'static str, f64)>>,
) -> Element {
    let columns_clone = columns.to_vec();
    let widths_clone = widths.to_vec();
    let sort_clone = active_sort.to_vec();
    let filters_clone = active_filters.clone();
    let open_col = open_filter_col.map(|s| s);

    rsx! {
        div {
            class: "dg-header-row",
            role: "row",
            aria_label: "Column headers",

            {columns_clone.into_iter().enumerate().map(|(i, col)| {
                let key = col.key;
                let is_sorted = sort_clone.iter().find(|s| s.key == key);
                let direction = is_sorted.map(|s| s.direction);
                let width = widths_clone.get(i).copied().unwrap_or(120.0);
                let on_sort = on_sort.clone();
                let has_filter = col.filter_type != FilterType::None;
                let filter_val = filters_clone.get(key).cloned().unwrap_or(FilterValue::None);
                let is_filter_active = filter_val.is_active();
                let is_filter_open = open_col == Some(key);
                let on_toggle = on_toggle_filter.clone();
                let on_filter_change = on_filter_change.clone();
                let filter_type = col.filter_type.clone();
                let is_resizable = col.resizable;
                let pinned_cls = match col.pinned {
                    Some(PinnedPosition::Left) => " dg-pinned-left",
                    Some(PinnedPosition::Right) => " dg-pinned-right",
                    None => "",
                };

                rsx! {
                    div {
                        key: "hdr-{key}",
                        class: format!(
                            "dg-header-cell dg-align-{} {} {} {}",
                            match col.align {
                                TextAlign::Left => "left",
                                TextAlign::Center => "center",
                                TextAlign::Right => "right",
                            },
                            if col.sortable { "dg-sortable" } else { "" },
                            if is_filter_active { "dg-has-active-filter" } else { "" },
                            if is_resizable { "dg-resizable" } else { "" },
                        ),
                        role: "columnheader",
                        style: "width: {width}px;",
                        aria_sort: match direction {
                            Some(SortDirection::Ascending) => "ascending",
                            Some(SortDirection::Descending) => "descending",
                            None => "none",
                        },
                        aria_label: "{col.header}",
                        tabindex: if col.sortable { 0 } else { -1 },

                        onclick: {
                            let key = col.key;
                            let on_sort = on_sort.clone();
                            move |_| {
                                if col.sortable {
                                    on_sort(key);
                                }
                            }
                        },
                        onkeydown: {
                            let key = col.key;
                            let on_sort = on_sort.clone();
                            move |e| {
                                if e.key() == dioxus::prelude::Key::Enter || e.key() == dioxus::prelude::Key::Character(" ".to_string()) {
                                    if col.sortable {
                                        on_sort(key);
                                    }
                                }
                            }
                        },

                        // Header text
                        span { class: "dg-header-text", "{col.header}" }
                        {render_sort_indicator(direction)}

                        // Filter button (if filterable)
                        if has_filter {
                            div {
                                class: "dg-filter-btn-wrapper",
                                onclick: |e| e.stop_propagation(),
                                {render_filter_button(is_filter_active, is_filter_open)}
                            }
                        }

                        // Resize handle (if resizable)
                        if is_resizable {
                            if let Some(ref resize_cb) = on_resize_start {
                                {render_resize_handle(col.key, resize_cb.clone())}
                            }
                        }

                        // Filter dropdown (if this column's filter is open)
                        if is_filter_open {
                            div {
                                class: "dg-filter-dropdown-wrapper",
                                onclick: |e| e.stop_propagation(),
                                {render_filter_dropdown(
                                    key,
                                    &filter_type,
                                    &filter_val,
                                    on_filter_change,
                                )}
                            }
                        }
                    }
                }
            })}
        }
    }
}
