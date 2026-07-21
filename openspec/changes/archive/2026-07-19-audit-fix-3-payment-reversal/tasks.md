## 1. Payment Data Retrieval

- [x] 1.1 In `delete_payment`, SELECT payment data before deletion (amount, customer_id, payment_no, payment_date)
- [x] 1.2 SELECT all payment_allocations for this payment

## 2. Invoice Balance Reversal

- [x] 2.1 For each allocation: UPDATE invoices SET paid_amount -= alloc.amount, balance_amount += alloc.amount
- [x] 2.2 Recalculate invoice status: unpaid / partially paid / paid
- [x] 2.3 Handle payments without invoice_id (advance payments) — skip invoice update (allocations vec will be empty)

## 3. Customer Ledger Reversal

- [x] 3.1 Get last customer ledger balance
- [x] 3.2 INSERT customer_ledger entry: type='PAYMENT_REVERSAL', debit=payment_amount, credit=0, balance=last_balance + amount
- [x] 3.3 UPDATE customers SET current_balance += amount, credit_balance -= amount

## 4. GL Reversal

- [x] 4.1 INSERT journal_entries with reference_type='payment_deletion'
- [x] 4.2 INSERT journal_lines: debit AR (account_id=2), credit Cash (account_id=1) for payment amount

## 5. Cleanup

- [x] 5.1 DELETE payment_allocations WHERE payment_id = ?
- [x] 5.2 DELETE payments WHERE id = ?

## 6. Verification

- [x] 6.1 Compilation passes with `cargo check` (0 errors)
- [ ] 6.2 Test: create invoice, make payment, delete payment — verify invoice returns to 'Unpaid' with full balance
- [ ] 6.3 Test: create invoice, make partial payment, delete payment — verify correct status
- [ ] 6.4 Test: verify customer ledger shows PAYMENT_REVERSAL entry
- [ ] 6.5 Test: verify GL trial balance still balances after deletion
