## ADDED Requirements

### Requirement: Edit button navigates to edit page
The invoice detail page Edit button SHALL navigate to `/sales/invoices/:id/edit`.

#### Scenario: Click edit navigates
- **WHEN** user clicks the "Edit" button on the invoice detail page
- **THEN** the browser navigates to `/sales/invoices/{id}/edit`

### Requirement: Edit page loads existing invoice data
The invoice edit page SHALL load the existing invoice via `GET /api/invoices/:id` and pre-fill all form fields.

#### Scenario: Form pre-filled on load
- **WHEN** the invoice edit page loads for invoice #1001
- **THEN** customer, dates, line items, discount, tax rate, and notes are pre-populated from the existing invoice

#### Scenario: Invoice not found
- **WHEN** the edit page loads for a non-existent invoice ID
- **THEN** an error message is shown with a back button

### Requirement: Edit page submits updates via API
The invoice edit page SHALL submit changes via `PUT /api/invoices/:id` and navigate back to the detail page on success.

#### Scenario: Successful update
- **WHEN** user modifies the invoice and clicks Save
- **THEN** the form submits to `PUT /api/invoices/{id}`, shows a success toast, and navigates to `/sales/invoices/{id}`

#### Scenario: Update validation error
- **WHEN** user submits with no customer or no items
- **THEN** a validation error is shown and no API call is made
