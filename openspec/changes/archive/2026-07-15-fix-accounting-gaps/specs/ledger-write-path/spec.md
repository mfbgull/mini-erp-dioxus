## ADDED Requirements

### Requirement: Customer ledger entries are created on invoice
The system SHALL INSERT a row into `customer_ledger` with `type='INVOICE'`, `debit=invoice_total`, `credit=0` when an invoice is created via `POST /api/invoices`. The `reference_no` SHALL be the invoice number. The `balance` column SHALL be calculated as `previous_balance + debit - credit`.

#### Scenario: Invoice creation populates customer ledger
- **WHEN** a user creates an invoice for customer ID 5 with total 1000.00
- **THEN** a row is inserted into `customer_ledger` with `customer_id=5`, `type='INVOICE'`, `debit=1000.00`, `credit=0`, and `balance` equal to the customer's previous ledger balance plus 1000.00

### Requirement: Customer ledger entries are created on payment received
The system SHALL INSERT a row into `customer_ledger` with `type='PAYMENT'`, `debit=0`, `credit=payment_amount` when a payment is created via `POST /api/payments`. The `reference_no` SHALL be the payment reference number.

#### Scenario: Payment received populates customer ledger
- **WHEN** a user records a payment of 500.00 from customer ID 5
- **THEN** a row is inserted into `customer_ledger` with `customer_id=5`, `type='PAYMENT'`, `debit=0`, `credit=500.00`, and `balance` equal to the customer's previous ledger balance minus 500.00

### Requirement: Customer ledger entries are created on invoice return
The system SHALL INSERT a row into `customer_ledger` with `type='RETURN'`, `debit=0`, `credit=return_amount` when an invoice return is processed via `POST /api/invoices/{id}/return`.

#### Scenario: Invoice return populates customer ledger
- **WHEN** a user processes a return of 200.00 on invoice for customer ID 5
- **THEN** a row is inserted into `customer_ledger` with `customer_id=5`, `type='RETURN'`, `debit=0`, `credit=200.00`

### Requirement: Supplier ledger entries are created on purchase order
The system SHALL INSERT a row into `supplier_ledger` with `type='PURCHASE'`, `debit=po_total`, `credit=0` when a purchase order is created via `POST /api/purchase-orders`.

#### Scenario: Purchase order populates supplier ledger
- **WHEN** a user creates a purchase order for supplier ID 3 with total 2500.00
- **THEN** a row is inserted into `supplier_ledger` with `supplier_id=3`, `type='PURCHASE'`, `debit=2500.00`, `credit=0`

### Requirement: Supplier ledger entries are created on goods receipt
The system SHALL INSERT a row into `supplier_ledger` with `type='RECEIPT'`, `debit=receipt_total`, `credit=0` when a goods receipt is created.

#### Scenario: Goods receipt populates supplier ledger
- **WHEN** a user records a goods receipt for supplier ID 3 with total 2500.00
- **THEN** a row is inserted into `supplier_ledger` with `supplier_id=3`, `type='RECEIPT'`, `debit=2500.00`, `credit=0`

### Requirement: Supplier ledger entries are created on direct purchase
The system SHALL INSERT a row into `supplier_ledger` with `type='PURCHASE'`, `debit=purchase_total`, `credit=0` when a direct purchase is created via `POST /api/purchases`.

#### Scenario: Direct purchase populates supplier ledger
- **WHEN** a user creates a direct purchase for supplier ID 3 with total 800.00
- **THEN** a row is inserted into `supplier_ledger` with `supplier_id=3`, `type='PURCHASE'`, `debit=800.00`, `credit=0`

### Requirement: Supplier ledger entries are created on payment to supplier
The system SHALL INSERT a row into `supplier_ledger` with `type='PAYMENT'`, `debit=0`, `credit=payment_amount` when a payment is recorded to a supplier. A supplier payment endpoint SHALL be added.

#### Scenario: Supplier payment populates supplier ledger
- **WHEN** a user records a payment of 1000.00 to supplier ID 3
- **THEN** a row is inserted into `supplier_ledger` with `supplier_id=3`, `type='PAYMENT'`, `debit=0`, `credit=1000.00`

### Requirement: Broken supplier balance query is fixed
The query at `purchase_routes.rs:716` that references `SUM(amount)` SHALL be changed to `SUM(credit)` to match the `supplier_ledger` schema which has `debit` and `credit` columns.

#### Scenario: Supplier balance query returns correct value
- **WHEN** a supplier has 3 payment entries totaling 1500.00 in `supplier_ledger`
- **THEN** the `supplier_po_balance` endpoint returns `paid=1500.00` (not 0.0)

### Requirement: Supplier ledger API endpoint
The system SHALL expose `GET /api/suppliers/{id}/ledger` that returns all ledger entries for a supplier, ordered by `transaction_date` descending, with optional `from_date` and `to_date` query parameters.

#### Scenario: List supplier ledger entries
- **WHEN** a user requests the ledger for supplier ID 3
- **THEN** the system returns a JSON array of `SupplierLedgerEntry` objects with all fields populated

### Requirement: Supplier ledger UI tab
The `supplier_detail.rs` page SHALL include a "Ledger" tab that displays supplier ledger entries in a table with columns: Date, Type, Reference, Debit, Credit, Balance. The hardcoded `Vec::new()` at line 156 SHALL be replaced with an API call to fetch actual ledger data.

#### Scenario: Supplier detail shows ledger tab
- **WHEN** a user views the detail page for supplier ID 3
- **THEN** the page displays a "Ledger" tab with the supplier's ledger entries
