## 1. Form State

- [x] 1.1 Add `record_payment`, `payment_amount`, and `payment_method` signals to the component state in `src/pages/invoice_create.rs`

## 2. Payment UI Section

- [x] 2.1 Add a "Record Payment" toggle section after the Notes section with conditional payment amount and method fields
- [x] 2.2 Default payment amount to computed invoice total when toggle is enabled, and default method to "Cash"

## 3. Form Submission

- [x] 3.1 Update `save_invoice` handler to pass payment signals to `InvoiceForm` fields
- [x] 3.2 Update `save_and_new` handler to pass payment signals to `InvoiceForm` fields
- [x] 3.3 Reset payment signals when form is cleared (Save & New)

## 4. Verify

- [x] 4.1 Run `cargo check` to confirm no compile errors
