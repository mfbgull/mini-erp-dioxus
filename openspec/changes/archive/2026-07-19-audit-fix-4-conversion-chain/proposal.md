## Why

The sales conversion chain (Quotation → Sales Order → Invoice) loses all line items at each step. `convert_quotation` creates a SO with the total amount but no `sales_order_items`. `convert_sales_order` creates an invoice with the total amount but no `invoice_items`. The resulting invoices have header totals but no detail rows — they're orphaned headers.

This is audit findings **F6** and **F7**.

## What Changes

- **Quotation → SO:** Copy all `quotation_items` to `sales_order_items` with correct quantities and prices.
- **SO → Invoice:** Copy all `sales_order_items` to `invoice_items` with correct quantities, prices, and tax. Also create stock movements (OUT) for the converted items, matching what `create_invoice` does.

## Capabilities

### New Capabilities

- `conversion-line-items`: Quotation→SO and SO→Invoice conversions preserve all line items, quantities, prices, and tax. Stock movements are created on SO→Invoice conversion.

## Impact

- **Server routes**: `src/server/sales_routes.rs` — rewrite `convert_quotation` and `convert_sales_order`
- **Models**: No changes
- **UI pages**: No changes
- **API client**: No changes
- **Database**: No schema changes
- **Breaking**: None — previously created empty invoices, now they'll have line items
