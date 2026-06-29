//! Core types for the generic `DataGrid<T>` component.
//!
//! Phase 1 covers: column definitions, sorting, pagination, cell rendering.
//! These types are designed to be extended in later phases (filtering, editing, etc.)
//! without breaking changes.

use dioxus::prelude::*;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Column Definition
// ---------------------------------------------------------------------------

/// Configuration for a single column in the data grid.
///
/// `T` is the row data type. Each column stores a `get_value` closure that
/// extracts the displayable string from a row, keeping the component generic
/// without requiring a trait bound on `T` for field access.
#[derive(Clone)]
pub struct ColumnDef<T> {
    /// Unique key for this column (used for sorting, filtering, and `key` attributes).
    pub key: &'static str,

    /// Human-readable header text.
    pub header: &'static str,

    /// Width strategy (fixed px, fractional fill, or auto).
    pub width: ColumnWidth,

    /// Whether the user can sort by this column (single-click header).
    pub sortable: bool,

    /// Filter type for this column (Phase 2+).
    pub filter_type: FilterType,

    /// Whether the cell can be edited inline (Phase 4+).
    pub editable: bool,

    /// Text alignment for the cell content.
    pub align: TextAlign,

    /// Closure that extracts the cell's display value from a row.
    ///
    /// # Example
    /// ```ignore
    /// get_value: Rc::new(|item: &Item| item.item_code.clone()),
    /// ```
    pub get_value: Rc<dyn Fn(&T) -> String>,

    /// Renderer that controls how the value is displayed (text, currency, badge, …).
    pub renderer: CellRenderer,

    /// Optional dynamic CSS class rule based on the row's value.
    pub cell_class_rule: Option<CellClassRule<T>>,

    /// Whether the column is resizable by dragging the header edge (Phase 3+).
    pub resizable: bool,

    /// Whether this column should be pinned to the left or right (Phase 3+).
    pub pinned: Option<PinnedPosition>,
}

impl<T> PartialEq for ColumnDef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: 'static> ColumnDef<T> {
    /// Create a new text column with the given key, header, and value extractor.
    pub fn text(
        key: &'static str,
        header: &'static str,
        get_value: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self {
            key,
            header,
            width: ColumnWidth::Auto,
            sortable: true,
            filter_type: FilterType::None,
            editable: false,
            align: TextAlign::Left,
            get_value: Rc::new(get_value),
            renderer: CellRenderer::Text,
            cell_class_rule: None,
            resizable: false,
            pinned: None,
        }
    }

    /// Builder-style setter for width.
    pub fn with_width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Builder-style setter for alignment.
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Builder-style setter for renderer.
    pub fn with_renderer(mut self, renderer: CellRenderer) -> Self {
        self.renderer = renderer;
        self
    }

    /// Builder-style setter for sortable.
    pub fn with_sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Builder-style setter for cell class rule.
    pub fn with_cell_class(mut self, rule: CellClassRule<T>) -> Self {
        self.cell_class_rule = Some(rule);
        self
    }

    /// Builder-style setter for filter type.
    ///
    /// # Example
    /// ```ignore
    /// ColumnDef::text("status", "Status", |r| r.status.clone())
    ///     .with_filter(FilterType::Select {
    ///         options: vec!["Active".to_string(), "Inactive".to_string()],
    ///     });
    /// ```
    pub fn with_filter(mut self, filter_type: FilterType) -> Self {
        self.filter_type = filter_type;
        self
    }

    /// Builder-style setter for resizable (Phase 3).
    ///
    /// When resizable, the column header shows a drag handle on the right
    /// edge that the user can mouse-drag to resize the column width.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Builder-style setter for pinned / frozen column (Phase 3).
    ///
    /// Pinned columns stick to the left or right of the grid while the rest
    /// of the columns scroll horizontally.
    ///
    /// # Example
    /// ```ignore
    /// ColumnDef::text("actions", "", |r| String::new())
    ///     .with_pinned(PinnedPosition::Right)
    ///     .with_width(ColumnWidth::Px(80));
    /// ```
    pub fn with_pinned(mut self, pinned: PinnedPosition) -> Self {
        self.pinned = Some(pinned);
        self
    }

    /// Builder-style setter for inline editing (Phase 4).
    ///
    /// When `editable`, double-clicking a cell enters edit mode with an
    /// `<input>` element. Pressing Enter or blurring commits the change,
    /// firing `on_cell_edit` on the DataGrid. Pressing Escape cancels.
    ///
    /// # Example
    /// ```ignore
    /// ColumnDef::text("name", "Name", |r| r.name.clone())
    ///     .with_editable(true);
    /// ```
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
}

// ---------------------------------------------------------------------------
// Column Width
// ---------------------------------------------------------------------------

/// Determines how a column's width is computed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColumnWidth {
    /// Fixed pixel width, e.g. `ColumnWidth::Px(120)`.
    Px(u32),

    /// Fractional remaining space (similar to CSS `flex: <number>`).
    /// The total fr sum divides the remaining width after fixed columns.
    Fr(f32),

    /// Automatically fit the content width (uses header + sample cell text).
    Auto,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Auto
    }
}

// ---------------------------------------------------------------------------
// Cell Renderer
// ---------------------------------------------------------------------------

/// Determines how a cell value is formatted for display.
///
/// These renderers are stateless value formatters. For fully custom rendering
/// (e.g., action buttons inside a cell), use [`CellRenderer::Custom`].
#[derive(Clone)]
pub enum CellRenderer {
    /// Plain text — the exact string from `get_value`.
    Text,

    /// Numeric value with optional prefix and decimal places.
    Number {
        /// Prefix string (e.g. "Rs. ").
        prefix: &'static str,
        /// Number of decimal places.
        decimals: u8,
    },

    /// Currency formatting (prefix + thousands separator).
    Currency {
        /// Currency code for display (e.g. "PKR", "USD").
        code: &'static str,
        decimals: u8,
    },

    /// Date formatted with `chrono` format string.
    Date {
        /// `chrono` format string, e.g. `"%Y-%m-%d"`.
        format: &'static str,
    },

    /// DateTime formatted with `chrono` format string.
    DateTime {
        format: &'static str,
    },

    /// Colored badge (status indicator).
    Badge {
        /// Map of value → color variant.
        color_map: Vec<(&'static str, BadgeColor)>,
        /// Fallback color when value isn't in the map.
        default_color: BadgeColor,
    },

    /// Percentage (multiply by 100, append "%").
    Percentage {
        decimals: u8,
    },

    /// A custom renderer that receives the raw string value and returns an
    /// `Element`. Useful for action buttons, links, or rich markup.
    Custom(Rc<dyn Fn(&str) -> Element>),
}

// ---------------------------------------------------------------------------
// Supporting Enums
// ---------------------------------------------------------------------------

/// Text alignment within a cell.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Badge / status pill color.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BadgeColor {
    Gray,
    Green,
    Red,
    Yellow,
    Blue,
    Purple,
    Cyan,
}

/// Whether a column is pinned.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PinnedPosition {
    Left,
    Right,
}

/// Filter type for a column (placeholder for Phase 2+).
#[derive(Clone, PartialEq, Debug)]
pub enum FilterType {
    None,
    Text,
    Number,
    Date,
    Select { options: Vec<String> },
}

/// Dynamic cell class rule — applies a CSS class based on the row value.
///
/// # Example
/// ```ignore
/// CellClassRule::new(|item: &Item| {
///     if item.current_stock <= item.reorder_level {
///         "text-danger".to_string()
///     } else {
///         String::new()
///     }
/// })
/// ```
#[derive(Clone)]
pub struct CellClassRule<T> {
    pub class_fn: Rc<dyn Fn(&T) -> String>,
}

impl<T: 'static> CellClassRule<T> {
    pub fn new(class_fn: impl Fn(&T) -> String + 'static) -> Self {
        Self {
            class_fn: Rc::new(class_fn),
        }
    }
}

// ---------------------------------------------------------------------------
// Row Height
// ---------------------------------------------------------------------------

/// Row height presets (affects overall table density).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RowHeight {
    Compact,
    Standard,
    Comfortable,
}

impl RowHeight {
    pub fn px(&self) -> u32 {
        match self {
            Self::Compact => 32,
            Self::Standard => 40,
            Self::Comfortable => 48,
        }
    }
}

impl Default for RowHeight {
    fn default() -> Self {
        Self::Standard
    }
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

/// Row selection mode.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SelectionMode {
    None,
    Single,
    Multi,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::None
    }
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/// Pagination mode for the data grid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PaginationMode {
    /// No pagination — display all rows (use with virtual scroll for large sets).
    None,

    /// Client-side pagination with a configurable page size.
    /// The DataGrid processes filter → sort → paginate locally.
    Client { page_size: usize },

    /// Server-side pagination mode.
    /// The DataGrid does NOT filter, sort, or paginate locally. Instead it
    /// fires callbacks (`on_sort_change`, `on_filter_change`, and page
    /// events) so the parent can fetch the correct page from the server.
    /// Requires a `total_records` prop for accurate pagination bar display.
    Server { page_size: usize },
}

impl Default for PaginationMode {
    fn default() -> Self {
        Self::Client { page_size: 50 }
    }
}

impl PaginationMode {
    /// Returns the page size regardless of which variant.
    pub fn page_size(&self) -> usize {
        match self {
            Self::None => usize::MAX,
            Self::Client { page_size } | Self::Server { page_size } => *page_size,
        }
    }

    /// Returns `true` if this is a server-side mode.
    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server { .. })
    }
}

// ---------------------------------------------------------------------------
// Sort
// ---------------------------------------------------------------------------

/// A single sort column and direction.
#[derive(Clone, PartialEq, Debug)]
pub struct SortColumn {
    pub key: &'static str,
    pub direction: SortDirection,
}

impl SortColumn {
    pub fn ascending(key: &'static str) -> Self {
        Self {
            key,
            direction: SortDirection::Ascending,
        }
    }

    pub fn descending(key: &'static str) -> Self {
        Self {
            key,
            direction: SortDirection::Descending,
        }
    }
}

/// Sort direction.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

// ---------------------------------------------------------------------------
// Processed Row (internal)
// ---------------------------------------------------------------------------

/// A row that has passed through the filter/sort pipeline, carrying its
/// original index for selection tracking.
#[derive(Clone)]
pub(crate) struct IndexedRow<T> {
    pub index: usize,
    pub data: T,
}
