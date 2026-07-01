## Context

Four files, four independent fixes. None share logic or state. Each is described below.

## Decisions

**Dashboard: use the same pattern as InventoryDashboardPage**. `use_navigator()` at the top of the component, clone, push routes in onclick closures. Routes: `/sales/invoices/new`, `/inventory/items/new`, `/customers/new`, `/reports`.

**AR Aging Print: call trigger_print() directly**. No dedicated print-optimized page exists for the AR report. `trigger_print()` (`print_shared.rs`) invokes the browser print dialog for the current page — not pretty, but functional. A dedicated print layout can be added later if needed.

**Discount scope toggle: new signal, not a new component**. A single `use_signal(|| DiscountScope::BeforeTax)` toggled by the existing button. The `Discount` struct is rebuilt using this signal. The `scope_btn_class` CSS class should reflect the active scope (it currently derives from `discount_pct > 0` — wrong). Button text toggles between "Before Tax" / "After Tax".

**Supplier "New PO" buttons: navigate to create page**. `/purchases/orders/new` opens a fresh PO form. The supplier is not pre-filled (no URL param support in PO create page). This is still strictly better than a "Coming Soon" toast.

**Edit Supplier / Edit Employee: deferred**. Both need backend PUT routes and frontend edit pages — comparable to the invoice edit feature already built. Out of scope for this change.

## Risks / Trade-offs

- **[Risk]** AR Aging print prints the entire page with sidebar and filters → Accepted. A dedicated print template would be larger work. `ponytail: trigger_print() for now, print-optimized report page if print quality matters.`
- **[Risk]** Discount scope toggle only fixes the create page, not the edit page → Invoice edit page (`invoice_edit.rs`) manually calculates totals inline and doesn't use `DiscountScope` at all. Aligning it is a separate task.
- **[Risk]** Supplier PO navigation doesn't pre-fill the supplier → PO create page has no URL parameter support. Adding it would require refactoring the page to accept an optional `supplier_code` param.
