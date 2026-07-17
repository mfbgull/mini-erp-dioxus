# MiniERP Dioxus — Incomplete Work Plan

Accurate as of 2026-07-17, verified against source (not the old audit).
Build state: **compiles clean, 0 errors, 298 warnings** (mostly unused
vars/mut, non-snake-case `App`). Git tree clean.

> The previous version of this file described the ledger/journal systems as
> "entirely dead." That is **no longer true** — the accounting system
> (commit `0a8a106`) implemented those write paths. This file reflects the
> current, verified state.

---

## What is ACTUALLY incomplete

### 🔴 P1 — Two detail pages fetch data then throw it away

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

### 🟡 P2 — Deferred actions (button present, action toasts "coming soon")

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

### 🟡 P2 — AR aging uses a hardcoded estimate

`src/calculations/customer.rs:114` — `calculate_average_days_to_pay()` returns
a flat `30.0` instead of computing from actual payment-vs-invoice dates.

- [ ] Compute `SUM(payment_date - invoice_date) / paid_count` from payment
  allocations instead of the 30-day placeholder.

---

### 🟢 P3 — Dead / unused tables (schema only)

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

### 🟢 P3 — Housekeeping

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

## Priority summary

```
┌───────────────────────────────────────────────────────────┐
│ P1  ██████████  Render quotation + SO detail pages        │
│ P2  ████        Wire deferred action buttons (or remove)  │
│ P2  ███         AR aging real calculation                 │
│ P3  ██          Remove 4 dead tables + orphan JOIN        │
│ P3  █           Delete stubs.rs, clear warnings           │
└───────────────────────────────────────────────────────────┘
```

Only the **P1 detail pages** are genuinely unfinished features. Everything
else is deferred actions, a placeholder calculation, or cleanup.
