## Why

The invoice detail page (`/sales/invoices/:id`) has Edit and Record Payment buttons that are non-functional stubs showing "coming soon" toasts. The backend APIs (`PUT /api/invoices/:id`, `POST /api/payments`) and API client methods already exist, but the frontend hasn't wired them up.

## What Changes

- **Record Payment**: Add a payment modal on the invoice detail page using the existing `InvoicePaymentPanel` component, calling `POST /api/payments`
- **Edit Invoice**: Add an invoice edit route and page that pre-fills the create form with existing invoice data, calling `PUT /api/invoices/:id`

## Capabilities

### New Capabilities
- `invoice-detail-payment`: Record payment from invoice detail page via modal
- `invoice-edit`: Edit existing invoices via a dedicated edit page

### Modified Capabilities

## Impact

- `src/pages/invoice_detail.rs` — Add payment modal with `InvoicePaymentPanel`, wire up Record Payment button
- `src/pages/invoice_edit.rs` — New invoice edit page (adapted from create page)
- `src/main.rs` — Add `/sales/invoices/:id/edit` route
- `src/components/layout/sidebar.rs` — No changes needed (existing invoice nav)
