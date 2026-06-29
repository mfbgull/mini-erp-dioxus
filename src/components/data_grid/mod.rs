//! # DataGrid — Generic data table component for MiniERP (Dioxus)
//!
//! A feature-rich, accessible, and performant replacement for AG-Grid.
//! Built as a generic Dioxus component with pluggable cell renderers,
//! client-side sorting, pagination, and row selection.
//!
//! ## Module Structure
//!
//! ```text
//! data_grid/
//! ├── mod.rs         ← Module root (this file)
//! ├── types.rs       ← Core type definitions (ColumnDef, CellRenderer, etc.)
//! ├── columns.rs     ← Column header rendering & width calculation
//! ├── sort.rs        ← Client-side multi-column sorting
//! ├── pagination.rs  ← Pagination component & state
//! ├── renderers.rs   ← Cell renderer implementations (text, number, badge, etc.)
//! ├── data_grid.rs   ← Main DataGrid<T> component
//! └── styles.rs      ← CSS stylesheet & class name constants
//! ```
//!
//! ## Phases
//!
//! | Phase | Features | Status |
//! |-------|----------|--------|
//! | 1     | Columns, sort, pagination, renderers, selection, loading/empty states | ✅ |
//! | 2     | Column filtering (text, number, date, select), filter bar, filter dropdowns | ✅ |
//! | 3     | Virtual scrolling, column resize handles, pinned/frozen columns | ✅ |
//! | 4     | Inline cell editing (double-click, input, Enter/blur commit, Escape cancel) | ✅ |
//! | 5     | Server-side pagination/sort/filter mode (callbacks, pipeline bypass, total_records), column visibility toggle menu (show/hide columns by key) | ✅ |
//!
//! ## Usage
//!
//! ```ignore
//! use crate::components::data_grid::{DataGrid, ColumnDef, CellRenderer, TextAlign, ColumnWidth, PaginationMode, SelectionMode};
//!
//! #[component]
//! fn InvoiceList() -> Element {
//!     let invoices = use_resource(|| get_invoices());
//!     let columns = vec![
//!         ColumnDef::text("no", "Invoice #", |inv: &Invoice| inv.invoice_no.clone()),
//!         ColumnDef::text("customer", "Customer", |inv| inv.customer_name.clone())
//!             .with_width(ColumnWidth::Fr(1.0)),
//!         ColumnDef::text("date", "Date", |inv| inv.invoice_date.to_string())
//!             .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" }),
//!         ColumnDef::text("amount", "Amount", |inv| inv.total_amount.to_string())
//!             .with_align(TextAlign::Right)
//!             .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
//!         ColumnDef::text("status", "Status", |inv| inv.status.to_string())
//!             .with_renderer(CellRenderer::Badge {
//!                 color_map: vec![("Paid", BadgeColor::Green), ("Unpaid", BadgeColor::Red), ("Partially Paid", BadgeColor::Yellow)],
//!                 default_color: BadgeColor::Gray,
//!             }),
//!     ];
//!
//!     rsx! {
//!         DataGrid {
//!             columns,
//!             rows: invoices.read().unwrap_or_default(),
//!             loading: invoices.read().is_none(),
//!             pagination: PaginationMode::Client { page_size: 25 },
//!             selection_mode: SelectionMode::Multi,
//!             on_row_click: move |(idx, inv)| {
//!                 // navigate to invoice detail
//!             },
//!         }
//!     }
//! }
//! ```

// ---------------------------------------------------------------------------
// Sub-modules
// ---------------------------------------------------------------------------

pub(crate) mod columns;
pub(crate) mod filter;
pub(crate) mod pagination;
pub(crate) mod renderers;
pub(crate) mod sort;
pub mod styles;
pub mod types;

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

mod data_grid;
pub use data_grid::DataGrid;

// Convenience re-exports for callers
pub use types::{
    BadgeColor, CellClassRule, CellRenderer, ColumnDef, ColumnWidth, FilterType, PaginationMode,
    RowHeight, SelectionMode, SortColumn, SortDirection, TextAlign,
};
pub use filter::FilterValue;
