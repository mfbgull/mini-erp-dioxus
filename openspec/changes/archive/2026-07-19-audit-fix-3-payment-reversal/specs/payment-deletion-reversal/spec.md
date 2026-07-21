## ADDED Requirements

### Requirement: Deleting a payment reverses invoice balance
When a payment is deleted via `DELETE /api/payments/{id}`, the system SHALL decrease the linked invoice's `paid_amount` by the payment amount and increase `balance_amount` by the payment amount. The invoice status SHALL be recalculated.

#### Scenario: Delete payment on partially paid invoice
- **WHEN** an invoice of 1000.00 has a payment of 300.00 deleted
- **THEN** the invoice's paid_amount decreases by 300.00
- **AND** balance_amount increases by 300.00
- **AND** status changes from 'Partially Paid' to the correct status based on remaining payments

#### Scenario: Delete payment that fully paid an invoice
- **WHEN** an invoice of 500.00 has its only payment of 500.00 deleted
- **THEN** paid_amount becomes 0, balance_amount becomes 500.00
- **AND** status changes to 'Unpaid'

### Requirement: Deleting a payment reverses customer ledger
When a payment is deleted, the system SHALL INSERT a customer ledger entry with `type='PAYMENT_REVERSAL'`, `debit=payment_amount`, `credit=0`, and a running balance that increases by the payment amount.

#### Scenario: Delete payment creates reversal ledger entry
- **WHEN** a payment of 500.00 from customer ID 5 is deleted
- **THEN** a ledger entry is created: customer_id=5, type='PAYMENT_REVERSAL', debit=500.00, credit=0
- **AND** the running balance increases by 500.00

### Requirement: Deleting a payment restores customer balance
When a payment is deleted, `customers.current_balance` SHALL be increased by the payment amount and `customers.credit_balance` SHALL be decreased by the payment amount.

#### Scenario: Delete payment restores customer balance
- **WHEN** a payment of 500.00 from customer ID 5 is deleted
- **THEN** customers.current_balance += 500.00
- **AND** customers.credit_balance -= 500.00

### Requirement: Deleting a payment reverses GL entry
When a payment is deleted, the system SHALL INSERT a reversing journal entry: debit Accounts Receivable (account_id=2) and credit Cash (account_id=1) for the payment amount.

#### Scenario: Delete payment creates reversing journal
- **WHEN** a payment of 500.00 is deleted
- **THEN** a journal entry is created: debit account 2 (AR) for 500.00, credit account 1 (Cash) for 500.00
- **AND** reference_type='payment_deletion'

### Requirement: Deleting a payment deletes payment allocations
When a payment is deleted, all associated `payment_allocations` rows SHALL be deleted first (before the payment record).

#### Scenario: Payment with allocations is fully cleaned
- **WHEN** a payment allocated to 2 invoices is deleted
- **THEN** both payment_allocations rows are deleted
- **AND** both invoices' paid_amount and balance_amount are updated
