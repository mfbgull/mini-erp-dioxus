## ADDED Requirements

### Requirement: Multi-step operations are wrapped in transactions
Every route handler that executes 2 or more SQL statements as part of a single business operation SHALL wrap those statements in a SQLite transaction. The transaction SHALL use `BEGIN IMMEDIATE` to acquire a write lock. On successful completion of all statements, the transaction SHALL be committed with `COMMIT`. On any SQL error, the transaction SHALL be rolled back with `ROLLBACK` and the handler SHALL return an HTTP 500 error response.

#### Scenario: Invoice creation succeeds atomically
- **WHEN** a user creates an invoice with 3 line items, stock deductions, a journal entry, and a customer ledger entry
- **THEN** either ALL of these are committed to the database, or NONE are (on error)
- **AND** the client receives HTTP 201 on success or HTTP 500 on failure

#### Scenario: Invoice creation rolls back on stock deduction failure
- **WHEN** a user creates an invoice but the stock deduction for one item fails (e.g., the item was deleted between request start and stock update)
- **THEN** the invoice record, invoice items, journal entry, and customer ledger entry are all rolled back
- **AND** the client receives an HTTP 500 error response
- **AND** no partial data remains in the database

#### Scenario: Payment creation succeeds atomically
- **WHEN** a user records a payment that updates the invoice, customer balance, customer ledger, and creates a journal entry
- **THEN** either ALL updates are committed, or NONE are

#### Scenario: Production creation succeeds atomically
- **WHEN** a user creates a production that creates output stock movement, deducts input stock movements, updates stock balances for all affected items
- **THEN** either ALL stock changes are committed, or NONE are

### Requirement: Critical path operations propagate errors within transactions
Inside a transaction block, SQL operations that are essential to the business operation (invoice INSERT, stock UPDATE, journal INSERT, ledger INSERT) SHALL use error propagation (`?` operator or explicit error handling) instead of silent `.ok()` swallowing. Non-critical optional operations (e.g., creating a stock batch for tracking purposes) may retain `.ok()` behavior within the transaction scope.

#### Scenario: Journal entry failure rolls back invoice
- **WHEN** creating an invoice succeeds but the journal entry INSERT fails (e.g., account_id doesn't exist)
- **THEN** the entire invoice creation is rolled back
- **AND** no invoice, no stock deduction, no ledger entry remains

#### Scenario: Optional batch creation doesn't block transaction
- **WHEN** creating a goods receipt succeeds but the optional FIFO batch INSERT fails
- **THEN** the transaction continues (batch creation is non-critical)
- **AND** the goods receipt, stock update, and stock movement are still committed

### Requirement: All identified handlers are wrapped
The following handler functions SHALL be wrapped in transactions:

| File | Handler | Statement Count |
|------|---------|----------------|
| invoice_routes.rs | create_invoice | 12+ |
| invoice_routes.rs | update_invoice | 8+ |
| invoice_routes.rs | return_invoice | 6+ per item |
| payment_routes.rs | create_payment | 6+ |
| sales_routes.rs | create_sales_order | 3+ |
| sales_routes.rs | create_quotation | 3+ |
| purchase_routes.rs | create_purchase_order | 4+ |
| purchase_routes.rs | create_goods_receipt | 8+ |
| purchase_routes.rs | return_receipt | 3+ per item |
| purchase_routes.rs | create_direct_purchase | 6+ |
| manufacturing_routes.rs | create_production | 6+ |
| inventory_routes.rs | create_stock_movement | 5+ |
| accounting_routes.rs | create_expense | 3+ |
| accounting_routes.rs | pay_salary | 3+ |
| accounting_routes.rs | create_journal_entry | 2+ |

#### Scenario: Goods receipt transaction includes all sub-operations
- **WHEN** a user creates a goods receipt for a PO with 5 items
- **THEN** the transaction includes: GRN insert, 5× GRN item inserts, 5× PO item received_quantity updates, 5× stock balance updates, 5× stock batch inserts, 5× stock movement inserts, and 1× supplier ledger entry
- **AND** all 27+ operations commit or rollback atomically
