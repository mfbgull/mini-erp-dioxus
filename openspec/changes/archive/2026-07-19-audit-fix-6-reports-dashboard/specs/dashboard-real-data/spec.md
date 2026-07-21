## ADDED Requirements

### Requirement: Dashboard weekly sales uses real data
The `GET /api/dashboard/sales-summary` endpoint SHALL compute `this_week` by querying actual invoices from Monday of the current week to today, filtered by `status != 'Cancelled'`. It SHALL NOT use `today_sales * 5.0`.

#### Scenario: Weekly sales reflects actual invoices
- **WHEN** there are 3 invoices this week totaling 1500.00 and today's sales are 500.00
- **THEN** the `this_week` value is 1500.00 (not 2500.00)

### Requirement: Dashboard monthly sales uses real data
The `GET /api/dashboard/sales-summary` endpoint SHALL compute `this_month` by querying actual invoices for the current calendar month, filtered by `status != 'Cancelled'`. It SHALL NOT use `today_sales * 22.0`.

#### Scenario: Monthly sales reflects actual invoices
- **WHEN** there are 20 invoices this month totaling 50000.00 and today's sales are 2000.00
- **THEN** the `this_month` value is 50000.00 (not 44000.00)

### Requirement: Dashboard outstanding AP is computed
The `GET /api/dashboard/summary` endpoint SHALL compute `outstanding_ap` from the supplier ledger: `SUM(debit) - SUM(credit)` for all supplier_ledger entries. It SHALL NOT be hardcoded to 0.

#### Scenario: Dashboard shows real AP
- **WHEN** there are outstanding purchase orders totaling 15000.00 and supplier payments of 8000.00
- **THEN** `outstanding_ap` = 7000.00 (not 0)
