## Why

Six buttons across the app are non-functional stubs or no-ops. They don't do anything when clicked, or show a "coming soon" toast for actions that already exist. Fixing them improves first-run impression and removes dead ends from common workflows.

## What Changes

### Wire up 4 dashboard quick-action buttons
`DashboardPage` has New Invoice, New Item, New Customer, and View Reports buttons with `onclick: move |_| {}`. Add `use_navigator()` and navigate to existing routes.

### Wire up AR Aging Print button
The Print button on the AR Aging report page has `onclick: move |_| {}`. Import and call `trigger_print()` from `print_shared`.

### Implement discount scope toggle on Invoice Create page
The "Before Tax" button is supposed to toggle discount calculation scope. The calculation layer (`DiscountScope`, `Discount`) already supports both BeforeTax / AfterTax, but the create page hardcodes `DiscountScope::BeforeTax` and the button does nothing. Add a state signal, wire toggle, update button text and Discount struct accordingly.

### Replace "Coming Soon" stubs with real navigation
- Supplier detail "New PO" and "New Purchase Order" → navigate to `/purchases/orders/new`
- **Deferred**: "Edit Supplier" and "Edit Employee" stay as "Coming Soon" toasts — they require backend PUT routes + frontend edit pages, which is a separate feature.

## Capabilities

### New
- `dashboard-quick-actions`: 4 quick-action buttons navigate to their respective pages
- `ar-aging-print`: Print button invokes browser print dialog
- `invoice-discount-scope`: Discount scope toggle (Before Tax / After Tax) on invoice create form
- `supplier-new-po-nav`: New PO buttons on supplier detail navigate to purchase order creation

### Modified

## Impact

- `src/pages/dashboard.rs` — Add `use_navigator()`, wire 4 onclick handlers
- `src/pages/ar_aging.rs` — Import `trigger_print()`, wire Print onclick
- `src/pages/invoice_create.rs` — Add discount scope signal, wire toggle button, update Discount struct
- `src/pages/supplier_detail.rs` — Replace "Coming Soon" handlers with `navigator.push`
