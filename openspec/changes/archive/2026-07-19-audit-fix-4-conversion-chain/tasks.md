## 1. Quotation → SO Line Item Copy

- [x] 1.1 In `convert_quotation`, SELECT all quotation_items for the source quotation
- [x] 1.2 For each quotation_item, INSERT into sales_order_items with item_id, quantity, unit_price, amount
- [x] 1.3 Compute SO total_amount from SUM of copied line item amounts
- [x] 1.4 Wrap in transaction (BEGIN IMMEDIATE / COMMIT / ROLLBACK)

## 2. SO → Invoice Line Item Copy + Stock

- [x] 2.1 In `convert_sales_order`, SELECT all sales_order_items for the source SO
- [x] 2.2 For each sales_order_item, INSERT into invoice_items with item_id, quantity, unit_price, amount
- [x] 2.3 Compute invoice total_amount from SUM of copied line item amounts
- [x] 2.4 For each item, create stock movement OUT (unit_cost from items.standard_cost)
- [x] 2.5 For each item, update stock_balances: quantity -= item_quantity
- [x] 2.6 For each item, update items.current_stock -= item_quantity
- [x] 2.7 Create customer ledger entry for the invoice
- [x] 2.8 Create journal entry (debit AR, credit Revenue)
- [x] 2.9 Wrap in transaction (BEGIN IMMEDIATE / COMMIT / ROLLBACK)

## 3. Verification

- [x] 3.1 Compilation passes with `cargo check` (0 errors)
- [ ] 3.2 Test: create quotation with 3 items, convert to SO — verify SO has 3 line items with correct amounts
- [ ] 3.3 Test: convert SO to invoice — verify invoice has 3 line items, stock movements created, stock balances updated
- [ ] 3.4 Test: verify invoice total matches sum of line items
- [ ] 3.5 Test: verify customer ledger entry created for converted invoice
