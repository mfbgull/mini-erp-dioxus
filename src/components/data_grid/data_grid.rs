//! Main `DataGrid<T>` component — the primary entry point for the data grid.
//!
//! # Phase 1 Features
//! - Generic over row type `T: Clone + PartialEq + 'static`
//! - Column definitions with type-safe value extraction via closures
//! - Client-side multi-column sorting
//! - Client-side pagination with configurable page size
//! - Row selection (single / multi), row click callback
//! - Loading spinner, skeleton rows, empty state
//! - Seven cell renderers: text, number, currency, date, datetime, badge, percentage
//! - Dynamic cell CSS class rules, row striping/hover
//! - ARIA accessibility, keyboard navigation
//!
//! # Phase 2 Features
//! - Column-level filtering: text substring, number range, date range, select multi
//! - Filter dropdown with input widgets per column header
//! - Active filter indicator dots on filter buttons
//! - Data pipeline: filter → sort → paginate
//! - External `filter_state` prop for controlled filter state
//!
//! # Phase 3 Features (new)
//! - Virtual scrolling: only renders visible rows + buffer for large datasets
//! - Column resize: draggable handle on resizable column headers
//! - Pinned (frozen) columns: CSS sticky positioning for left/right columns

use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::columns::{calculate_column_widths, render_header_row};
use super::filter::{apply_filters, count_active_filters, FilterValue};
use super::pagination::{PageState, PaginationBar};
use super::renderers::render_cell;
use super::types::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Extra rows rendered above/below the visible viewport for smooth scrolling.
const VIRTUAL_SCROLL_BUFFER: usize = 5;

/// Default height for the virtual scroll container when not constrained.
const DEFAULT_CONTAINER_HEIGHT: f64 = 600.0;

/// Minimum column width in pixels (resize constraint).
const MIN_COLUMN_WIDTH: f64 = 40.0;

/// Maximum column width in pixels (resize constraint).
const MAX_COLUMN_WIDTH: f64 = 800.0;

/// The width of the checkbox selection column.
const CHECKBOX_COL_WIDTH: f64 = 40.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn DataGrid<T: 'static + Clone + PartialEq>(
    /// Column definitions (order determines display order).
    columns: Vec<ColumnDef<T>>,

    /// Row data. All rows are passed for client-side mode.
    rows: Vec<T>,

    /// Optional callback when a row is clicked (index, data).
    on_row_click: Option<EventHandler<(usize, T)>>,

    /// Optional callback when a row is double-clicked.
    on_row_double_click: Option<EventHandler<(usize, T)>>,

    /// Show loading spinner overlay.
    #[props(default = false)]
    loading: bool,

    /// Show skeleton loading rows (before data is available).
    #[props(default = false)]
    skeleton: bool,

    /// Number of skeleton rows to show.
    #[props(default = 10)]
    skeleton_rows: usize,

    /// Message to display when rows are empty.
    #[props(default = "No data".to_string())]
    empty_message: String,

    /// Optional hint text below the empty message.
    empty_hint: Option<String>,

    /// Pagination mode (default: client-side with 50 rows per page).
    #[props(default)]
    pagination: PaginationMode,

    /// Row selection mode.
    #[props(default)]
    selection_mode: SelectionMode,

    /// Row height preset.
    #[props(default)]
    row_height: RowHeight,

    /// Enable alternating row background colors (default: true).
    #[props(default = true)]
    striped: bool,

    /// Enable row highlight on hover (default: true).
    #[props(default = true)]
    hoverable: bool,

    /// Optional external sort state.
    sort_state: Option<Signal<Vec<SortColumn>>>,

    /// Optional external selection state.
    selected_rows: Option<Signal<HashSet<usize>>>,

    // ── Phase 2: Filter props ──

    /// Optional external filter state (column key → filter value).
    filter_state: Option<Signal<HashMap<&'static str, FilterValue>>>,

    /// Optional callback fired whenever a filter changes.
    on_filter_change: Option<EventHandler<(&'static str, FilterValue)>>,

    // ── Phase 3: Virtual scroll + column resize + pinned columns ──

    /// Enable virtual scrolling for large datasets.
    /// When true and PaginationMode::None, only visible rows + buffer are
    /// rendered in the DOM.
    #[props(default = false)]
    virtual_scroll: bool,

    /// Height of the virtual scroll container in pixels.
    /// Ignored when virtual_scroll is false.
    #[props(default = DEFAULT_CONTAINER_HEIGHT)]
    virtual_scroll_height: f64,

    /// Called when a column is resized by the user (column key, new width).
    on_column_resize: Option<EventHandler<(&'static str, f64)>>,

    // ── Phase 4: Inline cell editing ──

    /// Called when a cell value is edited and committed.
    /// Parameters: (row_index, column_key, old_value, new_value).
    /// The DataGrid does NOT update `rows` — the parent must handle
    /// this callback and update its data source accordingly.
    on_cell_edit: Option<EventHandler<(usize, &'static str, String, String)>>,

    // ── Phase 5: Server-side mode ──

    /// Total records count from the server (required when using
    /// `PaginationMode::Server`). Ignored in Client/None modes.
    #[props(default = 0)]
    total_records: usize,

    /// Called when the sort changes in server-side mode.
    /// The parent should re-fetch data with the new sort params.
    /// Ignored in client-side mode.
    on_sort_change: Option<EventHandler<(&'static str, SortDirection)>>,

    /// Called when the page changes in server-side mode.
    /// The parent should re-fetch the given page.
    /// Overrides the internal page change handler when provided.
    on_server_page_change: Option<EventHandler<usize>>,

    /// Called when the page size changes in server-side mode.
    /// The parent should re-fetch with the new page size.
    on_server_page_size_change: Option<EventHandler<usize>>,

    // ── Phase 5: Column visibility toggle ──

    /// Show the column visibility toggle menu button in the toolbar.
    /// When enabled, users can show/hide columns by key from a dropdown.
    #[props(default = true)]
    column_toggle: bool,
) -> Element
where
    T: 'static + Clone + PartialEq,
{
    // ---- Internal state ----

    // Sorting
    let internal_sort = use_signal(|| Vec::<SortColumn>::new());
    let sort = sort_state.as_ref().unwrap_or(&internal_sort);

    // Pagination — use total_records for server mode, rows.len() for client mode
    let is_server = pagination.is_server();
    let total_items = if is_server { total_records } else { rows.len() };
    let page_size = match pagination {
        PaginationMode::None => rows.len(),
        PaginationMode::Client { page_size } | PaginationMode::Server { page_size } => page_size,
    };
    let page_state = use_signal(|| PageState::new(page_size.max(1), total_items));

    // Selection
    let internal_selection = use_signal(|| HashSet::<usize>::new());
    let selection = selected_rows.as_ref().unwrap_or(&internal_selection);

    // Container width
    let container_width = use_signal(|| 800.0_f64);

    // ── Phase 2: Filter state ──

    let internal_filters = use_signal(|| {
        let mut map: HashMap<&'static str, FilterValue> = HashMap::new();
        for col in &columns {
            if col.filter_type != FilterType::None {
                map.insert(col.key, FilterValue::None);
            }
        }
        map
    });
    let filters = filter_state.as_ref().unwrap_or(&internal_filters);
    let open_filter = use_signal(|| Option::<&'static str>::None);

    // ── Phase 3: Column resize state ──

    // Overridden column widths (set by user dragging the resize handle).
    // Key is the column key, value is the pixel width.
    // If a column is not in this map, the calculated width is used instead.
    let resized_widths: Signal<HashMap<&'static str, f64>> = use_signal(HashMap::new);

    // Tracks an active resize operation: (column_key, start_mouse_x, start_width).
    let resize_active: Signal<Option<(&'static str, f64, f64)>> = use_signal(|| None);

    // The scroll position for virtual scrolling.
    let mut scroll_top: Signal<f64> = use_signal(|| 0.0);

    // ── Phase 4: Inline cell editing state ──

    // Tracks which cell is currently being edited: (row_index, column_key, original_value).
    // `None` means no cell is being edited.
    let mut editing_cell: Signal<Option<(usize, &'static str, String)>> = use_signal(|| None);

    // The current value in the inline edit input.
    let mut edit_value: Signal<String> = use_signal(String::new);

    // ── Phase 5: Column visibility toggle state ──

    // Tracks which columns are currently visible (by column key).
    // Initialised with all column keys.
    let mut visible_columns: Signal<HashSet<&'static str>> = use_signal(|| {
        columns.iter().map(|c| c.key).collect()
    });

    // Whether the column visibility dropdown menu is open.
    let mut show_column_menu: Signal<bool> = use_signal(|| false);

    // Resolved columns — if column toggle is enabled, only visible columns;
    // otherwise the full column list. Used for rendering.
    let display_columns: Vec<ColumnDef<T>> = if column_toggle {
        let vis = visible_columns.read();
        columns.iter().filter(|c| vis.contains(c.key)).cloned().collect()
    } else {
        columns.clone()
    };

    // ---- Data pipeline (filter → sort → paginate) ----
    //
    // In CLIENT mode: filter → sort → paginate locally.
    // In SERVER mode: rows are already pre-processed by the server;
    //                 just wrap them with indices, no local processing.

    let processed_rows = if is_server {
        // Server mode: use rows as-is, wrap with indices
        rows.iter()
            .enumerate()
            .map(|(idx, data)| IndexedRow {
                index: idx,
                data: data.clone(),
            })
            .collect::<Vec<IndexedRow<T>>>()
    } else {
        let page = page_state.read();
        let sort_state_ref = sort.read();
        let filters_ref = filters.read();

        // Step 1: Wrap rows with original index
        let indexed: Vec<IndexedRow<T>> = rows
            .iter()
            .enumerate()
            .map(|(idx, data)| IndexedRow {
                index: idx,
                data: data.clone(),
            })
            .collect();

        // Step 2: Apply filters (client-side only)
        let filtered = apply_filters(indexed, &columns, &filters_ref);

        // Step 3: Apply multi-column sort (client-side only)
        let sorted = crate::components::data_grid::sort::apply_sort(
            filtered,
            &columns,
            &sort_state_ref,
        );

        // Step 4: Slice the current page
        let offset = page.offset();
        let end = (offset + page.page_size).min(sorted.len());
        if offset < sorted.len() {
            sorted[offset..end].to_vec()
        } else {
            Vec::new()
        }
    };

    // ---- Event handlers ----

    // Handle sort click
    let on_sort = {
        let sort = sort.clone();
        let on_sort_change = on_sort_change.clone();
        let is_server = is_server;
        Rc::new(move |key: &'static str| {
            let mut sort = sort;
            let old_sort = sort.read().clone();
            let new_sort = crate::components::data_grid::sort::handle_sort_click(key, &old_sort, false);
            sort.set(new_sort.clone());

            if is_server {
                if let Some(new_col) = new_sort.first() {
                    if let Some(cb) = &on_sort_change {
                        cb.call((new_col.key, new_col.direction));
                    }
                }
            }
        })
    };

    // Handle page change
    let on_page_change = {
        let mut page_state = page_state.clone();
        let on_server_page_change = on_server_page_change.clone();
        let is_server = is_server;
        move |page: usize| {
            page_state.write().go_to(page);
            if is_server {
                if let Some(cb) = &on_server_page_change {
                    cb.call(page);
                }
            }
            scroll_top.set(0.0);
        }
    };

    // Handle page size change
    let on_page_size_change = {
        let mut page_state = page_state.clone();
        let on_server_page_size_change = on_server_page_size_change.clone();
        let is_server = is_server;
        move |size: usize| {
            page_state.write().set_page_size(size);
            if is_server {
                if let Some(cb) = &on_server_page_size_change {
                    cb.call(size);
                }
            }
            scroll_top.set(0.0);
        }
    };

    // ── Phase 2: Filter event handlers ──

    let on_toggle_filter = {
        let open_filter = open_filter.clone();
        Rc::new(move |key: &'static str| {
            let mut open_filter = open_filter;
            let current = *open_filter.read();
            if current == Some(key) {
                open_filter.set(None);
            } else {
                open_filter.set(Some(key));
            }
        })
    };

    let on_filter_change_internal = {
        let filters = filters.clone();
        let on_filter_change = on_filter_change.clone();
        Rc::new(move |key: &'static str, value: FilterValue| {
            let mut filters = filters;
            let mut f = filters.read().clone();
            f.insert(key, value.clone());
            filters.set(f);
            if let Some(cb) = &on_filter_change {
                cb.call((key, value));
            }
        })
    };

    // ── Phase 3: Resize event handlers ──

    let on_resize_start = Rc::new({
        let resize_active = resize_active.clone();
        let resized_widths = resized_widths.clone();
        move |col_key: &'static str, start_x: f64| {
            let mut resize_active = resize_active;
            let resized_widths = resized_widths;
            let current_width = resized_widths
                .read()
                .get(col_key)
                .copied()
                .unwrap_or(120.0);
            resize_active.set(Some((col_key, start_x, current_width)));
        }
    });

    // Called from the document mousemove listener (via a div overlay during resize)
    let on_resize_move = {
        let resize_active = resize_active.clone();
        let mut resized_widths = resized_widths.clone();
        let on_column_resize = on_column_resize.clone();
        move |current_x: f64| {
            let active = resize_active.read().clone();
            if let Some((col_key, start_x, start_width)) = active {
                let delta = current_x - start_x;
                let new_width = (start_width + delta)
                    .max(MIN_COLUMN_WIDTH)
                    .min(MAX_COLUMN_WIDTH);
                resized_widths.write().insert(col_key, new_width);
                if let Some(cb) = &on_column_resize {
                    cb.call((col_key, new_width));
                }
            }
        }
    };

    // Called from the document mouseup listener
    let mut on_resize_end = {
        let mut resize_active = resize_active.clone();
        move || {
            resize_active.set(None);
        }
    };

    // ── Phase 4: Inline cell editing event handlers ──

    let mut on_edit_start = {
        let mut editing_cell = editing_cell.clone();
        let mut edit_value = edit_value.clone();
        move |row_idx: usize, col_key: &'static str, current_val: String| {
            let old_val = current_val.clone();
            editing_cell.set(Some((row_idx, col_key, old_val)));
            edit_value.set(current_val);
        }
    };

    let mut on_edit_commit = {
        let mut editing_cell = editing_cell.clone();
        let mut edit_value = edit_value.clone();
        let on_cell_edit = on_cell_edit.clone();
        move || {
            let cell = editing_cell.read().clone();
            if let Some((row_idx, col_key, old_val)) = cell {
                let new_val = edit_value.read().clone();
                if let Some(cb) = &on_cell_edit {
                    cb.call((row_idx, col_key, old_val, new_val));
                }
            }
            editing_cell.set(None);
            edit_value.set(String::new());
        }
    };

    let mut on_edit_cancel = {
        let mut editing_cell = editing_cell.clone();
        let mut edit_value = edit_value.clone();
        move || {
            editing_cell.set(None);
            edit_value.set(String::new());
        }
    };

    // ---- Column widths (using display_columns to exclude hidden columns) ----

    let col_widths = {
        let base_widths = calculate_column_widths(&display_columns, *container_width.read(), None);
        let resized = resized_widths.read();
        base_widths
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let key = display_columns[i].key;
                resized.get(key).copied().unwrap_or(*w)
            })
            .collect::<Vec<f64>>()
    };

    // ---- Compute pinned column boundaries for CSS sticky ----

    let (left_pinned_keys, right_pinned_keys) = {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for col in &display_columns {
            match col.pinned {
                Some(PinnedPosition::Left) => left.push(col.key),
                Some(PinnedPosition::Right) => right.push(col.key),
                None => {}
            }
        }
        // Right pinned is stored in reverse order so rightmost column is last
        right.reverse();
        (left, right)
    };

    // Compute cumulative left offset for left-pinned columns
    let compute_left_offset = |idx: usize| -> f64 {
        let checkbox_offset = if selection_mode == SelectionMode::Multi {
            CHECKBOX_COL_WIDTH
        } else {
            0.0
        };
        let mut offset = checkbox_offset;
        for i in 0..idx {
            if let Some(PinnedPosition::Left) = display_columns.get(i).and_then(|c| c.pinned) {
                offset += col_widths.get(i).copied().unwrap_or(0.0);
            }
        }
        offset
    };

    // Compute cumulative right offset for right-pinned columns
    let compute_right_offset = |idx: usize| -> f64 {
        let mut offset = 0.0;
        let right_indices: Vec<usize> = display_columns
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.pinned, Some(PinnedPosition::Right)))
            .map(|(i, _)| i)
            .collect();
        // Right-pinned columns are in reverse view-order, so the last one is rightmost
        let pos_in_right = right_indices.iter().position(|&i| i == idx).unwrap_or(0);
        for i in (pos_in_right + 1)..right_indices.len() {
            let col_idx = right_indices[i];
            offset += col_widths.get(col_idx).copied().unwrap_or(0.0);
        }
        offset
    };

    // ── Render ──

    // Skeleton state
    if skeleton && rows.is_empty() {
        return rsx! {
            div {
                class: "dg-container",
                role: "table",
                aria_label: "Loading data grid",
                div { class: "dg-header-row",
                    {columns.iter().map(|col| {
                        rsx! {
                            div {
                                class: "dg-header-cell",
                                style: "width: 120px;",
                                div { class: "dg-skeleton", style: "width: 80%; height: 14px;" }
                            }
                        }
                    })}
                },
                {(0..skeleton_rows).map(|_| {
                    rsx! {
                        div { class: "dg-skeleton-row",
                            {columns.iter().map(|_| {
                                rsx! {
                                    div {
                                        class: "dg-skeleton",
                                        style: "width: {random_width()}%; height: 14px; flex: 1;",
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
        };
    }

    // Loading state
    if loading {
        return rsx! {
            div {
                class: "dg-container",
                role: "table",
                aria_label: "Loading data grid",
                div { class: "dg-loading-overlay",
                    div { class: "dg-loading-spinner" }
                }
            }
        };
    }

    // Empty state
    if rows.is_empty() {
        return rsx! {
            div {
                class: "dg-container",
                role: "table",
                aria_label: "Empty data grid",
                div { class: "dg-empty-state",
                    div { class: "dg-empty-state-icon", "📋" }
                    div { class: "dg-empty-state-text", "{empty_message}" }
                    if let Some(hint) = &empty_hint {
                        div { class: "dg-empty-state-hint", "{hint}" }
                    }
                }
            }
        };
    }

    // ── Main render ──

    let all_rows = processed_rows;
    let total_display_rows = all_rows.len();

    // Clone everything needed for the RSX closures
    let columns_for_header = display_columns.clone();
    let widths = col_widths.clone();
    let sort_state = sort.read().clone();
    let on_sort_cb = on_sort.clone();
    let row_height_px = row_height.px();
    let is_striped = striped;
    let selection_read = selection.read().clone();
    let sel_mode = selection_mode;
    let filters_read = filters.read().clone();
    let open_filter_val = *open_filter.read();
    let active_filter_count = count_active_filters(&filters_read);

    // --------
    // Virtual scrolling: compute visible range
    // --------
    let use_virtual = virtual_scroll && total_display_rows > 20;

    let scroll_top_val = *scroll_top.read();
    let mut visible_start: usize;
    let mut visible_end: usize;
    let spacer_top_px: f64;
    let spacer_bottom_px: f64;
    let total_content_height: f64;

    if use_virtual {
        let row_h = row_height_px as f64;
        total_content_height = total_display_rows as f64 * row_h;

        // Compute visible range from scroll position
        let start_row = (scroll_top_val / row_h).floor() as usize;
        let viewport_height = virtual_scroll_height;
        let visible_count = (viewport_height / row_h).ceil() as usize + VIRTUAL_SCROLL_BUFFER * 2;

        visible_start = start_row.saturating_sub(VIRTUAL_SCROLL_BUFFER);
        visible_end = (visible_start + visible_count).min(total_display_rows);
        // Ensure at least some rows
        if visible_end <= visible_start {
            visible_start = 0;
            visible_end = visible_count.min(total_display_rows);
        }
        spacer_top_px = visible_start as f64 * row_h;
        spacer_bottom_px = (total_display_rows - visible_end) as f64 * row_h;

        // Scroll to top when page changes
    } else {
        visible_start = 0;
        visible_end = total_display_rows;
        spacer_top_px = 0.0;
        spacer_bottom_px = 0.0;
        total_content_height = 0.0;
    }

    // The slice of rows to render
    let visible_rows = &all_rows[visible_start..visible_end];

    // Track resize overlay (full-screen capture during drag)
    let is_resizing = resize_active.read().is_some();

    // Compute sticky styles for each column (using display_columns for correct indices)
    let get_cell_styles = |col_idx: usize, is_header: bool| -> String {
        let col = &display_columns[col_idx];
        let base_width = widths.get(col_idx).copied().unwrap_or(120.0);
        let mut styles = format!("width: {base_width}px;");

        match col.pinned {
            Some(PinnedPosition::Left) => {
                let offset = compute_left_offset(col_idx);
                styles.push_str(&format!(
                    "position: sticky; left: {offset}px; z-index: {};",
                    if is_header { 20 } else { 12 },
                ));
            }
            Some(PinnedPosition::Right) => {
                let offset = compute_right_offset(col_idx);
                styles.push_str(&format!(
                    "position: sticky; right: {offset}px; z-index: {};",
                    if is_header { 20 } else { 12 },
                ));
            }
            None => {}
        }

        styles
    };

    rsx! {
        div {
            class: "dg-container",
            role: "table",
            aria_label: "Data grid",
            aria_rowcount: total_items,

            // Close column visibility menu when clicking anywhere on the grid
            onclick: {
                let mut show_menu = show_column_menu.clone();
                move |_| {
                    show_menu.set(false);
                }
            },

            div {
                class: "dg-table-wrapper",
                role: "rowgroup",

                // ── Filter bar ──
                if active_filter_count > 0 {
                    div { class: "dg-filter-bar",
                        span { class: "dg-filter-bar-label",
                            "{active_filter_count} filter(s) active"
                        }
                        button {
                            class: "dg-filter-bar-clear-all",
                            r#type: "button",
                            onclick: {
                                let mut filters = filters.clone();
                                move |_| {
                                    let mut f = filters.write();
                                    for v in f.values_mut() {
                                        v.clear();
                                    }
                                }
                            },
                            "Clear all filters"
                        }
                    }
                }

                // ── Column visibility toggle toolbar ──
                if column_toggle {
                    div {
                        class: "dg-column-toggle-bar",
                        div { class: "dg-column-toggle-spacer" }
                        div {
                            class: "dg-column-toggle-btn-wrapper",
                            button {
                                class: format!("dg-column-toggle-btn{}",
                                    if *show_column_menu.read() { " dg-column-toggle-open" } else { "" }
                                ),
                                r#type: "button",
                                onclick: {
                                    let mut show_menu = show_column_menu.clone();
                                    move |e| {
                                        e.stop_propagation();
                                        let mut s = show_menu.write();
                                        *s = !*s;
                                    }
                                },
                                span { class: "dg-column-toggle-icon", "⚙" }
                                span { "Columns" }
                                span { class: "dg-column-toggle-arrow", "▾" }
                            }
                            if *show_column_menu.read() {
                                div {
                                    class: "dg-column-toggle-menu",
                                    onclick: |e| e.stop_propagation(),
                                    {columns.iter().map(|col| {
                                        let is_visible = visible_columns.read().contains(col.key);
                                        rsx! {
                                            label {
                                                class: "dg-column-toggle-option",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: is_visible,
                                                    oninput: {
                                                        let key = col.key;
                                                        move |_: Event<FormData>| {
                                                            let mut vis = visible_columns.write();
                                                            if vis.contains(key) {
                                                                vis.remove(key);
                                                            } else {
                                                                vis.insert(key);
                                                            }
                                                        }
                                                    },
                                                },
                                                span { "{col.header}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }

                // ── Header row (with sort + filter controls + resize handles) ──
                {render_header_row(
                    &columns_for_header,
                    &widths,
                    &sort_state,
                    on_sort_cb,
                    &filters_read,
                    open_filter_val,
                    on_toggle_filter,
                    on_filter_change_internal,
                    Some(on_resize_start.clone()),
                )}

                // ── Body ──
                if use_virtual {
                    // Virtual scrolling body
                    div {
                        class: "dg-body dg-virtual-body",
                        style: "height: {virtual_scroll_height}px; overflow-y: auto;",
                        role: "rowgroup",

                        // Listen for scroll to update visible range
                        onscroll: {
                            let mut scroll_top = scroll_top.clone();
                            move |e| {
                                scroll_top.set(e.scroll_top());
                            }
                        },

                        // Spacer above (to maintain total height)
                        div {
                            class: "dg-vspacer",
                            style: "height: {spacer_top_px}px;",
                        }

                        // Visible rows
                        {visible_rows.iter().map(|indexed_row| {
                            let idx = indexed_row.index;
                            let data = &indexed_row.data;
                            let is_selected = selection_read.contains(&idx);
                            let is_even = idx % 2 == 0;

                            let mut row_classes = vec!["dg-row"];
                            if is_striped && is_even {
                                row_classes.push("dg-row-even");
                            }
                            if is_selected {
                                row_classes.push("dg-row-selected");
                            }
                            let row_class = row_classes.join(" ");

                            let row_data = indexed_row.data.clone();
                            let row_data_dbl = row_data.clone();
                            let row_data_key = row_data.clone();
                            let row_on_click = on_row_click.clone();
                            let mut row_sel = selection.clone();
                            let row_sel_mode = selection_mode;
                            let mut row_edit_cancel = on_edit_cancel.clone();
                            let row_dbl_click = on_row_double_click.clone();
                            let row_key_click = on_row_click.clone();

                            rsx! {
                                div {
                                    key: "row-{idx}",
                                    class: "{row_class}",
                                    role: "row",
                                    style: "height: {row_height_px}px;",
                                    aria_selected: if is_selected { "true" } else { "false" },
                                    tabindex: 0,
                                    onclick: move |_| {
                                        row_edit_cancel();
                                        match row_sel_mode {
                                            SelectionMode::Single => {
                                                let mut s = row_sel.write();
                                                s.clear();
                                                s.insert(idx);
                                            }
                                            SelectionMode::Multi => {
                                                let mut s = row_sel.write();
                                                if s.contains(&idx) {
                                                    s.remove(&idx);
                                                } else {
                                                    s.insert(idx);
                                                }
                                            }
                                            SelectionMode::None => {}
                                        }
                                        if let Some(cb) = &row_on_click {
                                            cb.call((idx, row_data.clone()));
                                        }
                                    },
                                    ondoubleclick: move |_| {
                                        if let Some(cb) = &row_dbl_click {
                                            cb.call((idx, row_data_dbl.clone()));
                                        }
                                    },
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            if let Some(cb) = &row_key_click {
                                                cb.call((idx, row_data_key.clone()));
                                            }
                                        }
                                    },

                                    // Multi-select checkbox (always rendered)
                                    if row_sel_mode == SelectionMode::Multi {
                                        div {
                                            class: "dg-cell dg-align-center",
                                            style: "width: {CHECKBOX_COL_WIDTH}px; flex-shrink: 0;",
                                            role: "gridcell",
                                            input {
                                                r#type: "checkbox",
                                                checked: is_selected,
                                                oninput: {
                                                    let mut sel = selection.clone();
                                                    let idx = idx;
                                                    move |_| {
                                                        let mut s = sel.write();
                                                        if s.contains(&idx) {
                                                            s.remove(&idx);
                                                        } else {
                                                            s.insert(idx);
                                                        }
                                                    }
                                                },
                                                aria_label: "Select row {idx + 1}",
                                            }
                                        }
                                    }

                                    // Data cells with sticky positioning for pinned columns
                                    {display_columns.iter().enumerate().map(|(col_idx, col)| {
                                        let cell_value = (col.get_value)(data);
                                        let cell_width = widths
                                            .get(col_idx)
                                            .copied()
                                            .unwrap_or(120.0);

                                        let extra_class = col.cell_class_rule
                                            .as_ref()
                                            .map(|rule| (rule.class_fn)(data))
                                            .unwrap_or_default();

                                        let pinned_cls = match col.pinned {
                                            Some(PinnedPosition::Left) => " dg-pinned-left",
                                            Some(PinnedPosition::Right) => " dg-pinned-right",
                                            None => "",
                                        };

                                        let is_editing = editing_cell
                                            .read()
                                            .as_ref()
                                            .map(|(r, c, _)| *r == idx && *c == col.key)
                                            .unwrap_or(false);
                                        let edit_val = edit_value.read().clone();

                                        let cell_class = format!(
                                            "dg-cell dg-align-{} {}{}",
                                            match col.align {
                                                TextAlign::Left => "left",
                                                TextAlign::Center => "center",
                                                TextAlign::Right => "right",
                                            },
                                            extra_class,
                                            pinned_cls,
                                        );

                                        let cell_styles = get_cell_styles(col_idx, false);

                                        if is_editing && col.editable {
                                            rsx! {
                                                div {
                                                    class: "{cell_class} dg-cell-editing",
                                                    role: "gridcell",
                                                    style: "{cell_styles}",
                                                    aria_label: "Editing {col.header}",

                                                    input {
                                                        class: "dg-cell-editor",
                                                        r#type: "text",
                                                        value: "{edit_val}",
                                                        autofocus: true,
                                                        oninput: {
                                                            let mut edit_value = edit_value.clone();
                                                            move |e| {
                                                                edit_value.set(e.value());
                                                            }
                                                        },
                                                        onkeydown: {
                                                            let mut commit = on_edit_commit.clone();
                                                            let mut cancel = on_edit_cancel.clone();
                                                            move |e| {
                                                                if e.key() == Key::Enter {
                                                                    commit();
                                                                } else if e.key() == Key::Escape {
                                                                    cancel();
                                                                }
                                                            }
                                                        },
                                                        onblur: {
                                                            let mut commit = on_edit_commit.clone();
                                                            move |_| {
                                                                commit();
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div {
                                                    class: "{cell_class}",
                                                    role: "gridcell",
                                                    style: "{cell_styles}",
                                                    aria_label: "{col.header}: {cell_value}",
                                                    ondoubleclick: {
                                                        let idx = idx;
                                                        let col = col.clone();
                                                        let cell_value = cell_value.clone();
                                                        move |_| {
                                                            if col.editable {
                                                                on_edit_start(idx, col.key, cell_value.clone());
                                                            }
                                                        }
                                                    },
                                                    {render_cell(&cell_value, &col.renderer)}
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        })}

                        // Spacer below
                        div {
                            class: "dg-vspacer",
                            style: "height: {spacer_bottom_px}px;",
                        }
                    }
                } else {
                    // Non-virtual body (render all rows)
                    div {
                        class: "dg-body",
                        role: "rowgroup",

                        {all_rows.iter().map(|indexed_row| {
                            let idx = indexed_row.index;
                            let data = &indexed_row.data;
                            let is_selected = selection_read.contains(&idx);
                            let is_even = idx % 2 == 0;

                            let mut row_classes = vec!["dg-row"];
                            if is_striped && is_even {
                                row_classes.push("dg-row-even");
                            }
                            if is_selected {
                                row_classes.push("dg-row-selected");
                            }
                            let row_class = row_classes.join(" ");

                            let row_data = indexed_row.data.clone();
                            let row_data_dbl = row_data.clone();
                            let row_data_key = row_data.clone();
                            let row_on_click = on_row_click.clone();
                            let mut row_sel = selection.clone();
                            let row_sel_mode = selection_mode;
                            let mut row_edit_cancel = on_edit_cancel.clone();
                            let row_dbl_click = on_row_double_click.clone();
                            let row_key_click = on_row_click.clone();

                            rsx! {
                                div {
                                    key: "row-{idx}",
                                    class: "{row_class}",
                                    role: "row",
                                    style: "height: {row_height_px}px;",
                                    aria_selected: if is_selected { "true" } else { "false" },
                                    tabindex: 0,
                                    onclick: move |_| {
                                        row_edit_cancel();
                                        match row_sel_mode {
                                            SelectionMode::Single => {
                                                let mut s = row_sel.write();
                                                s.clear();
                                                s.insert(idx);
                                            }
                                            SelectionMode::Multi => {
                                                let mut s = row_sel.write();
                                                if s.contains(&idx) {
                                                    s.remove(&idx);
                                                } else {
                                                    s.insert(idx);
                                                }
                                            }
                                            SelectionMode::None => {}
                                        }
                                        if let Some(cb) = &row_on_click {
                                            cb.call((idx, row_data.clone()));
                                        }
                                    },
                                    ondoubleclick: move |_| {
                                        if let Some(cb) = &row_dbl_click {
                                            cb.call((idx, row_data_dbl.clone()));
                                        }
                                    },
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            if let Some(cb) = &row_key_click {
                                                cb.call((idx, row_data_key.clone()));
                                            }
                                        }
                                    },

                                    if row_sel_mode == SelectionMode::Multi {
                                        div {
                                            class: "dg-cell dg-align-center",
                                            style: "width: {CHECKBOX_COL_WIDTH}px; flex-shrink: 0;",
                                            role: "gridcell",
                                            input {
                                                r#type: "checkbox",
                                                checked: is_selected,
                                                oninput: {
                                                    let mut sel = selection.clone();
                                                    let idx = idx;
                                                    move |_| {
                                                        let mut s = sel.write();
                                                        if s.contains(&idx) {
                                                            s.remove(&idx);
                                                        } else {
                                                            s.insert(idx);
                                                        }
                                                    }
                                                },
                                                aria_label: "Select row {idx + 1}",
                                            }
                                        }
                                    }

                                    {display_columns.iter().enumerate().map(|(col_idx, col)| {
                                        let cell_value = (col.get_value)(data);
                                        let cell_width = widths
                                            .get(col_idx)
                                            .copied()
                                            .unwrap_or(120.0);

                                        let extra_class = col.cell_class_rule
                                            .as_ref()
                                            .map(|rule| (rule.class_fn)(data))
                                            .unwrap_or_default();

                                        let pinned_cls = match col.pinned {
                                            Some(PinnedPosition::Left) => " dg-pinned-left",
                                            Some(PinnedPosition::Right) => " dg-pinned-right",
                                            None => "",
                                        };

                                        let is_editing = editing_cell
                                            .read()
                                            .as_ref()
                                            .map(|(r, c, _)| *r == idx && *c == col.key)
                                            .unwrap_or(false);
                                        let edit_val = edit_value.read().clone();

                                        let cell_class = format!(
                                            "dg-cell dg-align-{} {}{}",
                                            match col.align {
                                                TextAlign::Left => "left",
                                                TextAlign::Center => "center",
                                                TextAlign::Right => "right",
                                            },
                                            extra_class,
                                            pinned_cls,
                                        );

                                        let cell_styles = get_cell_styles(col_idx, false);

                                        if is_editing && col.editable {
                                            rsx! {
                                                div {
                                                    class: "{cell_class} dg-cell-editing",
                                                    role: "gridcell",
                                                    style: "{cell_styles}",
                                                    aria_label: "Editing {col.header}",

                                                    input {
                                                        class: "dg-cell-editor",
                                                        r#type: "text",
                                                        value: "{edit_val}",
                                                        autofocus: true,
                                                        oninput: {
                                                            let mut edit_value = edit_value.clone();
                                                            move |e| {
                                                                edit_value.set(e.value());
                                                            }
                                                        },
                                                        onkeydown: {
                                                            let mut commit = on_edit_commit.clone();
                                                            let mut cancel = on_edit_cancel.clone();
                                                            move |e| {
                                                                if e.key() == Key::Enter {
                                                                    commit();
                                                                } else if e.key() == Key::Escape {
                                                                    cancel();
                                                                }
                                                            }
                                                        },
                                                        onblur: {
                                                            let mut commit = on_edit_commit.clone();
                                                            move |_| {
                                                                commit();
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div {
                                                    class: "{cell_class}",
                                                    role: "gridcell",
                                                    style: "{cell_styles}",
                                                    aria_label: "{col.header}: {cell_value}",
                                                    ondoubleclick: {
                                                        let idx = idx;
                                                        let col = col.clone();
                                                        let cell_value = cell_value.clone();
                                                        move |_| {
                                                            if col.editable {
                                                                on_edit_start(idx, col.key, cell_value.clone());
                                                            }
                                                        }
                                                    },
                                                    {render_cell(&cell_value, &col.renderer)}
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        })}
                    }
                }
            }

            // Resize capture overlay (full-screen while dragging)
            if is_resizing {
                div {
                    class: "dg-resize-overlay",
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 9999; cursor: col-resize;",
                    onmousemove: move |e| {
                        let mut cb = on_resize_move.clone();
                        cb(e.data().client_coordinates().x);
                    },
                    onmouseup: move |_| {
                        on_resize_end();
                    },
                    onmouseleave: move |_| {
                        on_resize_end();
                    },
                }
            }

            // Pagination bar
            if pagination != PaginationMode::None {
                {rsx! {
                    PaginationBar {
                        page_state: page_state(),
                        on_page_change: on_page_change,
                        on_page_size_change: on_page_size_change,
                    }
                }}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn random_width() -> u32 {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    let c = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    30 + (c * 7) % 65
}
