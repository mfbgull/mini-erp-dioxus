## ADDED Requirements

### Requirement: Invoice cancellation reverses stock
When an invoice is cancelled via `PUT /api/invoices/{id}/cancel`, the system SHALL reverse stock effects for all items that haven't been returned. For each item, the unreturned quantity (`quantity - returned_qty`) is restored via a stock movement IN.

#### Scenario: Cancel unreturned invoice
- **WHEN** an invoice with 2 items (qty 10 each, 0 returned) is cancelled
- **THEN** stock movements IN of 10 units each are created for both items
- **AND** stock_balances and current_stock are increased by 10 each

#### Scenario: Cancel partially returned invoice
- **WHEN** an invoice with item qty=10, returned_qty=3 is cancelled
- **THEN** a stock movement IN of 7 units (10 - 3) is created
- **AND** stock_balances and current_stock are increased by 7

#### Scenario: Cancel fully returned invoice
- **WHEN** an invoice with item qty=10, returned_qty=10 is cancelled
- **THEN** no stock movement is created (nothing to restore)
- **AND** stock_balances and current_stock are unchanged

### Requirement: Invoice cancellation reverses customer ledger
When an invoice is cancelled, the system SHALL INSERT a customer ledger entry with type='CANCELLATION', debit=0, credit=(total_amount - returned_amount), reducing the customer's outstanding balance.

#### Scenario: Cancel unpaid invoice reverses ledger
- **WHEN** an unpaid invoice of 1000.00 is cancelled
- **THEN** a ledger entry is created with credit=1000.00
- **AND** customers.current_balance decreases by 1000.00

#### Scenario: Cancel partially paid invoice reverses remaining balance
- **WHEN** an invoice of 1000.00 with paid_amount=400.00 is cancelled
- **THEN** a ledger entry is created with credit=600.00 (remaining balance)
- **AND** customers.current_balance decreases by 600.00

### Requirement: Invoice cancellation reverses GL entries
When an invoice is cancelled, the system SHALL INSERT a reversing journal entry: debit Revenue (account_id=11) and credit Accounts Receivable (account_id=2) for the invoice's net amount (total_amount - returned_amount).

#### Scenario: Cancel invoice creates reversing journal
- **WHEN** an invoice of 1000.00 is cancelled
- **THEN** a journal entry is created with: debit account 11 (Revenue) for 1000.00, credit account 2 (AR) for 1000.00
- **AND** reference_type='invoice_cancellation'
