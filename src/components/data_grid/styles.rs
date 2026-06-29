//! CSS styles for the `DataGrid<T>` component.
//!
//! This module exports a static CSS string that can be included once in the
//! application root. All DataGrid class names use the `dg-` prefix to avoid
//! collisions with application styles.
//!
//! # Usage
//! ```ignore
//! // In main.rs or app root component:
//! rsx! {
//!     style { {DATA_GRID_CSS} }
//!     // ... rest of the app
//! }
//! ```

// ---------------------------------------------------------------------------
// Class name constants
// ---------------------------------------------------------------------------

pub const CONTAINER: &str = "dg-container";
pub const TABLE_WRAPPER: &str = "dg-table-wrapper";
pub const HEADER_ROW: &str = "dg-header-row";
pub const HEADER_CELL: &str = "dg-header-cell";
pub const HEADER_TEXT: &str = "dg-header-text";
pub const SORT_INDICATOR: &str = "dg-sort-indicator";
pub const SORT_INACTIVE: &str = "dg-sort-inactive";
pub const SORTABLE: &str = "dg-sortable";
pub const BODY: &str = "dg-body";
pub const ROW: &str = "dg-row";
pub const ROW_EVEN: &str = "dg-row-even";
pub const ROW_SELECTED: &str = "dg-row-selected";
pub const CELL: &str = "dg-cell";
pub const CELL_ALIGN_LEFT: &str = "dg-align-left";
pub const CELL_ALIGN_CENTER: &str = "dg-align-center";
pub const CELL_ALIGN_RIGHT: &str = "dg-align-right";
pub const LOADING_OVERLAY: &str = "dg-loading-overlay";
pub const LOADING_SPINNER: &str = "dg-loading-spinner";
pub const EMPTY_STATE: &str = "dg-empty-state";
pub const SKELETON: &str = "dg-skeleton";
pub const SKELETON_ROW: &str = "dg-skeleton-row";
pub const PAGINATION: &str = "dg-pagination";

// ---------------------------------------------------------------------------
// CSS stylesheet
// ---------------------------------------------------------------------------

/// Default stylesheet for the DataGrid component.
///
/// Include this once in your application root via a `<style>` tag.
pub const DATA_GRID_CSS: &str = r##"
/* ================================
   DataGrid (dg-) Component Styles
   ================================ */

.dg-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    line-height: 1.4;
    color: #1a1a2e;
    background: #ffffff;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    overflow: hidden;
}

.dg-table-wrapper {
    overflow-x: auto;
    overflow-y: auto;
    position: relative;
}

/* ---- Header ---- */

.dg-header-row {
    display: flex;
    background: #f8f9fa;
    border-bottom: 2px solid #e0e0e0;
    position: sticky;
    top: 0;
    z-index: 10;
    min-height: 40px;
}

.dg-header-cell {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    font-weight: 600;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: #495057;
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background-color 0.15s ease;
}

.dg-header-cell.dg-sortable {
    cursor: pointer;
}

.dg-header-cell.dg-sortable:hover {
    background: #e9ecef;
}

.dg-header-cell.dg-sortable:focus-visible {
    outline: 2px solid #4a90d9;
    outline-offset: -2px;
}

.dg-header-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
}

.dg-sort-indicator {
    margin-left: 4px;
    font-size: 11px;
    color: #6c757d;
    flex-shrink: 0;
}

.dg-sort-indicator.dg-sort-inactive {
    opacity: 0.3;
}

.dg-sortable:hover .dg-sort-inactive {
    opacity: 0.7;
}

/* ---- Body ---- */

.dg-body {
    position: relative;
    min-height: 100px;
}

.dg-row {
    display: flex;
    border-bottom: 1px solid #f0f0f0;
    transition: background-color 0.12s ease;
}

.dg-row:hover {
    background: #f8f9ff;
}

.dg-row-even {
    background: #fafbfc;
}

.dg-row-even:hover {
    background: #f0f2f5;
}

.dg-row-selected {
    background: #e8f0fe !important;
    box-shadow: inset 3px 0 0 0 #4a90d9;
}

/* ---- Cells ---- */

.dg-cell {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-height: 32px;
}

.dg-align-left {
    justify-content: flex-start;
    text-align: left;
}

.dg-align-center {
    justify-content: center;
    text-align: center;
}

.dg-align-right {
    justify-content: flex-end;
    text-align: right;
}

/* ---- Text Renderer ---- */

.dg-cell-text {
    overflow: hidden;
    text-overflow: ellipsis;
}

/* ---- Number / Currency ---- */

.dg-cell-number {
    font-variant-numeric: tabular-nums;
    font-weight: 500;
}

.dg-cell-currency {
    font-variant-numeric: tabular-nums;
}

.dg-cell-currency-code {
    font-size: 11px;
    opacity: 0.6;
    margin-right: 2px;
}

.dg-cell-currency-value {
    font-weight: 500;
}

/* ---- Date ---- */

.dg-cell-date,
.dg-cell-datetime {
    font-variant-numeric: tabular-nums;
    color: #495057;
}

/* ---- Percentage ---- */

.dg-cell-percentage {
    font-variant-numeric: tabular-nums;
    font-weight: 500;
}

/* ---- Badge ---- */

.dg-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 10px;
    border-radius: 100px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.02em;
    white-space: nowrap;
    text-transform: capitalize;
}

.dg-badge-gray {
    background: #e9ecef;
    color: #495057;
}

.dg-badge-green {
    background: #d4edda;
    color: #155724;
}

.dg-badge-red {
    background: #f8d7da;
    color: #721c24;
}

.dg-badge-yellow {
    background: #fff3cd;
    color: #856404;
}

.dg-badge-blue {
    background: #cce5ff;
    color: #004085;
}

.dg-badge-purple {
    background: #e8daef;
    color: #6c3483;
}

.dg-badge-cyan {
    background: #d1ecf1;
    color: #0c5460;
}

/* ---- Loading State ---- */

.dg-loading-overlay {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px 0;
}

.dg-loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #e0e0e0;
    border-top: 3px solid #4a90d9;
    border-radius: 50%;
    animation: dg-spin 0.8s linear infinite;
}

@keyframes dg-spin {
    to { transform: rotate(360deg); }
}

/* ---- Skeleton Loading ---- */

.dg-skeleton-row {
    display: flex;
    padding: 8px 12px;
    border-bottom: 1px solid #f0f0f0;
    gap: 12px;
}

.dg-skeleton {
    background: linear-gradient(90deg, #e9ecef 25%, #f8f9fa 50%, #e9ecef 75%);
    background-size: 200% 100%;
    animation: dg-shimmer 1.5s infinite;
    border-radius: 4px;
    height: 16px;
}

@keyframes dg-shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
}

/* ---- Empty State ---- */

.dg-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    color: #6c757d;
    gap: 8px;
}

.dg-empty-state-icon {
    font-size: 36px;
    opacity: 0.4;
}

.dg-empty-state-text {
    font-size: 15px;
    font-weight: 500;
}

.dg-empty-state-hint {
    font-size: 13px;
    opacity: 0.7;
}

/* ---- Pagination ---- */

.dg-pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-top: 1px solid #e0e0e0;
    background: #f8f9fa;
    font-size: 13px;
    flex-wrap: wrap;
    gap: 8px;
}

.dg-page-size-label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #495057;
}

.dg-page-size-select {
    padding: 4px 8px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 13px;
    background: white;
    cursor: pointer;
}

.dg-pagination-summary {
    color: #495057;
    font-variant-numeric: tabular-nums;
}

.dg-pagination-controls {
    display: flex;
    align-items: center;
    gap: 2px;
}

.dg-page-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    height: 32px;
    padding: 0 8px;
    border: 1px solid transparent;
    border-radius: 4px;
    background: transparent;
    color: #495057;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
}

.dg-page-btn:hover:not(:disabled) {
    background: #e9ecef;
    border-color: #ced4da;
}

.dg-page-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.dg-page-active {
    background: #4a90d9 !important;
    color: white !important;
    border-color: #4a90d9 !important;
    font-weight: 600;
}

.dg-page-ellipsis {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    color: #6c757d;
    font-size: 14px;
}

/* ---- Filter Bar ---- */

.dg-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
    background: #fff8e1;
    border-bottom: 1px solid #ffe082;
    font-size: 13px;
}

.dg-filter-bar-label {
    color: #856404;
    font-weight: 600;
}

.dg-filter-bar-clear-all {
    padding: 2px 10px;
    border: 1px solid #ffe082;
    border-radius: 4px;
    background: #fffde7;
    color: #856404;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s ease;
}

.dg-filter-bar-clear-all:hover {
    background: #fff3cd;
    border-color: #ffca28;
}

/* ---- Filter Button (in header) ---- */

.dg-filter-btn-wrapper {
    display: flex;
    align-items: center;
    margin-left: 4px;
    flex-shrink: 0;
    position: relative;
}

.dg-filter-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: #adb5bd;
    cursor: pointer;
    position: relative;
    transition: all 0.15s ease;
}

.dg-filter-btn:hover {
    background: #dee2e6;
    color: #495057;
}

.dg-filter-btn:focus-visible {
    outline: 2px solid #4a90d9;
    outline-offset: 1px;
}

.dg-filter-btn.dg-filter-active {
    color: #4a90d9;
}

.dg-filter-btn.dg-filter-active:hover {
    background: #cce5ff;
}

.dg-filter-btn.dg-filter-open {
    background: #e9ecef;
    color: #4a90d9;
}

.dg-filter-dot {
    position: absolute;
    top: 1px;
    right: 1px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #4a90d9;
    border: 1.5px solid #f8f9fa;
}

.dg-has-active-filter .dg-header-text {
    color: #4a90d9;
}

/* ---- Filter Dropdown ---- */

.dg-filter-dropdown-wrapper {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    padding-top: 4px;
}

.dg-filter-dropdown {
    min-width: 220px;
    background: #ffffff;
    border: 1px solid #ced4da;
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
}

.dg-filter-input {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
    color: #1a1a2e;
    background: #ffffff;
    transition: border-color 0.15s ease;
    box-sizing: border-box;
}

.dg-filter-input:focus {
    outline: none;
    border-color: #4a90d9;
    box-shadow: 0 0 0 2px rgba(74, 144, 217, 0.2);
}

.dg-filter-input::placeholder {
    color: #adb5bd;
}

/* Number & Date range layout */
.dg-filter-range {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.dg-filter-label {
    font-size: 11px;
    font-weight: 600;
    color: #6c757d;
    text-transform: uppercase;
    letter-spacing: 0.04em;
}

/* Clear button inside dropdown */
.dg-filter-clear-btn {
    align-self: flex-end;
    padding: 4px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    background: #ffffff;
    color: #6c757d;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s ease;
}

.dg-filter-clear-btn:hover {
    background: #f8d7da;
    border-color: #dc3545;
    color: #dc3545;
}

/* ---- Select Filter ---- */

.dg-filter-select-dropdown {
    min-width: 200px;
}

.dg-filter-select-actions {
    display: flex;
    gap: 8px;
    border-bottom: 1px solid #e9ecef;
    padding-bottom: 8px;
}

.dg-filter-select-action {
    padding: 2px 8px;
    border: 1px solid #ced4da;
    border-radius: 3px;
    background: #f8f9fa;
    color: #495057;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s ease;
}

.dg-filter-select-action:hover {
    background: #e9ecef;
}

.dg-filter-select-options {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 200px;
    overflow-y: auto;
}

.dg-filter-select-option {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 4px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 13px;
    transition: background-color 0.1s ease;
}

.dg-filter-select-option:hover {
    background: #f0f2f5;
}

.dg-filter-select-option input[type="checkbox"] {
    margin: 0;
    accent-color: #4a90d9;
}

/* ---- Focus & Accessibility ---- */

.dg-row:focus-visible,
.dg-page-btn:focus-visible {
    outline: 2px solid #4a90d9;
    outline-offset: -2px;
}

/* ---- Responsive ---- */

/* ---- Virtual Scroll Body ---- */

.dg-virtual-body {
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
}

.dg-virtual-body::-webkit-scrollbar {
    width: 8px;
}

.dg-virtual-body::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 4px;
}

.dg-virtual-body::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 4px;
}

.dg-virtual-body::-webkit-scrollbar-thumb:hover {
    background: #a1a1a1;
}

.dg-vspacer {
    flex-shrink: 0;
    pointer-events: none;
}

/* ---- Pinned (Frozen) Columns ---- */

.dg-pinned-left {
    box-shadow: 2px 0 4px rgba(0, 0, 0, 0.08);
}

.dg-pinned-right {
    box-shadow: -2px 0 4px rgba(0, 0, 0, 0.08);
}

.dg-header-cell.dg-pinned-left,
.dg-header-cell.dg-pinned-right {
    background: #f0f2f5;
}

/* ---- Column Resize Handle ---- */

.dg-resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 5;
    background: transparent;
    transition: background-color 0.12s ease;
}

.dg-resize-handle:hover,
.dg-resize-handle:active {
    background: #4a90d9;
    width: 4px;
}

.dg-resize-handle:focus-visible {
    outline: 2px solid #4a90d9;
    outline-offset: -2px;
    background: #4a90d9;
    width: 4px;
}

/* The header cell that is being resized gets a highlight */
.dg-header-cell.dg-resizing {
    background: #e9ecef;
    user-select: none;
}

/* Resize overlay (full-screen capture during drag) */
.dg-resize-overlay {
    cursor: col-resize;
}

/* ---- Inline Cell Editing (Phase 4) ---- */

.dg-cell-editing {
    padding: 0 !important;
    overflow: visible !important;
}

.dg-cell-editor {
    width: 100%;
    height: 100%;
    padding: 4px 10px;
    border: 2px solid #4a90d9;
    border-radius: 4px;
    font-size: 14px;
    font-family: inherit;
    color: #1a1a2e;
    background: #ffffff;
    outline: none;
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.2);
    box-sizing: border-box;
    min-height: 28px;
}

.dg-cell-editor:focus {
    border-color: #357abd;
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.3);
}

/* ---- Column Visibility Toggle (Phase 5) ---- */

.dg-column-toggle-bar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 4px 12px;
    background: #f8f9fa;
    border-bottom: 1px solid #e0e0e0;
    min-height: 32px;
    position: relative;
}

.dg-column-toggle-spacer {
    flex: 1;
}

.dg-column-toggle-btn-wrapper {
    position: relative;
}

.dg-column-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    background: #ffffff;
    color: #495057;
    font-size: 13px;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.15s ease;
    user-select: none;
    white-space: nowrap;
}

.dg-column-toggle-btn:hover {
    background: #e9ecef;
    border-color: #adb5bd;
}

.dg-column-toggle-btn:focus-visible {
    outline: 2px solid #4a90d9;
    outline-offset: 1px;
}

.dg-column-toggle-btn.dg-column-toggle-open {
    background: #e9ecef;
    border-color: #4a90d9;
    color: #4a90d9;
}

.dg-column-toggle-icon {
    font-size: 14px;
    line-height: 1;
}

.dg-column-toggle-arrow {
    font-size: 10px;
    margin-left: 2px;
    opacity: 0.6;
}

.dg-column-toggle-menu {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 200;
    min-width: 200px;
    max-height: 320px;
    overflow-y: auto;
    background: #ffffff;
    border: 1px solid #ced4da;
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
    padding: 6px 0;
    margin-top: 4px;
}

.dg-column-toggle-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 14px;
    cursor: pointer;
    font-size: 13px;
    color: #1a1a2e;
    transition: background-color 0.1s ease;
    user-select: none;
}

.dg-column-toggle-option:hover {
    background: #f0f2f5;
}

.dg-column-toggle-option input[type="checkbox"] {
    margin: 0;
    accent-color: #4a90d9;
    flex-shrink: 0;
}

.dg-column-toggle-option span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* Show a subtle scrollbar in the column toggle menu */
.dg-column-toggle-menu::-webkit-scrollbar {
    width: 6px;
}

.dg-column-toggle-menu::-webkit-scrollbar-track {
    background: transparent;
}

.dg-column-toggle-menu::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 3px;
}

/* ---- Responsive ---- */

@media (max-width: 768px) {
    .dg-container {
        border-radius: 0;
        border-left: none;
        border-right: none;
    }

    .dg-pagination {
        flex-direction: column;
        gap: 8px;
    }

    .dg-resize-handle {
        display: none; /* Resize is impractical on touch devices */
    }

    .dg-column-toggle-menu {
        min-width: 160px;
        right: -8px;
    }
}
"##;
