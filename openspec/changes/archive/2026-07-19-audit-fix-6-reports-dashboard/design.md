## Context

The dashboard and reports have a mix of fabricated data, SQL bugs, and calculation errors. Each fix is independent.

## Goals / Non-Goals

**Goals:**
- Dashboard shows real weekly/monthly sales from database
- Dashboard shows real outstanding AP
- AR aging report aggregates correctly
- Supplier balance shows only outstanding amounts
- Tax summary uses pre-tax amounts

**Non-Goals:**
- New report types
- Report export/PDF
- P&L COGS methodology overhaul (F15 is uncertain, defer to later)
- Real-time dashboard updates

## Decisions

### D1: Dashboard weekly/monthly queries

**Decision:** Replace `today_sales * 5.0` with actual date-range queries:
```sql
-- This week (Monday to today)
SELECT COALESCE(SUM(total_amount), 0) FROM invoices
WHERE invoice_date >= date('now', 'weekday 0', '-6 days')
AND status != 'Cancelled'

-- This month
SELECT COALESCE(SUM(total_amount), 0) FROM invoices
WHERE strftime('%Y-%m', invoice_date) = strftime('%Y-%m', 'now')
AND status != 'Cancelled'
```

### D2: AR aging fix

**Decision:** Replace the broken GROUP BY query with a properly aggregated query that uses SUM(CASE ...) for each aging bucket:
```sql
SELECT c.id, c.customer_name, c.current_balance,
    SUM(CASE WHEN i.due_date >= ? THEN i.balance_amount ELSE 0 END) as current,
    SUM(CASE WHEN i.due_date < ? AND i.due_date >= date(?, '-30 days') THEN i.balance_amount ELSE 0 END) as days_1_30,
    ...
FROM customers c
JOIN invoices i ON c.id = i.customer_id AND i.status IN ('Unpaid','Partially Paid')
WHERE c.is_active = 1
GROUP BY c.id
HAVING SUM(i.balance_amount) > 0
```

**Key change:** Use `JOIN` instead of `LEFT JOIN` (we only want customers with outstanding invoices), and use `SUM(CASE ...)` for proper aggregation.

### D3: Outstanding AP

**Decision:** Query supplier balances from the supplier_ledger:
```sql
SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM supplier_ledger
```

### D4: Supplier balance fix

**Decision:** The current `supplier_po_balance` counts all non-cancelled POs. Fix to only count POs that haven't been fully received (sum of PO amounts minus sum of receipt amounts, or use the ledger balance directly).

### D5: Tax summary fix

**Decision:** The tax calculation should use the pre-tax amount. Since line items now include tax in their amount (Change 2), the tax amount per item is: `item.amount - (item.quantity × item.unit_price - item.discount)`. Alternatively, compute from the stored `tax_rate` and `quantity × unit_price`.

## Risks / Trade-offs

- **Existing invoice data:** Old invoices have tax-inclusive totals but line items without tax. The tax summary fix may produce different results for old vs new invoices. Mitigation: Note this as a known limitation for pre-fix data.
