## 1. Record Payment on Detail Page

- [x] 1.1 Add payment modal state and `InvoicePaymentPanel` to `src/pages/invoice_detail.rs`
- [x] 1.2 Wire Record Payment button to open the modal with invoice data
- [x] 1.3 Handle payment submission via `create_payment()` API and refresh invoice on success

## 2. Invoice Edit Page

- [x] 2.1 Create `src/pages/invoice_edit.rs` with form that loads existing invoice via `GET /api/invoices/:id`
- [x] 2.2 Implement form submission via `PUT /api/invoices/:id` and navigate to detail on success
- [x] 2.3 Register `/sales/invoices/:id/edit` route in `src/main.rs`
- [x] 2.4 Wire Edit button on detail page to navigate to edit route

## 3. Verify

- [x] 3.1 Run `cargo check` to confirm no compile errors
