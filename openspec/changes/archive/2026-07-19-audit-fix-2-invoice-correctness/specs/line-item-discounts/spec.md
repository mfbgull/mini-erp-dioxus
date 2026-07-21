## ADDED Requirements

### Requirement: Line item discount is applied during amount calculation
When creating or updating invoice items, the `discount_type` and `discount_value` fields from `InvoiceItemForm` SHALL be factored into the stored `amount`. The discount is applied to the base amount (quantity × unit_price) before tax is calculated.

#### Scenario: Percentage discount on line item
- **WHEN** an item has quantity=10, unit_price=20.00, discount_type="percentage", discount_value=10
- **THEN** base = 200.00, discount = 20.00, amount_before_tax = 180.00
- **AND** stored amount = 180.00 (plus tax if applicable)

#### Scenario: Fixed discount on line item
- **WHEN** an item has quantity=5, unit_price=100.00, discount_type="fixed", discount_value=25.00
- **THEN** base = 500.00, discount = 25.00
- **AND** stored amount = 475.00 (plus tax if applicable)

#### Scenario: No discount on line item
- **WHEN** an item has quantity=10, unit_price=20.00, no discount_type or discount_value
- **THEN** stored amount = 200.00 (plus tax if applicable)

### Requirement: Invoice detail displays discount per item
The invoice detail API response SHALL include each line item's `discount_type`, `discount_value`, and the computed `discount_amount` so the UI can display the discount applied to each item.

#### Scenario: Invoice detail shows item discounts
- **WHEN** a user views an invoice with 2 items, one with 10% discount and one with no discount
- **THEN** the response includes discount_type and discount_value for each item
- **AND** the UI can display "10% off" for the first item and nothing for the second
