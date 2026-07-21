## Why

Deleting a payment via `DELETE /api/payments/{id}` only deletes the `payments` row. It does not reverse: invoice `paid_amount`/`balance_amount`/status, customer ledger entries, customer `current_balance`, GL journal entries, or `payment_allocations`. This causes permanent data corruption — the customer still appears to owe money, the invoice shows incorrect payment status, and the GL is imbalanced.

This is audit finding **F4**.

## What Changes

Implement full reversal logic in `delete_payment`:
1. Look up the payment being deleted (amount, customer_id, invoice_id)
2. Delete associated `payment_allocations`
3. Reverse invoice `paid_amount` and `balance_amount`, recalculate status
4. Reverse customer ledger entry
5. Restore `customers.current_balance`
6. Reverse GL journal entry (debit AR, credit Cash)
7. Delete the payment record

## Capabilities

### New Capabilities

- `payment-deletion-reversal`: Deleting a payment reverses all effects on invoice balances, customer ledger, customer balance, and GL entries.

## Impact

- **Server routes**: `src/server/payment_routes.rs` — rewrite `delete_payment` handler
- **Models**: No changes
- **UI pages**: No changes (delete button already exists)
- **API client**: No changes
- **Database**: No schema changes
- **Breaking**: Payments that were previously deleted without reversal will have left orphaned data. A recalculation endpoint exists at `POST /api/customers/recalculate-balances` for customer balances.
