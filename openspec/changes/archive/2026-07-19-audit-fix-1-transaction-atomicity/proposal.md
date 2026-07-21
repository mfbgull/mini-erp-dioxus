## Why

Every multi-step database operation in the application performs 5-15+ individual SQL statements without transaction wrapping. Each statement commits independently. Errors are silently swallowed with `.ok()`. If any step fails midway, the database is left in an inconsistent state: invoices exist without stock deductions, payments exist without balance updates, journal entries are orphaned. This is the foundational flaw that makes all other financial data unreliable.

This is audit finding **F3** from the comprehensive business logic audit.

## What Changes

Wrap every multi-step route handler in a SQLite transaction (`BEGIN`/`COMMIT`/`ROLLBACK`). On any SQL error inside the transaction, roll back all changes and return an error response instead of silently continuing with partial data.

Affected handlers (grouped by file):

**invoice_routes.rs:**
- `create_invoice` — 12+ statements (invoice, items, stock movements, stock balances, customer ledger, journal entries, optional payment)
- `update_invoice` — 8+ statements (delete items, insert new items, recalculate paid/balance)
- `return_invoice` — 6+ statements per returned item (stock movement, stock balance, items update, ledger entry)

**payment_routes.rs:**
- `create_payment` — 6+ statements (payment, allocations, invoice updates, customer balance, ledger, journal)

**sales_routes.rs:**
- `create_sales_order` — 3+ statements
- `create_quotation` — 3+ statements

**purchase_routes.rs:**
- `create_purchase_order` — 4+ statements
- `create_goods_receipt` — 8+ statements (receipt, items, PO item updates, stock, batches, movements, ledger)
- `return_receipt` — 3+ statements per item
- `create_direct_purchase` — 6+ statements

**manufacturing_routes.rs:**
- `create_production` — 6+ statements (production, output stock, inputs, input stock movements)

**inventory_routes.rs:**
- `create_stock_movement` — 5+ statements

**accounting_routes.rs:**
- `create_expense` — 3+ statements
- `pay_salary` — 3+ statements
- `create_journal_entry` — 2+ statements

**Strategy:** Use `db.execute_batch("BEGIN IMMEDIATE")` at the start and `db.execute_batch("COMMIT")` on success. On any error, `db.execute_batch("ROLLBACK")`. Convert silent `.ok()` calls to `?` or explicit error propagation within the transaction scope.

## Capabilities

### New Capabilities

- `transaction-wrapping`: A consistent pattern for wrapping multi-statement operations in SQLite transactions with rollback-on-error. Applied to all 13+ route handlers identified above.

### Modified Capabilities

(none)

## Impact

- **Server routes**: All 8 `*_routes.rs` files — wrap existing logic in transaction blocks
- **Models**: No changes
- **UI pages**: No changes
- **API client**: No changes
- **Database**: No schema changes — uses SQLite transaction primitives
- **Breaking**: Error responses change from silent partial success to explicit failure with rollback. Clients that previously received "success" on partial failures will now receive errors. This is correct behavior.
