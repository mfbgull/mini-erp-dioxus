## ADDED Requirements

### Requirement: Supplier balance shows only outstanding amounts
The `GET /api/purchase-orders/suppliers/{id}/balance` endpoint SHALL return the supplier's outstanding balance computed from the supplier ledger: `SUM(debit) - SUM(credit)`. It SHALL NOT double-count by summing PO totals and receipt totals separately.

#### Scenario: Supplier balance reflects actual outstanding
- **WHEN** a supplier has 2 POs totaling 10000.00 and has been paid 6000.00
- **THEN** the balance = 4000.00 (not 10000.00 or 16000.00)
