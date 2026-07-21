## 1. Dashboard Weekly/Monthly Sales (F10)

- [x] 1.1 Replace `this_week: today_sales * 5.0` with actual SQL query for current week
- [x] 1.2 Replace `this_month: today_sales * 22.0` with actual SQL query for current month

## 2. Dashboard Outstanding AP (F13)

- [x] 2.1 Replace `"outstanding_ap": 0` with query: `SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM supplier_ledger`

## 3. AR Aging Fix (F11)

- [x] 3.1 Rewrite `report_ar_aging` query to use `SUM(CASE ...)` for each bucket
- [x] 3.2 Change `LEFT JOIN` to `JOIN` and add `HAVING SUM(i.balance_amount) > 0`

## 4. Supplier Balance Fix (F16)

- [x] 4.1 Rewrite `supplier_po_balance` to use ledger balance: `SUM(debit) - SUM(credit) FROM supplier_ledger`

## 5. Tax Summary Fix (F17)

- [x] 5.1 Rewrite tax calculation to use pre-tax base (back out from tax-inclusive total)

## 6. Verification

- [x] 6.1 Compilation passes with `cargo check` (0 errors)
- [ ] 6.2 Full integration test: create a week of invoices, verify dashboard values match
- [ ] 6.3 Verify AR aging total matches sum of all unpaid invoice balances
- [ ] 6.4 Verify supplier balance matches actual outstanding amount
- [ ] 6.5 Verify tax summary tax amounts are correct
