# MiniERP Dioxus — Incomplete Work Plan + Comprehensive Audit

Accurate as of 2026-07-19, verified against source.
Build state: **compiles clean, 0 errors, 298 warnings** (mostly unused
vars/mut, non-snake-case `App`). Git tree clean.

> The previous version of this file described the ledger/journal systems as
> "entirely dead." That is **no longer true** — the accounting system
> (commit `0a8a106`) implemented those write paths. This file reflects the
> current, verified state.

---

# PART 1: INCOMPLETE WORK (from prior audit)

## What is ACTUALLY incomplete

### P1 — Two detail pages fetch data then throw it away

Both pages wire up the API call, parse the record + line items into a full
detail struct, handle loading and not-found states correctly — then render a
`"coming soon"` empty-state instead of the data they just loaded.

```
┌──────────────────────────────────────────────────────────────┐
│  PAGE                       FETCH   PARSE   RENDER            │
├──────────────────────────────────────────────────────────────┤
│  quotation_detail.rs         ✓       ✓      ✗  "coming soon"  │
│  sales_order_detail.rs       ✓       ✓      ✗  "coming soon"  │
└──────────────────────────────────────────────────────────────┘
```

- `src/pages/quotation_detail.rs:203` — builds `to_quotation_detail(q, items)`,
  then body is `p { "Quotation detail view — coming soon" }`.
- `src/pages/sales_order_detail.rs:197` — builds full `SalesOrderDetail { .. }`,
  then body is `p { "Sales order detail view — coming soon" }`.

**Needed:** render the loaded struct. The plumbing exists; only the view body
is missing. Use an existing detail page (e.g. `invoice_detail.rs`,
`purchase_order_detail.rs`) as the layout reference.

- [ ] Render quotation header + items table in `quotation_detail.rs`
- [ ] Render sales order header + items table in `sales_order_detail.rs`
- [ ] Note: server `SalesOrder` only returns `total_amount` — no
  subtotal/discount/tax breakdown (see `ponytail:` comment at
  `sales_order_detail.rs:143`). Either accept the flat total or extend the
  server response.

---

### P2 — Deferred actions (button present, action toasts "coming soon")

These are deliberate stubs — the UI control exists but the action is not
wired. Decide per-item whether to implement or remove the button.

| Location | Button(s) stubbed |
|----------|-------------------|
| `production_detail.rs:255,261` | Edit Mode, Update Progress |
| `direct_purchase_detail.rs:138-140` | Edit, Goods Receipt, Print |
| `purchase_order_detail.rs:145` | Print |
| `role_detail.rs:191` | Edit Role ("not yet available") |

- [ ] Production: wire Edit + Update Progress, or hide buttons
- [ ] Direct purchase: wire Edit / Goods Receipt / Print, or hide
- [ ] PO / Quotation print views
- [ ] Role editing (or remove the button)

---

### P2 — AR aging uses a hardcoded estimate

`src/calculations/customer.rs:114` — `calculate_average_days_to_pay()` returns
a flat `30.0` instead of computing from actual payment-vs-invoice dates.

- [ ] Compute `SUM(payment_date - invoice_date) / paid_count` from payment
  allocations instead of the 30-day placeholder.

---

### P3 — Dead / unused tables (schema only)

Verified: referenced **only** in `db.rs` (schema), never queried by app code.

| Table | Note |
|-------|------|
| `item_locations` | zero queries/models/API/UI |
| `material_consumption` | zero queries/models/API/UI |
| `employee_documents` | zero queries/models/API/UI |
| `work_orders` | schema only + one orphaned `LEFT JOIN` at `report_routes.rs:755` |

- [ ] Decide: implement or drop these 4 tables. `work_orders` also has a dead
  JOIN to clean up.

---

### P3 — Housekeeping

- [ ] `src/pages/stubs.rs` — now empty ("all routes have real implementations").
  Delete the file and its `mod` reference.
- [ ] `invoice_drafts` — used only by `mobile_routes.rs` (mobile POS). Confirm
  mobile POS is in scope; if not, remove. If yes, it works as-is.
- [ ] `cargo fix --lib -p mini-erp` clears 295 of 298 warnings. Fix the
  remaining few by hand (`App` → snake case is cosmetic and can stay).

---

## Verified DONE (was flagged incomplete in the old plan)

Do **not** re-do these — confirmed present in source:

- ✅ `customer_ledger` INSERT write path — `invoice_routes.rs:190,241,429`,
  `payment_routes.rs:117`
- ✅ `supplier_ledger` INSERT write path — `purchase_routes.rs:293,503,1041`
- ✅ `journal_entries` + `journal_lines` write path + auto-journaling —
  `invoice_routes.rs`, `payment_routes.rs`, `purchase_routes.rs`,
  `inventory_routes.rs`, `accounting_routes.rs`
- ✅ Journal seed data — `db.rs:1700-1757`
- ✅ Old "broken query" bug (`SUM(amount)` on supplier_ledger) — **already
  fixed** to `SUM(credit)` at `purchase_routes.rs:893`
- ✅ Salary payments LIST endpoint — `accounting_routes.rs:405`
  (`GET /api/employees/{id}/salary-payments`)
- ✅ UI pages exist — `dashboard_layouts.rs`, `journal_entry_list.rs`,
  `journal_entry_create.rs`

---

# PART 2: COMPREHENSIVE BUSINESS LOGIC & FINANCIAL AUDIT (2026-07-19)

## Executive Summary

**Overall Health Score: 28/100**

| Dimension | Score |
|---|---|
| Financial Integrity | 15/100 |
| Data Integrity | 35/100 |
| Report Accuracy | 25/100 |
| Inventory Accuracy | 30/100 |
| Production Accuracy | 25/100 |
| Code Quality | 40/100 |

This ERP application has **systemic financial correctness problems** that make
it unsuitable for production use in any financial capacity. The issues range
from architectural (all monetary calculations use `f64` despite a `Money`
type existing) to critical business logic bugs (invoice line items don't match
invoice totals, payments can be deleted without reversing effects, and dashboard
figures are fabricated).

---

## Audit Findings

### CRITICAL — F1: Money Type Exists But Is Completely Unused

- **Severity:** CRITICAL
- **Module:** All
- **Affected files:** Every server route file, `src/models.rs`
- **Database tables:** ALL tables with REAL columns for monetary values

`money.rs` defines `Money` based on `rust_decimal::Decimal` with exact
arithmetic. **Zero imports of `crate::money::*` exist in any server route
file.** Every monetary calculation uses raw `f64`:

```
invoice_routes.rs:106   — let mut total_amount = 0.0;
invoice_routes.rs:108   — let amount = item.quantity * item.unit_price;
models.rs:198           — pub current_stock: f64
models.rs:409           — pub total_amount: f64
```

Every SQL column for money is `REAL` (SQLite floating-point). The `money.rs`
module has comprehensive `ToSql`/`FromSql` implementations but they convert
to/from `f64` at the storage boundary anyway, defeating Decimal precision.

- **Expected:** All monetary calculations use `Money` (Decimal) type.
- **Actual:** All monetary calculations use f64.
- **Impact:** Every financial number is subject to IEEE 754 rounding errors
  that compound over time.
- **Confidence:** 100%

---

### CRITICAL — F2: Invoice Line Item Amounts Don't Include Tax But Invoice Total Does

- **Severity:** CRITICAL
- **Module:** Invoice System
- **Affected files:** `src/server/invoice_routes.rs:106-141`
- **Database tables:** `invoices`, `invoice_items`

In `create_invoice`, the total includes per-item tax:

```rust
let amount = item.quantity * item.unit_price;
let tax = amount * (item.tax_rate.unwrap_or(0.0) / 100.0);
total_amount += amount + tax;  // total_amount INCLUDES tax
```

But the line item stored amount does NOT include tax:

```rust
let amount = item.quantity * item.unit_price;  // NO tax
// INSERT INTO invoice_items ... amount  // stores amount WITHOUT tax
```

**Worked example:** 10 items @ $20 with 17% tax:
- Line item `amount` stored = $200.00
- Invoice `total_amount` = $234.00
- **Discrepancy: $34 invisible in line items**

- **Expected:** Line item amounts match invoice total, or both are computed
  consistently.
- **Actual:** Invoice total is tax-inclusive; line items are tax-exclusive.
- **Confidence:** 100%

---

### CRITICAL — F3: No Transaction Wrapping (No Atomicity)

- **Severity:** CRITICAL
- **Module:** All
- **Affected files:** `invoice_routes.rs`, `payment_routes.rs`,
  `inventory_routes.rs`, `manufacturing_routes.rs`, `purchase_routes.rs`

Invoice creation performs **12+ separate database operations** without a
transaction. Each `.ok()` silently swallows errors. If any step fails midway,
the database is left in an inconsistent state (e.g., invoice exists with no
stock deduction, payment exists without balance update).

There is zero usage of `db.execute_batch("BEGIN...COMMIT")` or transaction
objects anywhere in the codebase.

- **Expected:** All multi-step workflows wrapped in database transactions.
- **Actual:** Each SQL statement commits independently; errors silently swallowed.
- **Confidence:** 100%

---

### CRITICAL — F4: Payment Deletion Doesn't Reverse Anything

- **Severity:** CRITICAL
- **Module:** Payment System
- **Affected files:** `src/server/payment_routes.rs:156-163`
- **Database tables:** `payments`, `invoices`, `customer_ledger`, `customers`,
  `journal_entries`, `journal_lines`

```rust
async fn delete_payment(...)
    let result = db.execute("DELETE FROM payments WHERE id = ?1", [id]);
    // That's it. No reversals.
```

**Missing reversals:**
1. Restore invoice `paid_amount` and `balance_amount`
2. Recalculate invoice status
3. Reverse customer ledger entry
4. Restore `customers.current_balance`
5. Reverse journal entry (debit Cash -> credit AR)
6. Delete `payment_allocations`

- **Expected:** Full reversal of all effects.
- **Actual:** Only the payment record is deleted; all related data becomes
  permanently inconsistent.
- **Confidence:** 100%

---

### HIGH — F5: Invoice Update Doesn't Reverse Stock Movements

- **Severity:** HIGH
- **Module:** Invoice System
- **Affected files:** `src/server/invoice_routes.rs:281-344`
- **Database tables:** `stock_movements`, `stock_balances`, `items`

`update_invoice` deletes old `invoice_items` and creates new ones, but never
touches stock. If item quantities change from 5 to 10, stock only went down by
5 (from creation), not 10. Permanent inventory drift.

- **Expected:** Reverse old stock movements, create new ones for updated quantities.
- **Actual:** Stock movements never updated; inventory drifts permanently.
- **Confidence:** 100%

---

### HIGH — F6: Sales Order -> Invoice Conversion Doesn't Copy Line Items

- **Severity:** HIGH
- **Module:** Sales
- **Affected files:** `src/server/sales_routes.rs:177-199`

`convert_sales_order` creates an invoice with the SO's `total_amount` but
never inserts any `invoice_items` rows. The invoice has a header total but no
detail rows.

- **Expected:** Copy all SO line items to `invoice_items`.
- **Actual:** Invoice created with no line items; only header total populated.
- **Confidence:** 100%

---

### HIGH — F7: Quotation -> SO -> Invoice Chain Loses All Line Items

- **Severity:** HIGH
- **Module:** Sales
- **Affected files:** `src/server/sales_routes.rs:352-374`

Same pattern as F6: `convert_quotation` creates a SO with the quotation's
`total_amount` but never inserts `sales_order_items` rows. The entire
conversion chain (Quotation -> SO -> Invoice) loses line items at each step.

- **Confidence:** 100%

---

### HIGH — F8: Cancellation Doesn't Reverse Stock, Payments, or Ledger

- **Severity:** HIGH
- **Module:** Invoice System
- **Affected files:** `src/server/invoice_routes.rs:346-354`

```rust
async fn cancel_invoice(...)
    let result = db.execute("UPDATE invoices SET status = 'Cancelled' ...");
    // That's it. Only status change.
```

Missing reversals: stock movements (OUT), customer ledger entries, customer
`current_balance`, journal entries, paid amounts/payments handling.

- **Confidence:** 100%

---

### HIGH — F9: Production Deletion Doesn't Reverse Stock

- **Severity:** HIGH
- **Module:** Manufacturing
- **Affected files:** `src/server/manufacturing_routes.rs:365-373`

Deleting a completed production run removes the production record but stock
changes remain: items produced stay in inventory and consumed materials are
not restored.

- **Confidence:** 100%

---

### HIGH — F10: Dashboard Weekly/Monthly Sales Are Fabricated

- **Severity:** HIGH
- **Module:** Dashboard
- **Affected files:** `src/server/dashboard_routes.rs:62-68`

```rust
"this_week": today_sales * 5.0,    // FABRICATED: today x 5
"this_month": today_sales * 22.0,  // FABRICATED: today x 22
```

Weekly and monthly sales are today's value multiplied by magic numbers, not
actual database queries.

- **Confidence:** 100%

---

### HIGH — F11: AR Aging Report Has SQL GROUP BY Bug

- **Severity:** HIGH
- **Module:** Reports
- **Affected files:** `src/server/report_routes.rs:73-87`

```sql
SELECT c.id, c.customer_name, c.current_balance,
    CASE WHEN i.due_date >= ?1 THEN i.balance_amount ELSE 0 END as current,
    ...
FROM customers c LEFT JOIN invoices i ON ...
WHERE c.is_active = 1 GROUP BY c.id HAVING current_balance > 0
```

`GROUP BY c.id` with non-aggregated `i.balance_amount` columns produces
undefined behavior in SQLite. Each aging bucket only gets ONE invoice's
`balance_amount`, not the sum.

- **Confidence:** 95%

---

### MEDIUM — F12: Invoice Line-Level Discounts Are Dead Fields

- **Severity:** MEDIUM
- **Module:** Invoice System
- **Affected files:** `src/server/invoice_routes.rs:133-141`

`invoice_items` has `discount_type`/`discount_value` columns that are stored
but never factored into the line item `amount` calculation:
```rust
let amount = item.quantity * item.unit_price;  // no discount applied
```

- **Confidence:** 100%

---

### MEDIUM — F13: Dashboard Outstanding AP is Hardcoded to Zero

- **Severity:** MEDIUM
- **Module:** Dashboard
- **Affected files:** `src/server/dashboard_routes.rs:43`

```rust
"outstanding_ap": 0,  // Hardcoded
```

No supplier balance or outstanding purchase bill is ever queried.

- **Confidence:** 100%

---

### MEDIUM — F14: No Validation That Payment Doesn't Exceed Invoice Balance

- **Severity:** MEDIUM
- **Module:** Payment System
- **Affected files:** `src/server/payment_routes.rs:60-139`

The only validation is `amount <= 0`. Users can overpay an invoice by any
amount, creating negative balances.

- **Confidence:** 100%

---

### MEDIUM — F15: P&L COGS Calculation May Undercount

- **Severity:** MEDIUM
- **Module:** Reports
- **Affected files:** `src/server/report_routes.rs:414-419`

COGS is calculated only from FIFO batch consumption within a date range on
`received_date`. Production consumption and sales from older batches may be
missed. The date filter is on batch creation date, not consumption date.

- **Confidence:** 65%

---

### MEDIUM — F16: Supplier Balance Calculation Double-Counts

- **Severity:** MEDIUM
- **Module:** Purchasing
- **Affected files:** `src/server/purchase_routes.rs:885-898`

Supplier balance sums ALL non-cancelled POs (not just unpaid). Supplier ledger
tracks debits from both PO creation AND goods receipt, double-counting the
obligation.

- **Confidence:** 85%

---

### MEDIUM — F17: Tax Summary Double-Taxes

- **Severity:** MEDIUM
- **Module:** Reports
- **Affected files:** `src/server/report_routes.rs:616-647`

```sql
SUM(i.total_amount * i.tax_rate / 100) as tax_amount
```

`total_amount` already includes tax (F2), so multiplying by `tax_rate/100`
applies tax on the tax-inclusive amount, overcounting.

- **Confidence:** 90%

---

### MEDIUM — F18: No Negative Stock Prevention

- **Severity:** MEDIUM
- **Module:** Inventory
- **Affected files:** Multiple routes

No route checks `stock_balances.quantity >= quantity` before deducting.
Negative stock is silently allowed.

- **Confidence:** 100%

---

### MEDIUM — F19: Expense Deletion Doesn't Reverse Journal Entry

- **Severity:** MEDIUM
- **Module:** Accounting
- **Affected files:** `src/server/accounting_routes.rs:230-238`

Deleting an expense doesn't reverse the journal entry (debit Expense, credit
Cash), leaving a permanent GL imbalance.

- **Confidence:** 100%

---

### MEDIUM — F20: PO Deletion Doesn't Reverse Supplier Ledger

- **Severity:** MEDIUM
- **Module:** Purchasing
- **Affected files:** `src/server/purchase_routes.rs:358-367`

Supplier ledger had a debit entry when PO was created. Deleting the PO doesn't
reverse it.

- **Confidence:** 100%

---

### MEDIUM — F21: Sequence Number Performance Bottleneck

- **Severity:** MEDIUM
- **Module:** All (document numbering)
- **Affected files:** Multiple route files

```rust
let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM invoices", ...);
```

While the `Mutex<Connection>` prevents race conditions on sequence numbers, it
serializes the entire application through a single SQLite connection, creating
a severe performance bottleneck under load.

- **Confidence:** 80%

---

### LOW — F22: Journal Entry Balance Uses f64 Comparison

- **Severity:** LOW
- **Module:** Accounting
- **Affected files:** `src/server/accounting_routes.rs:450`

```rust
if (total_debit - total_credit).abs() > 0.01
```

Uses f64 tolerance. Should use Decimal comparison.

- **Confidence:** 80%

---

### LOW — F23: SQL Injection Pattern (Not Currently Exploitable)

- **Severity:** LOW
- **Module:** Reports
- **Affected files:** `src/server/report_routes.rs:293-351`

Uses `format!` for SQL parameter interpolation. Not exploitable because
`Path<i64>` enforces integer parsing, but dangerous pattern.

- **Confidence:** 95%

---

### LOW — F24: Duplicate Sales Returns Endpoint

- **Severity:** LOW
- **Module:** Sales
- **Affected files:** `invoice_routes.rs:462-479`, `sales_routes.rs:387-405`

Both execute the exact same SQL query. Redundant but not incorrect.

- **Confidence:** 100%

---

## Cross-Module Validation Matrix

| Module Flow | Status | Issues |
|---|---|---|
| Quotation -> SO -> Invoice | BROKEN | Line items lost at each conversion step |
| Invoice Creation -> Stock | PARTIAL | Stock deducted but no atomicity |
| Invoice Edit -> Stock | BROKEN | Stock not adjusted for new quantities |
| Invoice Cancel -> Stock | BROKEN | Stock not reversed |
| Invoice Return -> Stock | OK | Stock correctly restored |
| Payment -> Invoice Balance | OK (create) | BROKEN (delete: no reversal) |
| Production -> Stock | OK (create) | BROKEN (delete: no reversal) |
| PO -> GRN -> Stock | OK (create) | PO deletion doesn't reverse ledger |
| Customer Ledger Balance | FRAGILE | Sequential read-write, manual recalc exists |
| P&L Revenue | OK | Direct SUM of invoice totals |
| P&L COGS | UNCERTAIN | FIFO-based, may undercount |
| Dashboard Sales | FABRICATED | Weekly/monthly are today x magic number |
| Dashboard AP | MISSING | Hardcoded to 0 |
| AR Aging | BROKEN | GROUP BY aggregation bug |

---

## Report Validation Matrix

| Report | Verified | Issues Found | Confidence |
|---|---|---|---|
| Sales Summary | Partial | Daily OK, weekly/monthly fabricated | 100% |
| P&L | Partial | Revenue OK, COGS methodology questionable | 65% |
| Balance Sheet | Partial | Uses journal_lines, appears correct | 70% |
| Trial Balance | Yes | Appears correct if journal entries are correct | 75% |
| AR Aging | NO | GROUP BY bug produces wrong per-bucket values | 95% |
| AR Summary | Yes | Straightforward SUM queries | 85% |
| Tax Summary | NO | Double-taxing bug | 90% |
| Cash Flow | Partial | Inflows from payments OK, outflows from purchases only | 70% |
| Stock Valuation | Partial | Uses standard_cost, not actual cost | 70% |
| FIFO Valuation | Yes | FIFO batch logic appears correct | 80% |
| Customer Statements | Yes | Uses ledger directly | 85% |
| Sales by Customer | Yes | Correct aggregation | 85% |
| Sales by Item | Yes | Correct aggregation | 85% |
| Purchase Summary | Yes | Simple aggregation | 85% |
| DSO | Yes | Straightforward calculation | 80% |
| Inventory Movement | Yes | Direct query of stock_movements | 85% |
| Stock History | Partial | Running balance logic is correct | 80% |
| Production Summary | Yes | Simple aggregation | 85% |

---

# PART 3: PRIORITIZED REMEDIATION PLAN

## Priority Summary

```
┌─────────────────────────────────────────────────────────────────────────┐
│ P0-CRIT  ██  F1   Use Money/Decimal type everywhere (architectural)    │
│ P0-CRIT  ██  F2   Fix invoice line item tax inconsistency              │
│ P0-CRIT  ██  F3   Wrap all multi-step ops in transactions              │
│ P0-CRIT  ██  F4   Full reversal on payment deletion                    │
│ P1-HIGH  █   F5   Reverse stock on invoice edit                       │
│ P1-HIGH  █   F6   Copy line items during SO -> Invoice conversion     │
│ P1-HIGH  █   F7   Copy line items during Quotation -> SO conversion  │
│ P1-HIGH  █   F8   Full reversal on invoice cancellation               │
│ P1-HIGH  █   F9   Reverse stock on production deletion                │
│ P1-HIGH  █   F10  Fix dashboard (query real data, not fabricated)     │
│ P1-HIGH  █   F11  Fix AR aging GROUP BY bug                           │
│ P2-MED       F12  Implement or remove line-level discount fields      │
│ P2-MED       F13  Implement outstanding AP dashboard card             │
│ P2-MED       F14  Add payment <= invoice balance validation           │
│ P2-MED       F15  Fix P&L COGS date range methodology                │
│ P2-MED       F16  Fix supplier balance double-counting                │
│ P2-MED       F17  Fix tax summary double-taxing                       │
│ P2-MED       F18  Add negative stock prevention                       │
│ P2-MED       F19  Reverse journal on expense deletion                 │
│ P2-MED       F20  Reverse supplier ledger on PO deletion              │
│ P2-MED       F21  Consider connection pooling / remove Mutex bottleneck│
│ P3-LOW       F22  Use Decimal for journal balance check               │
│ P3-LOW       F23  Use parameterized queries instead of format!        │
│ P3-LOW       F24  Remove duplicate sales returns endpoint             │
└─────────────────────────────────────────────────────────────────────────┘
```

## Remediation Phases

### Phase 1: Foundation (F1, F3) — 2-3 days

1. Make `money.rs` `ToSql`/`FromSql` store as TEXT (Decimal string) not REAL
2. Convert all model monetary fields from `f64` to `Money`
3. Add transaction wrappers (`BEGIN`/`COMMIT`/`ROLLBACK`) to every multi-step
   route handler
4. Replace `.ok()` error swallowing with proper error propagation inside
   transactions

### Phase 2: Invoice Correctness (F2, F5, F8, F12) — 1-2 days

5. Fix invoice total to be computed FROM line items (not independently)
6. Add line item discount calculations
7. On invoice edit: reverse old stock, create new stock for updated quantities
8. On invoice cancel: reverse stock, ledger, GL entries

### Phase 3: Conversion Chain (F6, F7) — 1 day

9. Copy line items during Quotation -> SO conversion
10. Copy line items during SO -> Invoice conversion

### Phase 4: Deletion Reversals (F4, F9, F19, F20) — 2 days

11. Full reversal on payment deletion
12. Full reversal on production deletion
13. Full reversal on expense deletion
14. Full reversal on PO deletion (supplier ledger)

### Phase 5: Reports & Dashboard (F10, F11, F13, F15, F16, F17) — 1-2 days

15. Fix dashboard to query actual weekly/monthly sales
16. Implement outstanding AP dashboard card
17. Fix AR aging SQL aggregation
18. Fix tax summary double-taxing
19. Fix supplier balance calculation
20. Review P&L COGS methodology

### Phase 6: Validation & Edge Cases (F14, F18) — 1 day

21. Add payment <= invoice balance validation
22. Add negative stock prevention (optional, configurable)

### Phase 7: Cleanup (F22, F23, F24 + prior P2/P3 items) — 1 day

23. Use Decimal for journal balance check
24. Use parameterized queries
25. Remove duplicate endpoints
26. Wire deferred UI buttons or remove them
27. Implement quotation/SO detail pages

---

## Risk Assessment for Production Deployment

**Risk Level: VERY HIGH — DO NOT DEPLOY FOR PRODUCTION USE**

This application will produce incorrect financial data under normal operation.
The floating-point arithmetic, missing atomicity, incomplete reversals, and
fabricated dashboard numbers mean:

- Financial reports will be wrong
- Customer balances will drift from reality
- Inventory will not reconcile with physical stock
- GL entries will not balance after deletions/cancellations
- Audit trails will be incomplete

The application is suitable as a prototype or learning project but requires
significant remediation before any production financial use. Estimated effort
to reach production readiness: **8-12 developer-days** following the phases above.
