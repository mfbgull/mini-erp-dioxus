//! Pagination controls for the `DataGrid<T>`.
//!
//! Phase 1 provides: page navigation (first, prev, next, last, go-to-page),
//! page size selector, and a summary label showing "1–50 of 234 items".

use dioxus::prelude::*;

// ---------------------------------------------------------------------------
// Page state
// ---------------------------------------------------------------------------

/// Internal pagination state.
#[derive(Clone, Copy, PartialEq)]
pub struct PageState {
    pub current_page: usize, // 1-indexed
    pub page_size: usize,
    pub total_items: usize,
}

impl PageState {
    pub fn new(page_size: usize, total_items: usize) -> Self {
        Self {
            current_page: 1,
            page_size,
            total_items,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.total_items == 0 {
            return 1;
        }
        (self.total_items + self.page_size - 1) / self.page_size
    }

    pub fn offset(&self) -> usize {
        (self.current_page - 1) * self.page_size
    }

    pub fn start_item(&self) -> usize {
        if self.total_items == 0 {
            return 0;
        }
        self.offset() + 1
    }

    pub fn end_item(&self) -> usize {
        self.offset() + self.page_size.min(self.total_items - self.offset())
    }

    pub fn can_prev(&self) -> bool {
        self.current_page > 1
    }

    pub fn can_next(&self) -> bool {
        self.current_page < self.total_pages()
    }

    pub fn prev(&mut self) {
        if self.can_prev() {
            self.current_page -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.can_next() {
            self.current_page += 1;
        }
    }

    pub fn go_to(&mut self, page: usize) {
        let total = self.total_pages();
        if page >= 1 && page <= total {
            self.current_page = page;
        }
    }

    pub fn set_page_size(&mut self, new_size: usize) {
        if new_size > 0 {
            let old_offset = self.offset();
            self.page_size = new_size;
            // Keep the current position, adjusting page number
            self.current_page = (old_offset / new_size).max(0) + 1;
            if self.current_page > self.total_pages() {
                self.current_page = self.total_pages();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Generate a set of page numbers to display, with ellipsis for large ranges.
fn visible_page_numbers(current: usize, total: usize, max_visible: usize) -> Vec<Option<usize>> {
    if total <= max_visible {
        return (1..=total).map(Some).collect();
    }

    let mut pages: Vec<Option<usize>> = Vec::new();

    // Always show first page
    pages.push(Some(1));

    let half = (max_visible - 2) / 2; // -2 for first and last page
    let start = (current.saturating_sub(half)).max(2);
    let end = (start + max_visible - 3).min(total - 1);

    if start > 2 {
        pages.push(None); // ellipsis
    }

    for p in start..=end {
        pages.push(Some(p));
    }

    if end < total - 1 {
        pages.push(None); // ellipsis
    }

    // Always show last page
    pages.push(Some(total));

    pages
}

/// Pagination bar component.
#[component]
pub fn PaginationBar(
    /// Current page state (1-indexed).
    page_state: PageState,
    /// Callback when the page changes.
    on_page_change: EventHandler<usize>,
    /// Callback when the page size changes.
    on_page_size_change: EventHandler<usize>,
) -> Element {
    let total_pages = page_state.total_pages();
    let page_numbers = visible_page_numbers(page_state.current_page, total_pages, 7);

    let page_sizes = [10, 25, 50, 100, 200];

    rsx! {
        nav {
            class: "dg-pagination",
            role: "navigation",
            aria_label: "Pagination",

            // Page size selector
            label {
                class: "dg-page-size-label",
                "Rows: ",
                select {
                    class: "dg-page-size-select",
                    value: "{page_state.page_size}",
                    oninput: move |e| {
                        if let Ok(size) = e.value().parse::<usize>() {
                            on_page_size_change.call(size);
                        }
                    },
                    {page_sizes.iter().map(|&size| {
                        rsx! {
                            option {
                                value: "{size}",
                                selected: page_state.page_size == size,
                                "{size}"
                            }
                        }
                    })}
                }
            },

            // Summary text
            span {
                class: "dg-pagination-summary",
                "{page_state.start_item()}–{page_state.end_item()} of {page_state.total_items} items",
            },

            // Navigation buttons
            div { class: "dg-pagination-controls",

                // First page
                button {
                    class: "dg-page-btn",
                    disabled: !page_state.can_prev(),
                    onclick: move |_| on_page_change.call(1),
                    aria_label: "First page",
                    "«",
                },

                // Previous page
                button {
                    class: "dg-page-btn",
                    disabled: !page_state.can_prev(),
                    onclick: move |_| on_page_change.call(page_state.current_page - 1),
                    aria_label: "Previous page",
                    "‹",
                },

                // Page number buttons
                {page_numbers.iter().map(|page_opt| {
                    match page_opt {
                        Some(page) => {
                            let is_active = *page == page_state.current_page;
                            let page_val = *page;
                            rsx! {
                                button {
                                    class: format!(
                                        "dg-page-btn {}",
                                        if is_active { "dg-page-active" } else { "" },
                                    ),
                                    disabled: is_active,
                                    onclick: move |_| on_page_change.call(page_val),
                                    aria_label: "Page {page_val}",
                                    aria_current: if is_active { "page" } else { "false" },
                                    "{page_val}",
                                }
                            }
                        }
                        None => rsx! {
                            span { class: "dg-page-ellipsis", "…" }
                        },
                    }
                })},

                // Next page
                button {
                    class: "dg-page-btn",
                    disabled: !page_state.can_next(),
                    onclick: move |_| on_page_change.call(page_state.current_page + 1),
                    aria_label: "Next page",
                    "›",
                },

                // Last page
                button {
                    class: "dg-page-btn",
                    disabled: !page_state.can_next(),
                    onclick: move |_| on_page_change.call(total_pages),
                    aria_label: "Last page",
                    "»",
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_state_basics() {
        let state = PageState::new(50, 234);
        assert_eq!(state.total_pages(), 5);
        assert_eq!(state.offset(), 0);
        assert_eq!(state.start_item(), 1);
        assert_eq!(state.end_item(), 50);
        assert!(state.can_next());
        assert!(!state.can_prev());
    }

    #[test]
    fn test_page_state_navigation() {
        let mut state = PageState::new(50, 120);
        assert_eq!(state.current_page, 1);
        state.next();
        assert_eq!(state.current_page, 2);
        state.next();
        assert_eq!(state.current_page, 3);
        assert!(!state.can_next());
        state.prev();
        assert_eq!(state.current_page, 2);
    }

    #[test]
    fn test_page_state_go_to() {
        let mut state = PageState::new(50, 234);
        state.go_to(3);
        assert_eq!(state.current_page, 3);
        assert_eq!(state.offset(), 100);
        state.go_to(99); // out of range
        assert_eq!(state.current_page, 3);
    }

    #[test]
    fn test_change_page_size() {
        let mut state = PageState::new(50, 234);
        state.go_to(3); // offset = 100
        state.set_page_size(25);
        assert_eq!(state.page_size, 25);
        // offset 100 → page 5 (since 100 / 25 = 4, +1 = 5)
        assert_eq!(state.current_page, 5);
    }

    #[test]
    fn test_empty_total() {
        let state = PageState::new(50, 0);
        assert_eq!(state.total_pages(), 1);
        assert_eq!(state.start_item(), 0);
        assert_eq!(state.end_item(), 0);
        assert!(!state.can_next());
    }

    #[test]
    fn test_visible_page_numbers_small() {
        let pages = visible_page_numbers(1, 5, 7);
        let expected: Vec<Option<usize>> = (1..=5).map(Some).collect();
        assert_eq!(pages, expected);
    }

    #[test]
    fn test_visible_page_numbers_large() {
        let pages = visible_page_numbers(50, 100, 7);
        assert_eq!(pages.first(), Some(&Some(1)));
        assert_eq!(pages.last(), Some(&Some(100)));
        // Should have at least one ellipsis
        assert!(pages.contains(&None));
    }
}
