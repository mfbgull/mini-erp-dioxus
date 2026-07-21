## Why

Six reporting/dashboard bugs produce incorrect or fabricated data:

1. **F10 — Dashboard fabricated:** Weekly sales = today × 5, monthly sales = today × 22. These are hardcoded multipliers, not real queries.
2. **F11 — AR aging GROUP BY bug:** `GROUP BY c.id` with non-aggregated invoice columns produces undefined SQLite behavior — each aging bucket gets one invoice's value, not the sum.
3. **F13 — AP hardcoded to 0:** Dashboard outstanding AP is `"outstanding_ap": 0`.
4. **F15 — P&L COGS methodology:** COGS uses `received_date` filter on batches instead of consumption date, potentially undercounting.
5. **F16 — Supplier balance double-counts:** Includes PO creation AND goods receipt debits, inflating the balance.
6. **F17 — Tax summary double-taxes:** Multiplies tax-inclusive `total_amount` by tax_rate/100.

## What Changes

- **Dashboard:** Query actual weekly and monthly sales from database. Implement outstanding AP card.
- **AR aging:** Fix SQL to properly aggregate per-bucket amounts using SUM with CASE.
- **Supplier balance:** Fix to count only outstanding amounts (POs not yet fully received/paid).
- **Tax summary:** Use pre-tax base for tax calculation, not tax-inclusive total.

## Capabilities

### New Capabilities

- `dashboard-real-data`: Dashboard weekly/monthly sales and AP are computed from actual database queries.
- `ar-aging-fix`: AR aging report correctly aggregates invoice amounts per aging bucket.
- `supplier-balance-fix`: Supplier balance shows only outstanding (unpaid) amounts.
- `tax-summary-fix`: Tax summary calculates tax on pre-tax amounts.

## Impact

- **Server routes**: `dashboard_routes.rs`, `report_routes.rs`, `purchase_routes.rs`
- **Models**: No changes
- **UI pages**: No changes
- **Database**: No schema changes
- **Breaking**: Dashboard values will change from fabricated to real. AR aging values will change. Supplier balances will decrease (no longer double-counted).
