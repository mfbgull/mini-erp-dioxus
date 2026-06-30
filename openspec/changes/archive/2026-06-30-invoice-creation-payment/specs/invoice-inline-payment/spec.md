## ADDED Requirements

### Requirement: Payment toggle on invoice creation form
The invoice creation form SHALL include a "Record Payment" toggle that defaults to off.

#### Scenario: Toggle defaults to off
- **WHEN** the invoice creation page loads
- **THEN** the "Record Payment" toggle is off and no payment fields are visible

#### Scenario: Enabling toggle shows payment fields
- **WHEN** user enables the "Record Payment" toggle
- **THEN** a payment amount field and payment method dropdown become visible

#### Scenario: Disabling toggle hides payment fields
- **WHEN** user disables the "Record Payment" toggle
- **THEN** the payment amount and payment method fields are hidden

### Requirement: Payment amount defaults to invoice total
When the payment toggle is enabled, the payment amount field SHALL default to the computed invoice total.

#### Scenario: Amount pre-filled on toggle enable
- **WHEN** user enables the "Record Payment" toggle and the invoice total is 1500.00
- **THEN** the payment amount field shows 1500.00

#### Scenario: User can override amount
- **WHEN** user changes the payment amount to 500.00
- **THEN** the form submits with payment_amount of 500.00

### Requirement: Payment method selection
The form SHALL provide a dropdown for payment method with sensible defaults.

#### Scenario: Default method is Cash
- **WHEN** user enables the "Record Payment" toggle
- **THEN** the payment method defaults to "Cash"

#### Scenario: User can select different method
- **WHEN** user selects "Bank Transfer" from the payment method dropdown
- **THEN** the form submits with payment_method of "Bank Transfer"

### Requirement: Payment submitted with invoice
When the payment toggle is enabled, the form SHALL submit `record_payment: true` along with the payment amount and method.

#### Scenario: Invoice created with payment
- **WHEN** user submits the form with "Record Payment" enabled, amount 1500, method "Cash"
- **THEN** the backend creates the invoice AND a payment record of 1500 via "Cash"

#### Scenario: Invoice created without payment
- **WHEN** user submits the form with "Record Payment" disabled
- **THEN** the backend creates the invoice with no payment record (existing behavior)
