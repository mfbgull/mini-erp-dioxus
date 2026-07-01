## ADDED Requirements

### Requirement: Discount scope button toggles between Before Tax / After Tax
The discount scope button on the invoice create form SHALL toggle between "Before Tax" and "After Tax" mode each time it is clicked. The active mode SHALL determine how the header-level discount is applied in the total calculation.

#### Scenario: Default state is "Before Tax"
- **GIVEN** the user is creating a new invoice
- **THEN** the discount scope button SHALL display "Before Tax"
- **AND** the discount SHALL be subtracted before computing tax

#### Scenario: Clicking toggles to "After Tax"
- **GIVEN** the discount scope button shows "Before Tax"
- **WHEN** the user clicks it
- **THEN** the button SHALL display "After Tax"
- **AND** subsequent calculations SHALL apply the discount after computing tax

#### Scenario: Clicking again toggles back
- **GIVEN** the discount scope button shows "After Tax"
- **WHEN** the user clicks it
- **THEN** the button SHALL display "Before Tax"
- **AND** calculations SHALL revert to discount-before-tax behavior

### Requirement: Totals reflect active discount scope
The computed invoice totals (subtotal, discount amount, taxable amount, tax amount, total) SHALL update immediately when the discount scope is toggled.

#### Scenario: Before Tax calculation
- **GIVEN** subtotal is 1000, discount is 10%, tax rate is 16%
- **WHEN** discount scope is "Before Tax"
- **THEN** discount_amount = 100, taxable_amount = 900, tax_amount = 144, total = 1044

#### Scenario: After Tax calculation
- **GIVEN** subtotal is 1000, discount is 10%, tax rate is 16%
- **WHEN** discount scope is "After Tax"
- **THEN** taxable_amount = 1000, tax_amount = 160, discount_amount = 100, total = 1060

### Non-functional

- The discount scope state MUST use `use_signal` (must not use a simple variable)
- The `Discount` struct passed to `compute_invoice_metrics` MUST reflect the active scope
