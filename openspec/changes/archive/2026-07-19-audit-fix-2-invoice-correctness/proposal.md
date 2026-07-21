## Why

The invoice system has four distinct correctness bugs that produce wrong financial data:

1. **F2 — Tax mismatch:** Invoice total includes per-item tax, but line item `amount` excludes it. Viewing an invoice shows a total that doesn't match the sum of line items.
2. **F5 — Edit stock drift:** Editing an invoice creates new line items but never adjusts stock movements, balances, or `current_stock`. Quantities silently diverge.
3. **F8 — Cancel is status-only:** Cancelling an invoice changes the status field but doesn't reverse stock movements, customer ledger entries, customer balance, or GL journal entries.
4. **F12 — Dead discount fields:** `invoice_items` has `discount_type`/`discount_value` columns that are stored but never used in calculations.

## What Changes

- **Fix invoice total calculation:** Compute `total_amount` FROM the sum of line item amounts (which should include tax and discount), not independently.
- **Fix line item amount:** Include per-item tax and per-item discount in the stored `amount` for each invoice item.
- **Fix invoice edit:** On update, reverse old stock movements and create new ones for the updated quantities.
- **Fix invoice cancellation:** On cancel, reverse stock movements, customer ledger, customer balance, and GL entries.
- **Implement line-item discounts:** Factor `discount_type`/`discount_value` into line item `amount` calculation.

## Capabilities

### New Capabilities

- `invoice-tax-consistency`: Invoice total is computed from line items (quantity × unit_price ± discount + tax), ensuring line item sum always equals invoice total.
- `invoice-edit-stock`: Editing an invoice reverses old stock movements and creates new ones, keeping inventory consistent.
- `cancellation-reversal`: Cancelling an invoice reverses all effects: stock, ledger, balance, GL.
- `line-item-discounts`: Per-item discount_type/discount_value are factored into the line item amount.

### Modified Capabilities

(none)

## Impact

- **Server routes**: `src/server/invoice_routes.rs` — rewrite create, update, cancel, return handlers
- **Models**: `src/models.rs` — possibly add `tax_amount` and `discount_amount` fields to `InvoiceItem`
- **UI pages**: `src/pages/invoice_detail.rs`, `invoice_edit.rs`, `invoice_create.rs` — display tax/discount breakdown
- **API client**: `src/api.rs` — update if model fields change
- **Database**: No schema changes — existing columns already support the needed data
- **Breaking**: Invoice `total_amount` values will change (now tax-inclusive per item). Existing invoices in the database will have inconsistent totals — a one-time data fix migration may be needed.
