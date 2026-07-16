# MiniERP Dioxus — Incomplete Features Plan

Database table audit findings. Revised after deep codebase exploration.

---

## Critical Finding: Ledger System Is Entirely Dead

Both `customer_ledger` and `supplier_ledger` have **zero INSERT statements** anywhere in the codebase. The original audit flagged only `supplier_ledger` as partial — but `customer_ledger` is equally broken. Both have SELECT endpoints and (for customer) UI, but nothing ever writes to either table. Customer balance is maintained via direct UPDATE on `customers.current_balance`, bypassing the ledger entirely.

```
┌──────────────────────────────────────────────────────────┐
│                  LEDGER SYSTEM STATUS                    │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  customer_ledger          supplier_ledger                │
│  ┌──────────────┐         ┌──────────────┐              │
│  │ Schema ✓     │         │ Schema ✓     │              │
│  │ Model ✓      │         │ Model ✓      │              │
│  │ SELECT ✓     │         │ SELECT ✗     │              │
│  │ INSERT ✗     │         │ INSERT ✗     │              │
│  │ UI ✓ (read)  │         │ UI ✗         │              │
│  │ API client ✓ │         │ API client ✗ │              │
│  └──────────────┘         └──────────────┘              │
│                                                          │
│  BOTH tables always contain zero rows.                   │
│  Balance queries SUM over empty tables.                  │
│  customer_detail.rs reads empty results gracefully.      │
│  supplier_detail.rs hardcoded to Vec::new().             │
│                                                          │
│  Additionally: supplier_po_balance query at              │
│  purchase_routes.rs:716 references column `amount`      │
│  which does NOT EXIST in the schema (has debit/credit). │
│  Silently returns 0.0 via unwrap_or.                    │
└──────────────────────────────────────────────────────────┘
```

---

## Critical Finding: Journal System Has No Seed Data and No Write Path

The original plan noted "only seed data exists" for `journal_entries` — but there is **zero seed data**. The `seed_data()` function in `db.rs:1084-1673` seeds everything EXCEPT journal entries. The financial reports (trial balance, balance sheet, P&L) query `journal_lines` JOINs that are always empty. All account balances resolve to 0.0.

```
┌──────────────────────────────────────────────────────────┐
│                JOURNAL SYSTEM STATUS                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  chart_of_accounts    journal_entries    journal_lines   │
│  ┌──────────────┐     ┌──────────────┐   ┌────────────┐ │
│  │ Schema ✓     │     │ Schema ✓     │   │ Schema ✓   │ │
│  │ Seed: 17 accts│     │ Seed: NONE   │   │ Seed: NONE │ │
│  │ Balance: 0.0 │     │ No INSERT API│   │ No INSERT  │ │
│  └──────────────┘     └──────────────┘   └────────────┘ │
│                                                          │
│  Reports (trial balance, BS, P&L) all return zeros.      │
│  create_expense() → INSERT into expenses, NO journal    │
│  pay_salary() → INSERT into salary_payments, NO journal │
│  No invoice/payment/purchase → journal entry wiring.     │
└──────────────────────────────────────────────────────────┘
```

---

## UNUSED TABLES (dead code — remove or implement)

| Table | Schema Location | Notes |
|-------|----------------|-------|
| `item_locations` | `db.rs:332` | Zero queries, zero models, zero API, zero UI |
| `work_orders` | `db.rs:717` | Schema only. One orphaned LEFT JOIN in `report_routes.rs:701` |
| `material_consumption` | `db.rs:733` | Zero queries, zero models, zero API, zero UI |
| `employee_documents` | `db.rs:874` | Zero queries, zero models, zero API, zero UI |

---

## PARTIALLY IMPLEMENTED TABLES

### 1. `customer_ledger` — write path missing

**What exists:**
- Schema: `db.rs:371` (migration 013)
- Model: `models.rs:378` — `CustomerLedgerEntry`
- SELECT endpoints: `customer_routes.rs:194` (list), `customer_routes.rs:220` (statement), `customer_routes.rs:269` (balance recalc)
- UI: `customer_detail.rs:232` reads ledger; `components/customer/customer_ledger.rs` renders it
- API client: `api.rs:1629`

**What's broken:**
- **Zero INSERT statements** — nothing writes to this table
- Balance recalculate (`customer_routes.rs:261`) does `SUM(debit) - SUM(credit)` on an always-empty table
- Customer `current_balance` is maintained via direct UPDATE, not via ledger

**What's needed:**
- [ ] INSERT on invoice creation (`invoice_routes.rs`) — debit entry for invoice amount
- [ ] INSERT on payment received (`payment_routes.rs`) — credit entry for payment amount
- [ ] INSERT on invoice return/cancellation — credit entry
- [ ] Calculate running_balance on each INSERT
- [ ] Fix `recalculate_balances()` to work from actual ledger data

---

### 2. `supplier_ledger` — write path missing + broken query

**What exists:**
- Schema: `db.rs:582` (migration 025) — identical structure to `customer_ledger`
- Model: `models.rs:664` — `SupplierLedgerEntry` (same fields, `supplier_id` instead of `customer_id`)
- One broken query: `purchase_routes.rs:716` references `SUM(amount)` but column is `debit`/`credit`

**What's missing:**
- Zero INSERT statements
- No dedicated SELECT endpoint (no list/get ledger endpoint)
- No UI page (`supplier_detail.rs:156` hardcodes `Vec::new()`)
- No API client function
- `supplier_detail.rs:137` hardcodes `current_balance: 0.0`

**What's needed:**
- [ ] Fix broken query at `purchase_routes.rs:716` — `amount` → `credit`
- [ ] INSERT on purchase order creation — debit entry
- [ ] INSERT on goods receipt — debit entry
- [ ] INSERT on direct purchase — debit entry
- [ ] INSERT on supplier payment (currently no payment endpoint exists)
- [ ] Add API: `GET /api/suppliers/{id}/ledger` (list with date range)
- [ ] Add API: `GET /api/suppliers/{id}/statement` (formatted)
- [ ] Add UI: ledger tab in `supplier_detail.rs`
- [ ] Add API client function in `api.rs`

---

### 3. `journal_entries` + `journal_lines` — no write path, no seed data

**What exists:**
- Schema: `db.rs:801` / `db.rs:814` (migrations 038/039)
- Model: `models.rs` — `JournalEntry`, `JournalLine`
- Read-only: `accounting_routes.rs` and `report_routes.rs` JOIN on these tables for balance calculations

**What's missing:**
- Zero seed data (the `seed_data()` function skips these tables entirely)
- Zero INSERT statements — no creation API
- All financial reports return zeros
- No auto-journaling for any business transaction

**What's needed:**
- [ ] Seed initial journal entries from existing seed data (invoices, payments, purchases, expenses) so reports aren't blank
- [ ] Add API: `POST /api/accounting/journal-entries` (create entry + lines atomically, debits must equal credits)
- [ ] Add API: `GET /api/accounting/journal-entries` (list with date range, account, reference filters)
- [ ] Add API: `GET /api/accounting/journal-entries/{id}` (detail with lines)
- [ ] Add UI: `src/pages/journal_entry_list.rs` (list with filters)
- [ ] Add UI: `src/pages/journal_entry_create.rs` (form with dynamic lines)
- [ ] Auto-journal wiring for: invoices, payments, purchase orders, goods receipts, expenses, salary payments
- [ ] Validation: debits = credits on every entry

---

### 4. `salary_payments` — write-only, no journal entry

**What exists:**
- Schema: `db.rs:885`, model: `models.rs` — `SalaryPayment`, `SalaryPaymentForm`
- INSERT endpoint: `POST /api/employees/{id}/salary` at `accounting_routes.rs:338`

**What's missing:**
- No LIST endpoint — past payments can never be viewed
- No UI page for salary payment history
- No journal entry creation when salary is paid
- `employee_detail.rs` has no salary history tab

**What's needed:**
- [ ] Add API: `GET /api/employees/{id}/salary-payments` (list with pagination)
- [ ] Add UI: salary payment history tab in `employee_detail.rs`
- [ ] Wire journal entry creation on salary payment
- [ ] Optional: `DELETE` endpoint for corrections

---

### 5. `dashboard_layouts` — API exists, no UI

**What exists:**
- Schema: `db.rs:1017`, model: `models.rs`
- Full CRUD API: `dashboard_routes.rs` (list, create, update, delete)

**What's missing:**
- No UI page to manage layouts
- Dashboard page doesn't load or apply saved layouts

**What's needed:**
- [ ] Add UI: `src/pages/dashboard_layouts.rs`
- [ ] Wire dashboard.rs to load and render saved layouts
- [ ] Add route in navigation

---

### 6. `invoice_drafts` — mobile POS only

**What exists:**
- Schema: `db.rs:1030`
- CRUD in `mobile_routes.rs` (INSERT, SELECT, UPDATE, DELETE)
- No Rust model struct

**Decision needed:** Is mobile POS in scope? If yes → add model struct. If no → remove.

---

## BUG FOUND

**`purchase_routes.rs:716`** — broken query:
```sql
SELECT COALESCE(SUM(amount), 0) FROM supplier_ledger ...
```
The `supplier_ledger` schema has `debit` and `credit` columns, not `amount`. This query always returns 0.0 via the `unwrap_or(0.0)` fallback. Should be:
```sql
SELECT COALESCE(SUM(credit), 0) FROM supplier_ledger WHERE supplier_id = ?1 AND type = 'PAYMENT'
```

---

## IMPLEMENTATION PRIORITY (revised)

```
┌─────────────────────────────────────────────────────────────┐
│  PRIORITY          TASK                         EFFORT      │
├─────────────────────────────────────────────────────────────┤
│  P1  ██████████    Ledger write paths           6-8 hrs     │
│      (both customer + supplier ledger INSERTs + balance)    │
│                                                             │
│  P1  ██████████    Journal write path           8-10 hrs    │
│      (CRUD API + seed data + auto-journaling + UI)          │
│                                                             │
│  P2  ████          salary_payments read path    1-2 hrs     │
│                                                             │
│  P2  ███           dashboard_layouts UI         1-2 hrs     │
│                                                             │
│  P3  ██            supplier_ledger bug fix      15 min      │
│      (amount → credit in purchase_routes.rs:716)            │
│                                                             │
│  P3  █             Unused tables cleanup        30 min      │
│                                                             │
│  P3  ?             invoice_drafts decision      —           │
└─────────────────────────────────────────────────────────────┘
```

---

## ESTIMATED SCOPE (revised)

| Task | Hours | Description |
|------|-------|-------------|
| Ledger write paths | 6-8 | INSERT triggers for customer_ledger + supplier_ledger from invoices, payments, purchases. Running balance calculation. |
| Journal system | 8-10 | CRUD API, seed data backfill, auto-journaling for 6+ transaction types, validation, 2 UI pages |
| salary_payments read | 1-2 | List endpoint + UI tab |
| dashboard_layouts UI | 1-2 | Management page + wiring |
| Bug fix | 0.25 | Fix broken column reference |
| Cleanup | 0.5 | Remove 4 dead tables or implement |
| **Total** | **~17-23 hrs** | |

---

## ARCHITECTURAL NOTES

- **Both ledgers are mirrors** — `customer_ledger` and `supplier_ledger` have identical schemas (only FK column differs). The implementation should be symmetrical: same INSERT pattern, same balance calculation, same API shape.
- **Journal entries are the backbone** — without them, all financial reports are meaningless. The seed data backfill is critical to avoid blank reports on existing databases.
- **Auto-journaling should be event-driven** — each transaction handler (invoice, payment, purchase, expense, salary) should insert journal lines as part of its transaction, not as a separate step.
- **The broken query at purchase_routes.rs:716** is a silent failure — it returns 0.0 and the UI displays a misleading balance. Fix this immediately regardless of priority.
