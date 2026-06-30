## Why

The invoice creation page (`/sales/invoices/new`) has no payment option. Users must create an invoice first, then navigate to a separate payments page to record payment. The backend already supports inline payment during invoice creation via `record_payment`, `payment_amount`, and `payment_method` fields on `InvoiceForm`, but the frontend hardcodes these to "no payment".

## What Changes

- Add a "Record Payment" toggle to the invoice creation form
- When enabled, show payment amount (defaulting to invoice total) and payment method fields
- Pass the payment fields to the backend on submit so a payment record is created alongside the invoice

## Capabilities

### New Capabilities
- `invoice-inline-payment`: UI and form logic for recording a payment at invoice creation time

### Modified Capabilities

## Impact

- `src/pages/invoice_create.rs` — Add payment toggle, amount, and method fields to the form
- No backend changes required — `InvoiceForm` already carries the necessary fields
- No model changes required
