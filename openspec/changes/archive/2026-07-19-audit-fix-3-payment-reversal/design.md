## Context

The `delete_payment` handler at `payment_routes.rs:156-163` is 7 lines. It deletes the payment row and nothing else. The payment creation flow touches 6+ tables. The deletion must reverse all of them.

## Goals / Non-Goals

**Goals:**
- Full reversal of all payment effects on delete
- Atomic reversal (wrapped in transaction — depends on Change 1)
- Correct invoice status recalculation after reversal

**Non-Goals:**
- Soft-delete / audit trail for deleted payments
- Reversal of payment updates (update_payment also has issues but is lower priority)
- Batch payment deletion

## Decisions

### D1: Reverse before delete

**Decision:** Read all payment data first, perform all reversals, then delete the payment last. This ensures we have the data needed for reversals before the record is gone.

**Pattern:**
1. SELECT payment (amount, customer_id, invoice_id, payment_date)
2. DELETE payment_allocations WHERE payment_id = ?
3. UPDATE invoices: paid_amount -= amount, balance_amount += amount, recalculate status
4. INSERT customer_ledger: type='PAYMENT_REVERSAL', debit=amount, credit=0
5. UPDATE customers: current_balance += amount
6. INSERT journal_entry + journal_lines: debit AR (2), credit Cash (1)
7. DELETE payments WHERE id = ?

### D2: Invoice status recalculation

**Decision:** After reversing the payment, recalculate invoice status:
- If paid_amount <= 0: status = 'Unpaid'
- If paid_amount > 0 AND paid_amount < total_amount: status = 'Partially Paid'
- If paid_amount >= total_amount: status = 'Paid'

**Rationale:** The invoice status should reflect the actual payment state after reversal.

## Risks / Trade-offs

- **Payments without invoice_id:** Payments can exist without an invoice_id (advance payments). These need special handling — skip invoice update but still reverse customer ledger and GL.

- **Multiple payments on one invoice:** Deleting one payment should only reverse that payment's portion, not all payments. The current `paid_amount` in the invoice is the authoritative source.

- **Already deleted payments:** If a payment was previously deleted without reversal, the data is already corrupt. The `recalculate_balances` endpoint can fix customer balances but not invoice balances.
