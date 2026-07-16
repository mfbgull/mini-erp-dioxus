## Why

The accounting subsystem has structural gaps that render financial reporting non-functional. Both customer and supplier ledger tables exist but are never written to — all balance queries sum over empty tables. The journal entry system (double-entry bookkeeping) has no write path and no seed data, so trial balance, balance sheet, and P&L reports always return zeros. Additionally, salary payments are write-only (no way to view history), the dashboard layout API exists with no UI, and there is a broken SQL query that silently returns incorrect supplier balances.

## What Changes

- **Fix customer ledger write path**: INSERT ledger entries on invoice creation, payment received, invoice return/cancellation. Calculate running balance on each entry.
- **Fix supplier ledger write path + broken query**: INSERT ledger entries on PO creation, goods receipt, direct purchase. Fix `purchase_routes.rs:716` where `SUM(amount)` references a non-existent column. Add SELECT endpoints and UI.
- **Implement journal entry write path**: CRUD API for journal entries with atomic entry+lines creation. Seed initial entries from existing transaction data. Wire auto-journaling for invoices, payments, purchases, expenses, and salary payments. Add validation (debits = credits).
- **Add salary payment history**: List endpoint + UI tab in employee detail page.
- **Add dashboard layout management UI**: Page to create, edit, delete saved layouts.
- **Clean up unused tables**: Remove `item_locations`, `work_orders`, `material_consumption`, `employee_documents` schema definitions (or decide to implement).

## Capabilities

### New Capabilities

- `ledger-write-path`: INSERT triggers for customer_ledger and supplier_ledger tables, triggered by invoices, payments, and purchases. Includes running balance calculation and balance recalculation endpoints.
- `journal-entry-system`: CRUD API for journal_entries and journal_lines tables. Atomic creation with debit=credit validation. Seed data backfill from existing transactions. Auto-journaling wiring for all transaction types.
- `salary-payment-history`: List and detail endpoints for salary_payments table. UI tab in employee detail page.
- `dashboard-layout-ui`: Management page for dashboard_layouts table. List, create, edit, delete layouts. Wire dashboard to load saved layouts.

### Modified Capabilities

(none — these are all new capabilities)

## Impact

- **Server routes**: `src/server/invoice_routes.rs`, `payment_routes.rs`, `purchase_routes.rs`, `accounting_routes.rs`, `customer_routes.rs`, `dashboard_routes.rs` — new INSERT/SELECT logic
- **Models**: `src/models.rs` — possibly new form structs for journal entry creation
- **UI pages**: `src/pages/supplier_detail.rs` (ledger tab), `src/pages/journal_entry_list.rs`, `journal_entry_create.rs`, `src/pages/employee_detail.rs` (salary tab), `src/pages/dashboard_layouts.rs`
- **API client**: `src/api.rs` — new client functions for ledger, journal, salary, layout endpoints
- **Seed data**: `src/server/db.rs` — new seed logic for journal entries
- **Database**: No schema changes needed — all tables already exist. One SQL fix in `purchase_routes.rs:716`
- **Breaking**: None — all changes are additive
