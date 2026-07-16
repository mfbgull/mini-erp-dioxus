## Context

MiniERP Dioxus is a Rust/Dioxus desktop ERP application with SQLite storage. The accounting subsystem has 56 database tables, but several critical write paths are missing:

- `customer_ledger` and `supplier_ledger` tables exist with full schemas but zero INSERT statements anywhere in the codebase
- `journal_entries` / `journal_lines` (double-entry bookkeeping) have no write API and no seed data — all financial reports return zeros
- `salary_payments` has an INSERT endpoint but no LIST endpoint
- `dashboard_layouts` has a full CRUD API but no UI

The existing code follows a consistent pattern: model structs in `models.rs`, route handlers in `src/server/*_routes.rs`, UI pages in `src/pages/`, API client functions in `api.rs`.

## Goals / Non-Goals

**Goals:**
- Make customer and supplier ledger tables functional (INSERT on transactions, running balance calculation)
- Implement journal entry CRUD with auto-journaling for all transaction types
- Add salary payment history endpoint and UI
- Add dashboard layout management UI
- Fix the broken `supplier_po_balance` query (`amount` → `credit`)

**Non-Goals:**
- Rewrite the existing accounting report logic (trial balance, BS, P&L already work if data exists)
- Add new database tables or modify schemas
- Implement multi-currency support
- Add audit trail for journal entry modifications (voided flag already exists in schema)

## Decisions

### D1: Ledger INSERT placement — inline in transaction handlers

**Decision**: INSERT ledger entries directly inside existing transaction handlers (`create_invoice`, `create_payment`, `create_purchase_order`, etc.) rather than via a separate ledger service.

**Rationale**: The existing codebase has no service layer — all business logic lives in route handlers. Adding a service layer just for ledgers would be inconsistent. Each handler already wraps its work in a transaction, so ledger INSERTs naturally participate in the same atomic operation.

**Alternatives considered**:
- Separate `LedgerService` struct: rejected — adds abstraction not present elsewhere in the codebase
- Post-transaction triggers via SQLite triggers: rejected — SQLite triggers can't call Rust code, and the running balance requires reading the previous balance

### D2: Running balance — sequential calculation

**Decision**: Calculate running balance as `previous_balance + debit - credit` by querying the last balance for the account before inserting.

**Rationale**: Simple, predictable, and matches the existing `customer_ledger` schema which has a `balance` column. For high-volume scenarios this could be a bottleneck, but for a desktop ERP the transaction volume is low.

**Alternatives considered**:
- Store balance separately and compute on read: rejected — the schema already has a `balance` column, changing the approach would require schema migration
- Periodic recalculation: rejected — adds complexity for no benefit at this scale

### D3: Journal seed data — backfill from existing transactions

**Decision**: Generate journal entries from seeded invoices, payments, purchases, and expenses during `seed_data()`. Each seeded transaction creates a corresponding journal entry with balanced debit/credit lines.

**Rationale**: Without seed data, all financial reports return zeros on fresh installs. The backfill ensures reports are immediately useful.

**Alternatives considered**:
- Leave reports empty and let users create entries: rejected — poor first-run experience
- Seed random test data: rejected — must be consistent with the seeded transaction amounts

### D4: Auto-journaling — synchronous within transaction

**Decision**: Create journal entries synchronously within each transaction handler, using the same database transaction.

**Rationale**: Journal entries must be consistent with the transactions they record. If the journal INSERT fails, the entire transaction should roll back. Async or deferred journaling would create consistency risks.

**Alternatives considered**:
- Background queue: rejected — adds complexity, risks losing entries if the process crashes before the queue is processed
- Separate transaction: rejected — creates windows where reports show incomplete data

### D5: Supplier ledger UI — mirror customer ledger tab

**Decision**: Add a "Ledger" tab to `supplier_detail.rs` that mirrors the existing customer ledger tab in `customer_detail.rs`.

**Rationale**: The schemas are identical. The UI pattern is already established. Consistency between customer and supplier modules reduces cognitive load.

## Risks / Trade-offs

- **Ledger balance drift**: If a transaction is deleted or modified after ledger entries are created, the running balances become incorrect. Mitigation: Add a recalculation endpoint (already exists for customers at `customer_routes.rs:261`, needs to work with actual data).

- **Journal entry complexity**: Auto-journaling for 6+ transaction types is significant surface area. Each transaction type needs correct account mappings (which account to debit, which to credit). Mitigation: Start with the most common types (invoices, payments, purchases) and add others incrementally.

- **Seed data consistency**: The backfill must produce journal entries that exactly match the seeded transaction amounts. Any mismatch would make trial balance reports show incorrect totals. Mitigation: Write the seed logic carefully and verify that `SUM(debit) = SUM(credit)` after seeding.

- **Performance on large datasets**: Running balance calculation queries the last balance before each INSERT. For very large ledgers this could slow down. Mitigation: Add an index on `(customer_id, id)` and `(supplier_id, id)` for efficient last-row queries. Acceptable for desktop ERP scale.
