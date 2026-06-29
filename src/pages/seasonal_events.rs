//! Seasonal Events Page — Manage seasonal events with impact factors for forecast adjustment.

use crate::components::common::{Button, ButtonSize, ButtonVariant, Modal, ModalSize, use_toast};
use crate::components::data_grid::{
    BadgeColor, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode, RowHeight,
    SelectionMode,
};
use dioxus::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.se-page { max-width: 1100px; margin: 0 auto; }
.se-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.se-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.se-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.se-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.se-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.se-actions { display: flex; gap: 8px; margin-bottom: 16px; }

.se-form { display: flex; flex-direction: column; gap: 14px; }
.se-form-row { display: flex; gap: 12px; flex-wrap: wrap; }
.se-form-row > * { flex: 1; min-width: 160px; }
.se-form-group { display: flex; flex-direction: column; gap: 4px; }
.se-form-group label { font-size: 12px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.3px; }
.se-form-group input, .se-form-group select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 8px 10px; font-size: 13px; background: #fff; }
.se-form-group textarea { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 8px 10px; font-size: 13px; background: #fff; resize: vertical; min-height: 60px; font-family: inherit; }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct SeasonalEvent {
    pub id: i64,
    pub event_name: String,
    pub start_date: String,
    pub end_date: String,
    pub impact_factor: f64,
    pub recurring: bool,
    pub category: String,
    pub notes: String,
}

// ============================================================================
// Component
// ============================================================================

fn empty_event() -> SeasonalEvent {
    SeasonalEvent {
        id: 0,
        event_name: String::new(),
        start_date: String::new(),
        end_date: String::new(),
        impact_factor: 10.0,
        recurring: true,
        category: "Commercial".to_string(),
        notes: String::new(),
    }
}

#[component]
pub fn SeasonalEventsPage() -> Element {
    let toast = use_toast();
    let _navigator = use_navigator();
    // ponytail: no seasonal events endpoint — add when server exposes one
    let events: Signal<Vec<SeasonalEvent>> = use_signal(Vec::new);
    let selected_ids = use_signal(|| HashSet::<usize>::new());
    let mut show_add_modal = use_signal(|| false);
    let mut form_event = use_signal(|| empty_event());

    let columns: Vec<ColumnDef<SeasonalEvent>> = vec![
        ColumnDef::text("name", "Event Name", |e: &SeasonalEvent| e.event_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("start", "Start Date", |e: &SeasonalEvent| e.start_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Text),
        ColumnDef::text("end", "End Date", |e: &SeasonalEvent| e.end_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Text),
        ColumnDef::text("impact", "Impact Factor", |e: &SeasonalEvent| format!("{:.0}%", e.impact_factor))
            .with_width(ColumnWidth::Px(120))
            .with_align(crate::components::data_grid::TextAlign::Right)
            .with_renderer(crate::components::data_grid::CellRenderer::Badge {
                color_map: vec![
                    ("45%", BadgeColor::Red),
                    ("35%", BadgeColor::Yellow),
                    ("30%", BadgeColor::Yellow),
                    ("28%", BadgeColor::Yellow),
                    ("25%", BadgeColor::Yellow),
                    ("22%", BadgeColor::Yellow),
                    ("20%", BadgeColor::Blue),
                    ("18%", BadgeColor::Blue),
                    ("15%", BadgeColor::Green),
                ],
                default_color: BadgeColor::Gray,
            }),
        ColumnDef::text("recurring", "Recurring", |e: &SeasonalEvent| if e.recurring { "Yes".to_string() } else { "No".to_string() })
            .with_width(ColumnWidth::Px(100))
            .with_renderer(crate::components::data_grid::CellRenderer::Badge {
                color_map: vec![
                    ("Yes", BadgeColor::Green),
                    ("No", BadgeColor::Gray),
                ],
                default_color: BadgeColor::Gray,
            }),
        ColumnDef::text("category", "Category", |e: &SeasonalEvent| e.category.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Select {
                options: vec!["Religious".to_string(), "Weather".to_string(), "Educational".to_string(), "National".to_string(), "Commercial".to_string()],
            }),
        ColumnDef::text("notes", "Notes", |e: &SeasonalEvent| e.notes.clone())
            .with_width(ColumnWidth::Fr(0.8)),
    ];

    let on_add = {
        let mut show = show_add_modal.clone();
        let mut f = form_event.clone();
        move |_| {
            f.set(empty_event());
            show.set(true);
        }
    };

    let on_save_event = {
        let mut show = show_add_modal.clone();
        let mut events = events.clone();
        let mut f = form_event.clone();
        let mut t = toast.clone();
        move |_| {
            let mut new_event = f.read().clone();
            if new_event.event_name.trim().is_empty() {
                t.warning("Name Required", "Event name is required.");
                return;
            }
            new_event.id = events.read().len() as i64 + 1;
            let name = new_event.event_name.clone();
            events.write().push(new_event);
            show.set(false);
            t.success("Event Added", &format!("\"{}\" has been added.", name));
        }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page se-page",

            div { class: "se-header",
                div {
                    h1 { "Seasonal Events" }
                    p { class: "page-subtitle", "Manage seasonal events and their impact on demand forecasts." }
                }
            }

            // Filter bar
            div { class: "se-filter-bar",
                label { "Category" }
                select {
                    option { value: "all", selected: true, "All Categories" }
                    option { value: "Religious", "Religious" }
                    option { value: "Weather", "Weather" }
                    option { value: "Educational", "Educational" }
                    option { value: "National", "National" }
                    option { value: "Commercial", "Commercial" }
                }
                label { "Recurring" }
                select {
                    option { value: "all", selected: true, "All" }
                    option { value: "yes", "Recurring Only" }
                    option { value: "no", "Non-Recurring" }
                }
                label { "Date Range" }
                select {
                    option { value: "all", selected: true, "All Dates" }
                    option { value: "upcoming", "Upcoming (3mo)" }
                    option { value: "past", "Past Events" }
                }
            }

            // Actions
            div { class: "se-actions",
                Button { variant: ButtonVariant::Primary, icon: Some("＋".to_string()), onclick: on_add, "Add Event" }
            }

            // DataGrid
            DataGrid {
                columns: columns.clone(),
                rows: events.read().clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
            }

            // Add Event Modal
            Modal {
                is_open: show_add_modal,
                title: Some("Add Seasonal Event".to_string()),
                size: ModalSize::Md,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| { show_add_modal.set(false); }, "Cancel" }
                    Button { variant: ButtonVariant::Primary, onclick: on_save_event, "Save Event" }
                },
                div { class: "se-form",
                    div { class: "se-form-row",
                        div { class: "se-form-group", style: "flex: 2;",
                            label { "Event Name" }
                            input { r#type: "text", placeholder: "Enter event name",
                                value: "{form_event.read().event_name}",
                                oninput: move |e| { form_event.write().event_name = e.value(); }
                            }
                        }
                        div { class: "se-form-group",
                            label { "Category" }
                            select {
                                value: "{form_event.read().category}",
                                oninput: move |e| { form_event.write().category = e.value(); },
                                option { value: "Religious", "Religious" }
                                option { value: "Weather", "Weather" }
                                option { value: "Educational", "Educational" }
                                option { value: "National", "National" }
                                option { value: "Commercial", "Commercial" }
                            }
                        }
                    }
                    div { class: "se-form-row",
                        div { class: "se-form-group",
                            label { "Start Date" }
                            input { r#type: "date",
                                value: "{form_event.read().start_date}",
                                oninput: move |e| { form_event.write().start_date = e.value(); }
                            }
                        }
                        div { class: "se-form-group",
                            label { "End Date" }
                            input { r#type: "date",
                                value: "{form_event.read().end_date}",
                                oninput: move |e| { form_event.write().end_date = e.value(); }
                            }
                        }
                        div { class: "se-form-group",
                            label { "Impact Factor (%)" }
                            input { r#type: "number", step: "1", min: "0", max: "100",
                                value: "{form_event.read().impact_factor}",
                                oninput: move |e| { form_event.write().impact_factor = e.value().parse().unwrap_or(10.0); }
                            }
                        }
                    }
                    div { class: "se-form-row",
                        div { class: "se-form-group",
                            label { "Recurring" }
                            select {
                                value: if form_event.read().recurring { "yes" } else { "no" },
                                oninput: move |e| { form_event.write().recurring = e.value() == "yes"; },
                                option { value: "yes", "Yes" }
                                option { value: "no", "No" }
                            }
                        }
                    }
                    div { class: "se-form-row",
                        div { class: "se-form-group",
                            label { "Notes" }
                            textarea { placeholder: "Optional notes…",
                                value: "{form_event.read().notes}",
                                oninput: move |e| { form_event.write().notes = e.value(); }
                            }
                        }
                    }
                }
            }
        }
    }
}
