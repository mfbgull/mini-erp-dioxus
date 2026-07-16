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
.cb-btn-sm { padding: var(--space-1) var(--space-2); font-size: var(--text-xs); border-radius: var(--radius-sm); }
.cb-btn-md { padding: var(--space-2) var(--space-5); font-size: var(--text-base); }
.cb-btn-lg { padding: var(--space-3) var(--space-6); font-size: var(--text-lg); border-radius: var(--radius-md); }

/* Variants */
.cb-btn-primary {
    background: var(--accent);
    color: #ffffff;
    border-color: var(--accent);
}
.cb-btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
}
.cb-btn-primary:active:not(:disabled) {
    background: #047857;
}

.cb-btn-secondary {
    background: #ffffff;
    color: var(--text-primary);
    border-color: var(--border-color);
}
.cb-btn-secondary:hover:not(:disabled) {
    background: var(--surface-tertiary);
    border-color: var(--border-color);
}

.cb-btn-danger {
    background: var(--danger);
    color: #ffffff;
    border-color: var(--danger);
}
.cb-btn-danger:hover:not(:disabled) {
    background: #DC2626;
    border-color: #DC2626;
}

.cb-btn-ghost {
    background: transparent;
    color: var(--text-primary);
    border-color: transparent;
}
.cb-btn-ghost:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.05);
}

.cb-btn-success {
    background: var(--success);
    color: #ffffff;
    border-color: var(--success);
}
.cb-btn-success:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
}

.cb-btn-warning {
    background: var(--warning);
    color: var(--text-primary);
    border-color: var(--warning);
}
.cb-btn-warning:hover:not(:disabled) {
    background: #D97706;
    border-color: #D97706;
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
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 4px;
}

.cb-input-required {
    color: var(--danger);
    font-size: 14px;
}

.cb-input-hint {
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 400;
}

.cb-input {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    background: #ffffff;
    font-family: var(--font-family);
    font-size: 14px;
    color: var(--text-primary);
    transition: all 0.15s ease;
}

.cb-input:hover {
    border-color: var(--text-muted);
}

.cb-input:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-ring);
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
    color: var(--text-muted);
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
    color: var(--text-muted);
    font-size: 16px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
}

.cb-input-error {
    border-color: var(--danger);
}
.cb-input-error:focus-within {
    box-shadow: 0 0 0 3px var(--danger-light);
}

.cb-input-error-text {
    font-size: var(--text-xs);
    color: var(--danger);
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: var(--space-1);
}

/* Textarea variant */
.cb-input-textarea {
    padding: var(--space-2) var(--space-3);
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
    padding: var(--space-6);
    animation: cb-fade-in var(--ease-fast);
}

.cb-modal {
    background: var(--surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    max-height: 85vh;
    width: 100%;
    animation: cb-slide-up var(--ease-normal);
}

.cb-modal-sm { max-width: 400px; }
.cb-modal-md { max-width: 540px; }
.cb-modal-lg { max-width: 720px; }
.cb-modal-xl { max-width: 960px; }
.cb-modal-full { max-width: calc(100vw - var(--space-12)); }

.cb-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
}

.cb-modal-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--text-primary);
}

.cb-modal-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 18px;
    transition: all var(--ease-fast);
}

.cb-modal-close:hover {
    background: var(--surface-tertiary);
    color: var(--text-primary);
}

.cb-modal-body {
    padding: var(--space-5);
    overflow-y: auto;
    flex: 1;
}

.cb-modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    border-top: 1px solid var(--border-color);
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
    padding: var(--space-3) var(--space-3);
    background: var(--surface);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    border-left: 4px solid var(--accent);
    pointer-events: auto;
    animation: cb-toast-in var(--ease-slow);
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
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 2px;
}

.cb-toast-message {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.4;
}

.cb-toast-close {
    flex-shrink: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 16px;
    padding: 2px;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    transition: background var(--ease-fast);
}

.cb-toast-close:hover {
    background: var(--surface-tertiary);
}

/* Toast type colors */
.cb-toast-success { border-left-color: var(--success); }
.cb-toast-error { border-left-color: var(--danger); }
.cb-toast-warning { border-left-color: var(--warning); }
.cb-toast-info { border-left-color: var(--info); }

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
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    background: #ffffff;
    cursor: pointer;
    min-height: 38px;
    transition: border-color 0.15s;
}

.cb-select-input:hover { border-color: var(--text-muted); }
.cb-select-input:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-ring); }

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
    color: var(--text-muted);
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
    background: var(--surface-tertiary);
}

.cb-select-option-selected {
    background: var(--accent-light);
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
    background: var(--accent-light);
    border-radius: 100px;
    font-size: 12px;
    color: var(--info);
}

.cb-select-tag-remove {
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--info);
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
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 500;
    min-width: 160px;
    padding: 4px 0;
    animation: cb-fade-in var(--ease-fast);
}

.cb-dropdown-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-primary);
    cursor: pointer;
    transition: background var(--ease-fast);
    border: none;
    background: transparent;
    width: 100%;
    text-align: left;
    font-family: inherit;
    white-space: nowrap;
}

.cb-dropdown-item:hover {
    background: var(--surface-tertiary);
}

.cb-dropdown-item-danger {
    color: var(--danger);
}
.cb-dropdown-item-danger:hover {
    background: var(--danger-light);
}

.cb-dropdown-divider {
    height: 1px;
    background: var(--border-light);
    margin: var(--space-1) 0;
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
    font-size: var(--text-xs);
    color: var(--text-muted);
    font-family: var(--font-mono);
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
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--surface);
    font-family: var(--font-family);
    font-size: var(--text-sm);
    color: var(--text-primary);
    transition: border-color var(--ease-fast);
    min-width: 0;
}

.cb-daterange-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-ring);
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
    padding: var(--space-4) var(--space-5);
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius);
    box-shadow: var(--shadow-sm);
    transition: box-shadow var(--ease-normal), transform var(--ease-normal);
    position: relative;
    overflow: hidden;
}

.cb-stat:hover {
    box-shadow: var(--shadow-md);
    transform: translateY(-2px);
}

.cb-stat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.cb-stat-title {
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
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
    font-size: var(--text-3xl);
    font-weight: 700;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
}

.cb-stat-footer {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-secondary);
}

/* Skeleton shimmer for stat cards and other components */
.cb-stat-loading .skeleton {
    background: linear-gradient(90deg, var(--border-color) 25%, var(--surface-secondary) 50%, var(--border-color) 75%);
    background-size: 200% 100%;
    animation: dg-shimmer 1.5s infinite;
    border-radius: var(--radius-sm);
    display: inline-block;
}

.cb-stat-trend {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-weight: 600;
    font-size: var(--text-xs);
}

.cb-stat-trend-up { color: var(--success); }
.cb-stat-trend-down { color: var(--danger); }
.cb-stat-trend-flat { color: var(--text-secondary); }

/* Variants */
.cb-stat-primary .cb-stat-value { color: var(--accent); }
.cb-stat-success .cb-stat-value { color: var(--success); }
.cb-stat-danger .cb-stat-value { color: var(--danger); }
.cb-stat-warning .cb-stat-value { color: var(--warning); }

/* Accent bar on top */
.cb-stat-accent {
    border-top: 3px solid var(--accent);
}
.cb-stat-accent-success {
    border-top: 3px solid var(--success);
}
.cb-stat-accent-danger {
    border-top: 3px solid var(--danger);
}
.cb-stat-accent-warning {
    border-top: 3px solid var(--warning);
}
"##;
