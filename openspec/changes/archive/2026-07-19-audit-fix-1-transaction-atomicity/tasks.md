## 1. Transaction Helper Pattern

- [x] 1.1 Define a helper function or macro for transaction begin/commit/rollback pattern
- [x] 1.2 Decide: inline `execute_batch` calls vs. a reusable wrapper function — chose inline for simplicity

## 2. Invoice Routes

- [x] 2.1 Wrap `create_invoice` in transaction — convert critical `.ok()` to explicit error checks
- [x] 2.2 Wrap `update_invoice` in transaction — convert critical `.ok()` to explicit error checks
- [x] 2.3 Wrap `return_invoice` in transaction — convert critical `.ok()` to explicit error checks

## 3. Payment Routes

- [x] 3.1 Wrap `create_payment` in transaction — convert critical `.ok()` to explicit error checks

## 4. Sales Routes

- [x] 4.1 Wrap `create_sales_order` in transaction
- [x] 4.2 Wrap `create_quotation` in transaction

## 5. Purchase Routes

- [x] 5.1 Wrap `create_purchase_order` in transaction
- [x] 5.2 Wrap `create_goods_receipt` in transaction
- [x] 5.3 Wrap `return_receipt` in transaction
- [x] 5.4 Wrap `create_direct_purchase` in transaction

## 6. Manufacturing Routes

- [x] 6.1 Wrap `create_production` in transaction

## 7. Inventory Routes

- [x] 7.1 Wrap `create_stock_movement` in transaction

## 8. Accounting Routes

- [x] 8.1 Wrap `create_expense` in transaction
- [x] 8.2 Wrap `pay_salary` in transaction
- [x] 8.3 Wrap `create_journal_entry` in transaction

## 9. Verification

- [x] 9.1 Compilation passes with `cargo check` (0 errors, only pre-existing warnings)
- [ ] 9.2 Test: create invoice with valid data — all records committed
- [ ] 9.3 Test: create invoice where stock update fails — all records rolled back
- [ ] 9.4 Test: create payment — invoice balance, customer balance, ledger, journal all update atomically
- [ ] 9.5 Test: create goods receipt — stock, batches, movements, PO items, ledger all update atomically
- [ ] 9.6 Verify no `.ok()` on critical-path operations inside transactions
