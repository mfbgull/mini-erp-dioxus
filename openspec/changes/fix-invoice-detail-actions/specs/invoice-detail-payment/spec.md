## ADDED Requirements

### Requirement: Record Payment button opens payment modal
The invoice detail page Record Payment button SHALL open a modal containing the `InvoicePaymentPanel` component pre-filled with the invoice's customer and remaining balance.

#### Scenario: Button opens modal
- **WHEN** user clicks the "Record Payment" button on the invoice detail page
- **THEN** a modal opens displaying the payment form with the invoice's customer pre-selected

#### Scenario: Modal pre-fills amount
- **WHEN** the payment modal opens for an invoice with balance 1500.00
- **THEN** the payment amount field is pre-filled with 1500.00

### Requirement: Payment submission creates payment record
The payment modal SHALL submit to `POST /api/payments` and refresh the invoice data on success.

#### Scenario: Successful payment
- **WHEN** user submits the payment form with amount 1500.00 via Cash
- **THEN** a payment record is created, the modal closes, and the invoice detail refreshes showing updated balance

#### Scenario: Payment validation error
- **WHEN** user submits with amount 0 or no method selected
- **THEN** the form shows a validation error and no API call is made
