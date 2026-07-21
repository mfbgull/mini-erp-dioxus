## ADDED Requirements

### Requirement: Line item amount includes tax and discount
When creating or updating an invoice, each invoice item's `amount` column SHALL be computed as: `(quantity × unit_price) - discount + tax`, where discount and tax are derived from the item's `discount_type`, `discount_value`, and `tax_rate` fields.

#### Scenario: Invoice item with tax
- **WHEN** a user creates an invoice item with quantity=10, unit_price=20.00, tax_rate=17, no discount
- **THEN** the stored `amount` = 10 × 20.00 + (200.00 × 17/100) = 234.00
- **AND** NOT 200.00 (the old behavior)

#### Scenario: Invoice item with percentage discount and tax
- **WHEN** a user creates an invoice item with quantity=10, unit_price=20.00, discount_type="percentage", discount_value=10, tax_rate=17
- **THEN** base = 200.00, discount = 20.00, taxable = 180.00, tax = 30.60
- **AND** stored `amount` = 210.60

#### Scenario: Invoice item with fixed discount and no tax
- **WHEN** a user creates an invoice item with quantity=5, unit_price=100.00, discount_type="fixed", discount_value=25.00, tax_rate=0
- **THEN** stored `amount` = 500.00 - 25.00 = 475.00

### Requirement: Invoice total equals sum of line item amounts
The invoice's `total_amount` SHALL be computed as `SUM(invoice_items.amount)` for all items on that invoice. The total is NEVER independently calculated from raw quantities and prices.

#### Scenario: Invoice total matches line items
- **WHEN** an invoice has 3 line items with amounts 234.00, 105.30, and 50.00
- **THEN** the invoice's `total_amount` = 389.30
- **AND** `SUM(ii.amount) WHERE ii.invoice_id = X` returns 389.30

#### Scenario: Invoice with header-level discount
- **WHEN** an invoice has line items totaling 1000.00 and a header discount of 5% (50.00)
- **THEN** the invoice's `total_amount` = 950.00
- **AND** the `balance_amount` is initialized to 950.00

### Requirement: Invoice detail view shows tax and discount breakdown
The invoice detail API response SHALL include per-item tax amount and discount amount so the UI can display a full breakdown. The response SHALL include: subtotal (sum of base amounts before tax/discount), total discount, total tax, and grand total.

#### Scenario: Invoice detail response includes breakdown
- **WHEN** a user views invoice detail for an invoice with 2 items
- **THEN** the response includes: subtotal, total_discount, total_tax, total_amount
- **AND** each line item includes: base_amount, discount_amount, tax_amount, amount
