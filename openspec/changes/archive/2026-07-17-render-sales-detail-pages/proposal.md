## Why

The Quotation Detail and Sales Order Detail pages already fetch and parse their
records (header + line items) into fully-populated structs, but then render a
`"coming soon"` placeholder instead of the data. Users can open a quotation or
sales order from the list, watch it load, and see nothing — the last real gap in
the sales flow.

## What Changes

- Render the loaded `QuotationDetail` in `src/pages/quotation_detail.rs` (header,
  status badge, KPI cards, info grid, line-items table, action bar) instead of
  the `"coming soon"` empty-state.
- Render the loaded `SalesOrderDetail` in `src/pages/sales_order_detail.rs` the
  same way.
- Wire the action bar to **existing** API methods only:
  - Quotation: Convert to Invoice (`convert_quotation`), Back to list.
  - Sales Order: Convert to Invoice (`convert_sales_order`), Cancel
    (`cancel_sales_order`), Back to list.
- No new backend endpoints, no new dependencies. The CSS, data structs, and
  status-class helpers already exist in both files.

## Capabilities

### New Capabilities
- `sales-detail-views`: Read-only detail views for quotations and sales orders —
  displaying header fields, computed KPIs, line items, and status-appropriate
  actions bound to existing sales APIs.

### Modified Capabilities
<!-- none: no existing spec's requirements change -->

## Impact

- Code: `src/pages/quotation_detail.rs`, `src/pages/sales_order_detail.rs`
  (render body only; fetch logic unchanged).
- APIs: consumes existing `convert_quotation`, `convert_sales_order`,
  `cancel_sales_order`. None added or modified.
- Data limitation: the server `SalesOrder`/`Quotation` responses expose only
  `total_amount` (no subtotal/discount/tax breakdown), so those KPI fields
  render from `total` and are marked with a `ponytail:` note rather than
  fabricated. Out of scope: extending the server response.
