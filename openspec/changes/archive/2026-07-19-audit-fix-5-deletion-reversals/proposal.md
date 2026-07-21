## Why

Three deletion handlers perform no reversals:

1. **F9 — Production deletion:** `delete_production` removes the production record and inputs but doesn't reverse stock movements, output stock, or input stock. Items produced stay in inventory; consumed materials are not restored.

2. **F19 — Expense deletion:** `delete_expense` removes the expense but doesn't reverse the journal entry (debit Expense, credit Cash), leaving a permanent GL imbalance.

3. **F20 — PO deletion:** `delete_purchase_order` removes the PO and items but doesn't reverse the supplier ledger entry created at PO creation.

## What Changes

- **Production delete:** Reverse output stock (subtract produced quantity), restore input stocks (add back consumed quantities), delete stock movements, update stock_balances and current_stock.
- **Expense delete:** Reverse journal entry (debit Cash, credit Expense), delete the journal entry and lines.
- **PO delete:** Reverse supplier ledger entry, reverse journal entry.

## Capabilities

### New Capabilities

- `production-delete-reversal`: Deleting a production reverses all stock effects (output and inputs).
- `expense-delete-reversal`: Deleting an expense reverses the associated journal entry.
- `po-delete-reversal`: Deleting a PO reverses the supplier ledger and journal entry.

## Impact

- **Server routes**: `manufacturing_routes.rs`, `accounting_routes.rs`, `purchase_routes.rs`
- **Models**: No changes
- **UI pages**: No changes
- **Database**: No schema changes
- **Breaking**: None — existing deletions leave corrupt data; new behavior is correct
