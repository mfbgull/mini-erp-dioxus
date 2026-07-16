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
    font-family: var(--font-family);
    font-size: var(--text-base);
    line-height: 1.4;
    color: var(--text-primary);
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius);
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
    background: var(--surface-secondary);
    border-bottom: 2px solid var(--border-color);
    position: sticky;
    top: 0;
    z-index: 10;
    min-height: 40px;
}

.dg-header-cell {
    display: flex;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    font-weight: 600;
    font-size: var(--text-sm);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-secondary);
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background-color var(--ease-fast);
}

.dg-header-cell.dg-sortable {
    cursor: pointer;
}

.dg-header-cell.dg-sortable:hover {
    background: var(--surface-tertiary);
}

.dg-header-cell.dg-sortable:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
}

.dg-header-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
}

.dg-sort-indicator {
    margin-left: 4px;
    font-size: var(--text-xs);
    color: var(--text-muted);
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
    border-bottom: 1px solid var(--border-light);
    transition: background-color var(--ease-fast);
}

.dg-row:hover {
    background: var(--surface-tertiary);
}

.dg-row-even {
    background: var(--surface-secondary);
}

.dg-row-even:hover {
    background: var(--surface-tertiary);
}

.dg-row-selected {
    background: var(--accent-light) !important;
    box-shadow: inset 3px 0 0 0 var(--accent);
}

/* ---- Cells ---- */

.dg-cell {
    display: flex;
    align-items: center;
    padding: var(--space-1) var(--space-3);
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
    color: var(--text-secondary);
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
    background: var(--surface-tertiary);
    color: var(--text-secondary);
}

.dg-badge-green {
    background: var(--success-light);
    color: var(--success);
}

.dg-badge-red {
    background: var(--danger-light);
    color: var(--danger);
}

.dg-badge-yellow {
    background: var(--warning-light);
    color: var(--warning);
}

.dg-badge-blue {
    background: rgba(59, 130, 246, 0.10);
    color: var(--info);
}

.dg-badge-purple {
    background: rgba(139, 92, 246, 0.10);
    color: #7C3AED;
}

.dg-badge-cyan {
    background: rgba(6, 182, 212, 0.10);
    color: #0891B2;
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
    border: 3px solid var(--border-color);
    border-top: 3px solid var(--accent);
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
    border-bottom: 1px solid var(--border-light);
    gap: 12px;
}

.dg-skeleton {
    background: linear-gradient(90deg, var(--border-color) 25%, var(--surface-secondary) 50%, var(--border-color) 75%);
    background-size: 200% 100%;
    animation: dg-shimmer 1.5s infinite;
    border-radius: var(--radius-sm);
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
    padding: var(--space-12) var(--space-6);
    color: var(--text-muted);
    gap: var(--space-2);
}

.dg-empty-state-icon {
    font-size: 36px;
    opacity: 0.3;
}

.dg-empty-state-text {
    font-size: 15px;
    font-weight: 500;
}

.dg-empty-state-hint {
    font-size: var(--text-sm);
    opacity: 0.7;
}

/* ---- Pagination ---- */

.dg-pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    border-top: 1px solid var(--border-color);
    background: var(--surface-secondary);
    font-size: var(--text-sm);
    flex-wrap: wrap;
    gap: var(--space-2);
}

.dg-page-size-label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
}

.dg-page-size-select {
    padding: 4px 8px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    background: var(--surface);
    cursor: pointer;
    color: var(--text-primary);
    font-family: inherit;
}

.dg-pagination-summary {
    color: var(--text-secondary);
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
    padding: 0 var(--space-2);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--ease-fast);
    font-family: inherit;
}

.dg-page-btn:hover:not(:disabled) {
    background: var(--surface-tertiary);
    border-color: var(--border-color);
}

.dg-page-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.dg-page-active {
    background: var(--accent) !important;
    color: white !important;
    border-color: var(--accent) !important;
    font-weight: 600;
}

.dg-page-ellipsis {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    color: var(--text-muted);
    font-size: var(--text-base);
}

/* ---- Filter Bar ---- */

.dg-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-1) var(--space-4);
    background: var(--warning-light);
    border-bottom: 1px solid rgba(245, 158, 11, 0.25);
    font-size: var(--text-sm);
}

.dg-filter-bar-label {
    color: var(--warning);
    font-weight: 600;
}

.dg-filter-bar-clear-all {
    padding: 2px 10px;
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.6);
    color: var(--warning);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--ease-fast);
    font-family: inherit;
}

.dg-filter-bar-clear-all:hover {
    background: var(--warning-light);
    border-color: var(--warning);
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
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    position: relative;
    transition: all var(--ease-fast);
}

.dg-filter-btn:hover {
    background: var(--surface-tertiary);
    color: var(--text-secondary);
}

.dg-filter-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
}

.dg-filter-btn.dg-filter-active {
    color: var(--accent);
}

.dg-filter-btn.dg-filter-active:hover {
    background: var(--accent-light);
}

.dg-filter-btn.dg-filter-open {
    background: var(--surface-tertiary);
    color: var(--accent);
}

.dg-filter-dot {
    position: absolute;
    top: 1px;
    right: 1px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    border: 1.5px solid var(--surface-secondary);
}

.dg-has-active-filter .dg-header-text {
    color: var(--accent);
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
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: var(--space-2) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: var(--text-sm);
}

.dg-filter-input {
    width: 100%;
    padding: 6px 10px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-family: inherit;
    color: var(--text-primary);
    background: var(--surface);
    transition: border-color var(--ease-fast);
    box-sizing: border-box;
}

.dg-filter-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-ring);
}

.dg-filter-input::placeholder {
    color: var(--text-muted);
}

/* Number & Date range layout */
.dg-filter-range {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.dg-filter-label {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
}

/* Clear button inside dropdown */
.dg-filter-clear-btn {
    align-self: flex-end;
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text-muted);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--ease-fast);
    font-family: inherit;
}

.dg-filter-clear-btn:hover {
    background: var(--danger-light);
    border-color: var(--danger);
    color: var(--danger);
}

/* ---- Select Filter ---- */

.dg-filter-select-dropdown {
    min-width: 200px;
}

.dg-filter-select-actions {
    display: flex;
    gap: var(--space-2);
    border-bottom: 1px solid var(--border-light);
    padding-bottom: var(--space-2);
}

.dg-filter-select-action {
    padding: 2px 8px;
    border: 1px solid var(--border-color);
    border-radius: 3px;
    background: var(--surface-secondary);
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--ease-fast);
    font-family: inherit;
}

.dg-filter-select-action:hover {
    background: var(--surface-tertiary);
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
    background: var(--surface-tertiary);
}

.dg-filter-select-option input[type="checkbox"] {
    margin: 0;
    accent-color: var(--accent);
}

/* ---- Focus & Accessibility ---- */

.dg-row:focus-visible,
.dg-page-btn:focus-visible {
    outline: 2px solid var(--accent);
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
    background: var(--surface-tertiary);
    border-radius: var(--radius-sm);
}

.dg-virtual-body::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: var(--radius-sm);
}

.dg-virtual-body::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
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
    background: var(--surface-tertiary);
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
    transition: background-color var(--ease-fast);
}

.dg-resize-handle:hover,
.dg-resize-handle:active {
    background: var(--accent);
    width: 4px;
}

.dg-resize-handle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    background: var(--accent);
    width: 4px;
}

/* The header cell that is being resized gets a highlight */
.dg-header-cell.dg-resizing {
    background: var(--surface-tertiary);
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
    border: 2px solid var(--accent);
    border-radius: var(--radius-sm);
    font-size: var(--text-base);
    font-family: inherit;
    color: var(--text-primary);
    background: var(--surface);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-ring);
    box-sizing: border-box;
    min-height: 28px;
}

.dg-cell-editor:focus {
    border-color: var(--accent-hover);
    box-shadow: 0 0 0 3px var(--accent-ring);
}

/* ---- Column Visibility Toggle (Phase 5) ---- */

.dg-column-toggle-bar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: var(--space-1) var(--space-3);
    background: var(--surface-secondary);
    border-bottom: 1px solid var(--border-color);
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
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all var(--ease-fast);
    user-select: none;
    white-space: nowrap;
}

.dg-column-toggle-btn:hover {
    background: var(--surface-tertiary);
    border-color: var(--text-muted);
}

.dg-column-toggle-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
}

.dg-column-toggle-btn.dg-column-toggle-open {
    background: var(--surface-tertiary);
    border-color: var(--accent);
    color: var(--accent);
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
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 6px 0;
    margin-top: var(--space-1);
}

.dg-column-toggle-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 7px 14px;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-primary);
    transition: background-color var(--ease-fast);
    user-select: none;
}

.dg-column-toggle-option:hover {
    background: var(--surface-tertiary);
}

.dg-column-toggle-option input[type="checkbox"] {
    margin: 0;
    accent-color: var(--accent);
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
    background: var(--border-color);
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
        gap: var(--space-2);
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
