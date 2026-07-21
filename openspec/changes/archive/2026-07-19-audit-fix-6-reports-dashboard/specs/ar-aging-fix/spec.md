## ADDED Requirements

### Requirement: AR aging report aggregates correctly
The `GET /api/reports/ar-aging` endpoint SHALL use `SUM(CASE ...)` for each aging bucket instead of bare `CASE` columns, and SHALL use `JOIN` (not `LEFT JOIN`) to only include customers with outstanding invoices. The `GROUP BY` SHALL aggregate invoice-level amounts correctly.

#### Scenario: AR aging aggregates multiple invoices per customer
- **WHEN** customer A has 3 unpaid invoices: one current (500), one 15 days overdue (300), one 45 days overdue (200)
- **THEN** the report shows customer A with: current=500, days_1_30=300, days_31_60=200, days_61_90=0, days_90_plus=0
- **AND** the total across buckets equals the customer's outstanding balance

#### Scenario: AR aging excludes fully paid customers
- **WHEN** customer B has all invoices fully paid (balance_amount=0)
- **THEN** customer B does not appear in the AR aging report
