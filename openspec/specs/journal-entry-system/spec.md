## ADDED Requirements

### Requirement: Journal entry creation API
The system SHALL expose `POST /api/accounting/journal-entries` that accepts a JSON body with `entry_date`, `reference_type`, `reference_id`, and `lines` (array of `{account_id, debit, credit, description}`). The system SHALL validate that `SUM(debit) = SUM(credit)` across all lines. The entry and all lines SHALL be inserted atomically within a single transaction.

#### Scenario: Create balanced journal entry
- **WHEN** a user POSTs a journal entry with two lines: account 1001 debit 500.00, account 4001 credit 500.00
- **THEN** a row is inserted into `journal_entries` and two rows into `journal_lines`, all within one transaction
- **AND** the response returns the created entry with its lines

#### Scenario: Reject unbalanced journal entry
- **WHEN** a user POSTs a journal entry with lines: debit 500.00, credit 300.00
- **THEN** the system returns HTTP 400 with error message "Debits must equal credits"

### Requirement: Journal entry list API
The system SHALL expose `GET /api/accounting/journal-entries` that returns a paginated list of journal entries with optional filters: `from_date`, `to_date`, `account_id`, `reference_type`. Each entry SHALL include its lines.

#### Scenario: List journal entries with date filter
- **WHEN** a user requests journal entries from 2026-01-01 to 2026-06-30
- **THEN** the system returns only entries within that date range

### Requirement: Journal entry detail API
The system SHALL expose `GET /api/accounting/journal-entries/{id}` that returns a single journal entry with all its lines.

#### Scenario: Get journal entry detail
- **WHEN** a user requests journal entry ID 42
- **THEN** the system returns the entry with `id=42` and all associated `journal_lines`

### Requirement: Journal entry seed data
The `seed_data()` function in `db.rs` SHALL create journal entries for all seeded transactions (invoices, payments, purchases, expenses) so that financial reports are not blank on fresh installs. Each seeded invoice SHALL produce a journal entry with a debit to Accounts Receivable and a credit to Revenue. Each seeded payment SHALL produce a debit to Cash and a credit to Accounts Receivable.

#### Scenario: Fresh install has non-zero account balances
- **WHEN** a fresh database is seeded
- **THEN** `chart_of_accounts` balance queries return non-zero values for Cash, AR, Revenue, and Expense accounts
- **AND** trial balance report shows `SUM(debit) = SUM(credit)`

### Requirement: Auto-journal on invoice creation
When an invoice is created, the system SHALL automatically create a journal entry with `reference_type='invoice'` and the invoice ID as `reference_id`. The lines SHALL debit Accounts Receivable and credit Revenue for the invoice total.

#### Scenario: Invoice creation triggers journal entry
- **WHEN** an invoice is created via `POST /api/invoices`
- **THEN** a journal entry is created with `reference_type='invoice'` and two lines: debit AR account, credit Revenue account

### Requirement: Auto-journal on payment received
When a customer payment is created, the system SHALL automatically create a journal entry with `reference_type='payment'` and lines debiting Cash and crediting Accounts Receivable.

#### Scenario: Payment triggers journal entry
- **WHEN** a payment is created via `POST /api/payments`
- **THEN** a journal entry is created with `reference_type='payment'` and lines: debit Cash, credit AR

### Requirement: Auto-journal on purchase order
When a purchase order is created, the system SHALL create a journal entry with `reference_type='purchase_order'` and lines debiting Inventory (or COGS) and crediting Accounts Payable.

#### Scenario: Purchase order triggers journal entry
- **WHEN** a purchase order is created via `POST /api/purchase-orders`
- **THEN** a journal entry is created with `reference_type='purchase_order'`

### Requirement: Auto-journal on expense
When an expense is created, the system SHALL create a journal entry with `reference_type='expense'` and lines debiting the Expense account and crediting Cash.

#### Scenario: Expense triggers journal entry
- **WHEN** an expense is created via `POST /api/expenses`
- **THEN** a journal entry is created with `reference_type='expense'`

### Requirement: Auto-journal on salary payment
When a salary payment is recorded, the system SHALL create a journal entry with `reference_type='salary'` and lines debiting Salary Expense and crediting Cash.

#### Scenario: Salary payment triggers journal entry
- **WHEN** a salary payment is recorded via `POST /api/employees/{id}/salary`
- **THEN** a journal entry is created with `reference_type='salary'`

### Requirement: Journal entry list UI page
The system SHALL provide a UI page at `src/pages/journal_entry_list.rs` that displays journal entries in a table with columns: Date, Reference Type, Reference ID, Description, Debit Total, Credit Total. The page SHALL include date range filters.

#### Scenario: View journal entries page
- **WHEN** a user navigates to the journal entries page
- **THEN** the page displays a list of journal entries with date filters

### Requirement: Journal entry create UI page
The system SHALL provide a UI page at `src/pages/journal_entry_create.rs` with a form for creating journal entries. The form SHALL include: entry date, reference type dropdown, and a dynamic lines table where users can add/remove lines with account, debit, credit, and description fields. The form SHALL display a running total of debits and credits and show an error if they don't balance before submission.

#### Scenario: Create journal entry via UI
- **WHEN** a user fills in the journal entry form with balanced debits and credits and submits
- **THEN** a journal entry is created and the user is redirected to the journal entry list
