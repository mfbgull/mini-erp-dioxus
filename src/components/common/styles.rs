//! CSS styles for the common UI component library.
//!
//! Every component in this module uses a `.cb-*` (codebuff/mini-erp) prefix
//! to avoid collisions with other CSS in the application. All styles are
//! embedded in a single `&'static str` injected by `<style>` in the root.

/// Global CSS string injected into the application `<style>` tag.
pub const COMMON_CSS: &str = r##"
/* ========================================================================
   MiniERP Common UI Components — Styles
   Prefix: .cb-  (codebuff/mini-erp)
   ======================================================================== */

/* -----------------------------------------------------------------------
   Button — .cb-btn
   ----------------------------------------------------------------------- */
.cb-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    font-family: var(--font-family);
    font-weight: 500;
    font-size: 14px;
    line-height: 1.4;
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
    user-select: none;
    text-decoration: none;
    position: relative;
    overflow: hidden;
}

.cb-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
}

.cb-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
}

/* Sizes */
.cb-btn-sm { padding: 4px 10px; font-size: 12px; border-radius: 3px; }
.cb-btn-md { padding: 8px 18px; font-size: 14px; }
.cb-btn-lg { padding: 12px 24px; font-size: 16px; border-radius: 6px; }

/* Variants */
.cb-btn-primary {
    background: var(--accent, #4a90d9);
    color: #ffffff;
    border-color: var(--accent, #4a90d9);
}
.cb-btn-primary:hover:not(:disabled) {
    background: #357abd;
    border-color: #357abd;
}
.cb-btn-primary:active:not(:disabled) {
    background: #2a6fb5;
}

.cb-btn-secondary {
    background: #ffffff;
    color: var(--text-primary, #1a1a2e);
    border-color: var(--border-color, #e0e0e0);
}
.cb-btn-secondary:hover:not(:disabled) {
    background: #f0f2f5;
    border-color: #ced4da;
}

.cb-btn-danger {
    background: var(--danger, #dc3545);
    color: #ffffff;
    border-color: var(--danger, #dc3545);
}
.cb-btn-danger:hover:not(:disabled) {
    background: #c82333;
    border-color: #c82333;
}

.cb-btn-ghost {
    background: transparent;
    color: var(--text-primary, #1a1a2e);
    border-color: transparent;
}
.cb-btn-ghost:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.05);
}

.cb-btn-success {
    background: var(--success, #28a745);
    color: #ffffff;
    border-color: var(--success, #28a745);
}
.cb-btn-success:hover:not(:disabled) {
    background: #218838;
    border-color: #218838;
}

.cb-btn-warning {
    background: var(--warning, #ffc107);
    color: #1a1a2e;
    border-color: var(--warning, #ffc107);
}
.cb-btn-warning:hover:not(:disabled) {
    background: #e0a800;
    border-color: #e0a800;
}

/* Full width */
.cb-btn-block {
    width: 100%;
}

/* Loading spinner */
.cb-btn-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #ffffff;
    border-radius: 50%;
    animation: cb-spin 0.6s linear infinite;
    flex-shrink: 0;
}

.cb-btn-spinner-dark {
    border-color: rgba(0, 0, 0, 0.15);
    border-top-color: var(--text-primary);
}

@keyframes cb-spin {
    to { transform: rotate(360deg); }
}

/* -----------------------------------------------------------------------
   Form Input — .cb-input-group
   ----------------------------------------------------------------------- */
.cb-input-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.cb-input-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #1a1a2e);
    display: flex;
    align-items: center;
    gap: 4px;
}

.cb-input-required {
    color: var(--danger, #dc3545);
    font-size: 14px;
}

.cb-input-hint {
    font-size: 12px;
    color: var(--text-secondary, #6c757d);
    font-weight: 400;
}

.cb-input {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius-sm, 4px);
    background: #ffffff;
    font-family: var(--font-family);
    font-size: 14px;
    color: var(--text-primary, #1a1a2e);
    transition: all 0.15s ease;
}

.cb-input:hover {
    border-color: #b0b0b0;
}

.cb-input:focus-within {
    border-color: var(--accent, #4a90d9);
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.15);
    outline: none;
}

.cb-input input,
.cb-input select {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    min-width: 0;
}

.cb-input input::placeholder {
    color: #adb5bd;
}

.cb-input input[type="number"] {
    -moz-appearance: textfield;
}
.cb-input input[type="number"]::-webkit-inner-spin-button,
.cb-input input[type="number"]::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
}

.cb-input-icon {
    color: var(--text-secondary, #6c757d);
    font-size: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
}

.cb-input-error {
    border-color: var(--danger, #dc3545);
}
.cb-input-error:focus-within {
    box-shadow: 0 0 0 3px rgba(220, 53, 69, 0.15);
}

.cb-input-error-text {
    font-size: 12px;
    color: var(--danger, #dc3545);
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 4px;
}

/* Textarea variant */
.cb-input-textarea {
    padding: 10px 12px;
    min-height: 80px;
    resize: vertical;
    line-height: 1.5;
}

.cb-input-textarea textarea {
    border: none;
    outline: none;
    background: transparent;
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    width: 100%;
    min-height: 60px;
    resize: vertical;
    line-height: inherit;
}

/* -----------------------------------------------------------------------
   Modal — .cb-modal-*
   ----------------------------------------------------------------------- */
.cb-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 24px;
    animation: cb-fade-in 0.15s ease;
}

.cb-modal {
    background: #ffffff;
    border-radius: var(--radius, 8px);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    max-height: 85vh;
    width: 100%;
    animation: cb-slide-up 0.2s ease;
}

.cb-modal-sm { max-width: 400px; }
.cb-modal-md { max-width: 560px; }
.cb-modal-lg { max-width: 720px; }
.cb-modal-xl { max-width: 960px; }
.cb-modal-full { max-width: calc(100vw - 48px); }

.cb-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    flex-shrink: 0;
}

.cb-modal-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #1a1a2e);
}

.cb-modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 4px;
    color: var(--text-secondary, #6c757d);
    font-size: 18px;
    transition: all 0.15s ease;
}

.cb-modal-close:hover {
    background: rgba(0, 0, 0, 0.06);
    color: var(--text-primary);
}

.cb-modal-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
}

.cb-modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    flex-shrink: 0;
}

@keyframes cb-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes cb-slide-up {
    from { opacity: 0; transform: translateY(16px); }
    to { opacity: 1; transform: translateY(0); }
}

/* Mobile: bottom sheet */
@media (max-width: 768px) {
    .cb-modal-overlay {
        align-items: flex-end;
        padding: 0;
    }

    .cb-modal {
        max-width: 100%;
        max-height: 90vh;
        border-radius: var(--radius, 8px) var(--radius, 8px) 0 0;
        animation: cb-slide-up-mobile 0.25s ease;
    }

    .cb-modal-sm,
    .cb-modal-md,
    .cb-modal-lg,
    .cb-modal-xl { max-width: 100%; }
}

@keyframes cb-slide-up-mobile {
    from { transform: translateY(100%); }
    to { transform: translateY(0); }
}

/* -----------------------------------------------------------------------
   Toast — .cb-toast-*
   ----------------------------------------------------------------------- */
.cb-toast-container {
    position: fixed;
    top: 16px;
    right: 16px;
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
    max-width: 400px;
    width: 100%;
}

.cb-toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    background: #ffffff;
    border-radius: var(--radius, 8px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
    border-left: 4px solid var(--accent, #4a90d9);
    pointer-events: auto;
    animation: cb-toast-in 0.25s ease;
    min-width: 280px;
}

.cb-toast-exit {
    animation: cb-toast-out 0.2s ease forwards;
}

.cb-toast-icon {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    margin-top: 1px;
}

.cb-toast-content {
    flex: 1;
    min-width: 0;
}

.cb-toast-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #1a1a2e);
    margin-bottom: 2px;
}

.cb-toast-message {
    font-size: 13px;
    color: var(--text-secondary, #6c757d);
    line-height: 1.4;
}

.cb-toast-close {
    flex-shrink: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--text-secondary, #6c757d);
    font-size: 16px;
    padding: 2px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background 0.15s;
}

.cb-toast-close:hover {
    background: rgba(0, 0, 0, 0.06);
}

/* Toast type colors */
.cb-toast-success { border-left-color: var(--success, #28a745); }
.cb-toast-error { border-left-color: var(--danger, #dc3545); }
.cb-toast-warning { border-left-color: var(--warning, #ffc107); }
.cb-toast-info { border-left-color: var(--accent, #4a90d9); }

@keyframes cb-toast-in {
    from { opacity: 0; transform: translateX(40px); }
    to { opacity: 1; transform: translateX(0); }
}

@keyframes cb-toast-out {
    from { opacity: 1; transform: translateX(0); }
    to { opacity: 0; transform: translateX(40px); }
}

@media (max-width: 768px) {
    .cb-toast-container {
        top: 8px;
        right: 8px;
        left: 8px;
        max-width: none;
    }
}

/* -----------------------------------------------------------------------
   Searchable Select — .cb-select-*
   ----------------------------------------------------------------------- */
.cb-select-wrapper {
    position: relative;
}

.cb-select-input {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius-sm, 4px);
    background: #ffffff;
    cursor: pointer;
    min-height: 38px;
    transition: border-color 0.15s;
}

.cb-select-input:hover { border-color: #b0b0b0; }
.cb-select-input:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(74,144,217,0.15); }

.cb-select-value {
    flex: 1;
    font-size: 14px;
    color: var(--text-primary);
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.cb-select-placeholder {
    flex: 1;
    font-size: 14px;
    color: #adb5bd;
}

.cb-select-chevron {
    color: var(--text-secondary);
    font-size: 12px;
    transition: transform 0.15s;
    flex-shrink: 0;
}

.cb-select-chevron-open {
    transform: rotate(180deg);
}

.cb-select-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: #ffffff;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.12);
    z-index: 500;
    max-height: 260px;
    display: flex;
    flex-direction: column;
    animation: cb-fade-in 0.1s ease;
}

.cb-select-search {
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
}

.cb-select-search input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--border-color);
    border-radius: 3px;
    font-size: 13px;
    outline: none;
    font-family: inherit;
}

.cb-select-search input:focus {
    border-color: var(--accent);
}

.cb-select-options {
    overflow-y: auto;
    flex: 1;
    padding: 4px 0;
}

.cb-select-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
    transition: background 0.1s;
}

.cb-select-option:hover,
.cb-select-option-highlighted {
    background: #f0f2f5;
}

.cb-select-option-selected {
    background: #e8f0fe;
    color: var(--accent);
    font-weight: 500;
}

.cb-select-option-disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.cb-select-no-results {
    padding: 16px 12px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 13px;
}

.cb-select-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: #e8f0fe;
    border-radius: 100px;
    font-size: 12px;
    color: #004085;
}

.cb-select-tag-remove {
    border: none;
    background: transparent;
    cursor: pointer;
    color: #004085;
    font-size: 14px;
    padding: 0;
    line-height: 1;
    opacity: 0.6;
}

.cb-select-tag-remove:hover { opacity: 1; }

/* -----------------------------------------------------------------------
   Dropdown Menu — .cb-dropdown-*
   ----------------------------------------------------------------------- */
.cb-dropdown-wrapper {
    position: relative;
    display: inline-flex;
}

.cb-dropdown-menu {
    position: absolute;
    background: #ffffff;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.12);
    z-index: 500;
    min-width: 160px;
    padding: 4px 0;
    animation: cb-fade-in 0.1s ease;
}

.cb-dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
    transition: background 0.1s;
    border: none;
    background: transparent;
    width: 100%;
    text-align: left;
    font-family: inherit;
    white-space: nowrap;
}

.cb-dropdown-item:hover {
    background: #f0f2f5;
}

.cb-dropdown-item-danger {
    color: var(--danger);
}
.cb-dropdown-item-danger:hover {
    background: #fff0f0;
}

.cb-dropdown-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
}

.cb-dropdown-item-icon {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    font-size: 14px;
}

.cb-dropdown-item-label {
    flex: 1;
}

.cb-dropdown-item-shortcut {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: var(--font-mono, monospace);
}

/* Positions */
.cb-dropdown-bottom-left { top: calc(100% + 4px); left: 0; }
.cb-dropdown-bottom-right { top: calc(100% + 4px); right: 0; }
.cb-dropdown-top-left { bottom: calc(100% + 4px); left: 0; }
.cb-dropdown-top-right { bottom: calc(100% + 4px); right: 0; }

/* -----------------------------------------------------------------------
   Date Range Picker — .cb-daterange-*
   ----------------------------------------------------------------------- */
.cb-daterange {
    display: flex;
    align-items: center;
    gap: 8px;
}

.cb-daterange-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius-sm, 4px);
    background: #ffffff;
    font-family: var(--font-family);
    font-size: 13px;
    color: var(--text-primary);
    transition: border-color 0.15s;
    min-width: 0;
}

.cb-daterange-input:focus {
    outline: none;
    border-color: var(--accent, #4a90d9);
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.15);
}

.cb-daterange-separator {
    color: var(--text-secondary);
    font-size: 13px;
    flex-shrink: 0;
}

.cb-daterange-error {
    font-size: 12px;
    color: var(--danger);
    margin-top: 4px;
}

@media (max-width: 768px) {
    .cb-daterange {
        flex-direction: column;
        align-items: stretch;
    }
    .cb-daterange-separator {
        align-self: center;
    }
}

/* -----------------------------------------------------------------------
   Stat Card — .cb-stat-*
   ----------------------------------------------------------------------- */
.cb-stat {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px 20px;
    background: #ffffff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    box-shadow: var(--shadow, 0 1px 3px rgba(0,0,0,0.08));
    transition: box-shadow 0.2s, transform 0.2s;
    position: relative;
    overflow: hidden;
}

.cb-stat:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    transform: translateY(-1px);
}

.cb-stat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.cb-stat-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary, #6c757d);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.cb-stat-icon {
    font-size: 20px;
    flex-shrink: 0;
    opacity: 0.7;
}

.cb-stat-value {
    font-size: 26px;
    font-weight: 700;
    color: var(--text-primary, #1a1a2e);
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
}

.cb-stat-footer {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
}

.cb-stat-trend {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-weight: 600;
    font-size: 12px;
}

.cb-stat-trend-up { color: var(--success, #28a745); }
.cb-stat-trend-down { color: var(--danger, #dc3545); }
.cb-stat-trend-flat { color: var(--text-secondary); }

/* Variants */
.cb-stat-primary .cb-stat-value { color: var(--accent, #4a90d9); }
.cb-stat-success .cb-stat-value { color: var(--success, #28a745); }
.cb-stat-danger .cb-stat-value { color: var(--danger, #dc3545); }
.cb-stat-warning .cb-stat-value { color: #e0a800; }

/* Accent bar on top */
.cb-stat-accent {
    border-top: 3px solid var(--accent, #4a90d9);
}
.cb-stat-accent-success {
    border-top: 3px solid var(--success, #28a745);
}
.cb-stat-accent-danger {
    border-top: 3px solid var(--danger, #dc3545);
}
.cb-stat-accent-warning {
    border-top: 3px solid var(--warning, #ffc107);
}
"##;
