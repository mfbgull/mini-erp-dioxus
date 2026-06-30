## Context

The invoice creation page (`src/pages/invoice_create.rs`) builds an `InvoiceForm` and submits it to `POST /api/invoices`. The `InvoiceForm` struct already has `record_payment: Option<bool>`, `payment_amount: Option<f64>`, and `payment_method: Option<String>` fields. The backend `create_invoice` handler (in `db.rs`) checks these fields and creates a `payments` row + updates invoice balances when `record_payment` is `Some(true)`. However, the frontend always hardcodes `record_payment: Some(false)` with `None` for amount and method.

## Goals / Non-Goals

**Goals:**
- Add a "Record Payment" toggle to the invoice creation form
- Show payment amount (defaulting to invoice total) and payment method when toggle is on
- Submit payment fields to the backend so a payment is created atomically with the invoice

**Non-Goals:**
- Split payments across multiple methods
- Partial payment at creation (always full amount by default)
- Modifying the backend API or data model

## Decisions

**Toggle + inline fields vs separate payment step**: A toggle with inline fields is simpler than a two-step wizard. The backend already supports this pattern — no API changes needed.

**Default payment amount to invoice total**: When the toggle is enabled, the payment amount field pre-fills with the computed invoice total. Users can override for partial payment.

**Payment method dropdown**: Use the same methods the payment system supports (Cash, Bank Transfer, Cheque, etc.). A simple `<select>` element is sufficient.

**Placed after notes section**: The payment section goes after the existing notes/totals area, visually grouped as an optional "Payment" section with a clear heading.

## Risks / Trade-offs

- **[Risk]** User enables payment but leaves amount empty → Mitigated by defaulting amount to invoice total and validating non-zero on submit.
- **[Risk]** Payment method not selected → Mitigated by defaulting to "Cash" when toggle is enabled.
