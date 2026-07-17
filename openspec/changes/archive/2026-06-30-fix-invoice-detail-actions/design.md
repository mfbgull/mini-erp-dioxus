## Context

The invoice detail page has Edit and Record Payment buttons that are stubs. The backend is fully implemented:
- `PUT /api/invoices/:id` — update invoice (invoice_routes.rs:184)
- `POST /api/payments` — create payment (payment_routes.rs:60)
- API client methods: `update_invoice()`, `create_payment()` (api.rs)
- Reusable `InvoicePaymentPanel` component exists (invoice_payment_panel.rs)

No invoice edit page or route exists yet.

## Goals / Non-Goals

**Goals:**
- Record Payment button opens a modal with the existing `InvoicePaymentPanel` component
- Edit button navigates to a new invoice edit page
- Invoice edit page pre-fills the create form with existing data and submits via `PUT /api/invoices/:id`

**Non-Goals:**
- Inline editing on the detail page
- Delete button (separate concern, backend route doesn't exist)
- Payment allocation splitting

## Decisions

**Payment: Modal with existing component**: Use the existing `InvoicePaymentPanel` in a modal on the detail page rather than navigating away. This keeps context and is a pattern already established in the codebase.

**Edit: Separate page vs modal**: A separate edit page (`/sales/invoices/:id/edit`) is preferred over a modal because the invoice form is complex (line items, discounts, tax). The create page can be adapted with minimal duplication.

**Edit page approach**: Create `invoice_edit.rs` that loads the existing invoice via `GET /api/invoices/:id`, populates the form, and submits via `PUT /api/invoices/:id`. Reuse the same form layout and calculations from `invoice_create.rs`.

## Risks / Trade-offs

- **[Risk]** Edit page duplicates create page code → Mitigated by extracting shared form logic into a reusable component if needed, but initial implementation can be standalone for simplicity.
- **[Risk]** Payment modal shows stale invoice total after payment → Mitigated by refreshing invoice data after successful payment submission.
