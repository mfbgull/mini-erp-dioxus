## 1. Invoice Total Calculation Fix (F2)

- [x] 1.1 Create helper function `calculate_line_item_amount(qty, unit_price, discount_type, discount_value, tax_rate) -> f64`
- [x] 1.2 Update `create_invoice` to compute each item's amount using the helper, then SUM for total_amount
- [x] 1.3 Update `update_invoice` to use same calculation
- [x] 1.4 Verify: `recalculate_invoice_totals` endpoint identifies and fixes mismatched totals

## 2. Line Item Discounts (F12)

- [x] 2.1 Factor discount_type/discount_value into the helper function from step 1.1
- [x] 2.2 Store discount in invoice_items (already in schema, just wire the calculation)
- [x] 2.3 `get_invoice` already returns discount_type/discount_value per item (verified in InvoiceItem model)

## 3. Invoice Edit Stock Reversal (F5)

- [x] 3.1 In `update_invoice`, SELECT old invoice_items before DELETE
- [x] 3.2 Compute delta (old_qty - new_qty) for each item
- [x] 3.3 Create stock movements for deltas (IN if qty decreased, OUT if increased)
- [x] 3.4 Update stock_balances and items.current_stock for each delta
- [x] 3.5 Handle item removals (full reverse) and additions (new OUT movement)
- [x] 3.6 Update customer ledger if total_amount changed (handles same-customer delta and customer-change scenarios)

## 4. Invoice Cancellation Reversal (F8)

- [x] 4.1 In `cancel_invoice`, load all invoice items with quantities and returned_qty
- [x] 4.2 For each item: create stock movement IN for (quantity - returned_qty)
- [x] 4.3 Update stock_balances and items.current_stock
- [x] 4.4 Calculate net amount = total_amount - returned_amount
- [x] 4.5 INSERT customer ledger entry: type='CANCELLATION', credit=net_amount
- [x] 4.6 UPDATE customers.current_balance: subtract net_amount
- [x] 4.7 INSERT journal entry: debit Revenue (11), credit AR (2) for net_amount
- [x] 4.8 Set invoice balance_amount = 0, paid_amount unchanged, status = 'Cancelled'

## 5. Data Consistency Check

- [x] 5.1 Verification query built into `recalculate_invoice_totals` endpoint
- [x] 5.2 Endpoint identifies all invoices with mismatched totals (ABS(total - line_sum) > 0.01)
- [x] 5.3 `POST /api/invoices/recalculate-totals` endpoint: fixes total_amount and balance_amount for all mismatched invoices

## 6. Verification

- [x] 6.1 Compilation passes with `cargo check` (0 errors)
- [ ] 6.2 Test: create invoice with 2 items including tax and discount — verify total matches sum of line items
- [ ] 6.3 Test: edit invoice to change quantity — verify stock movements created for delta
- [ ] 6.4 Test: cancel unpaid invoice — verify stock restored, ledger reversed, GL reversed
- [ ] 6.5 Test: cancel partially paid invoice — verify correct net amount reversed
- [ ] 6.6 Test: cancel fully returned invoice — verify no stock movement created
