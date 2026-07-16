## 1. Bug Fix — Supplier Balance Query

- [x] 1.1 Fix `purchase_routes.rs:716` — change `SUM(amount)` to `SUM(credit)` in the `supplier_po_balance` query

## 2. Customer Ledger Write Path

- [x] 2.1 Add helper function to calculate running balance for customer ledger (query last balance + new debit/credit)
- [x] 2.2 Insert customer ledger entry in `invoice_routes.rs` `create_invoice` — type='INVOICE', debit=total, credit=0
- [x] 2.3 Insert customer ledger entry in `payment_routes.rs` `create_payment` — type='PAYMENT', debit=0, credit=amount
- [x] 2.4 Insert customer ledger entry in `invoice_routes.rs` `return_invoice` — type='RETURN', debit=0, credit=return_amount
- [x] 2.5 Fix `customer_routes.rs` `recalculate_balances` to compute balance from actual ledger data (not empty table)

## 3. Supplier Ledger Write Path + API

- [x] 3.1 Add helper function to calculate running balance for supplier ledger (mirror customer helper)
- [x] 3.2 Insert supplier ledger entry in `purchase_routes.rs` `create_purchase_order` — type='PURCHASE', debit=total, credit=0
- [x] 3.3 Insert supplier ledger entry in `purchase_routes.rs` `create_goods_receipt` — type='RECEIPT', debit=total, credit=0
- [x] 3.4 Insert supplier ledger entry in `purchase_routes.rs` `create_direct_purchase` — type='PURCHASE', debit=total, credit=0
- [x] 3.5 Add supplier payment endpoint: `POST /api/suppliers/{id}/payments` — inserts into supplier_ledger with type='PAYMENT'
- [x] 3.6 Add supplier ledger list endpoint: `GET /api/suppliers/{id}/ledger` — returns ledger entries with optional date range filter
- [x] 3.7 Add supplier ledger API client function in `api.rs`

## 4. Supplier Ledger UI

- [x] 4.1 Add "Ledger" tab to `supplier_detail.rs` — replace hardcoded `Vec::new()` with API call
- [x] 4.2 Create `components/supplier/supplier_ledger.rs` — table component mirroring customer_ledger.rs
- [x] 4.3 Wire supplier detail page to fetch and display ledger data

## 5. Journal Entry API

- [x] 5.1 Add `POST /api/accounting/journal-entries` — create entry + lines atomically with debit=credit validation
- [x] 5.2 Add `GET /api/accounting/journal-entries` — list with date range, account, reference_type filters
- [x] 5.3 Add `GET /api/accounting/journal-entries/{id}` — detail with lines

## 6. Journal Entry Seed Data

- [x] 6.1 Add journal entry seed logic in `db.rs` `seed_data()` — generate entries for all seeded invoices (debit AR, credit Revenue)
- [x] 6.2 Add journal entry seed logic for seeded payments (debit Cash, credit AR)
- [x] 6.3 Add journal entry seed logic for seeded purchases (debit Inventory/COGS, credit AP)
- [x] 6.4 Add journal entry seed logic for seeded expenses (debit Expense, credit Cash)

## 7. Auto-Journaling

- [x] 7.1 Wire auto-journal in `invoice_routes.rs` `create_invoice` — create journal entry with reference_type='invoice'
- [x] 7.2 Wire auto-journal in `payment_routes.rs` `create_payment` — create journal entry with reference_type='payment'
- [x] 7.3 Wire auto-journal in `purchase_routes.rs` `create_purchase_order` — create journal entry with reference_type='purchase_order'
- [x] 7.4 Wire auto-journal in `purchase_routes.rs` `create_direct_purchase` — create journal entry with reference_type='purchase'
- [x] 7.5 Wire auto-journal in `accounting_routes.rs` `create_expense` — create journal entry with reference_type='expense'
- [x] 7.6 Wire auto-journal in `accounting_routes.rs` `pay_salary` — create journal entry with reference_type='salary'

## 8. Journal Entry UI

- [x] 8.1 Create `src/pages/journal_entry_list.rs` — list view with date range filters, table with Date/Reference/Description/Debit/Credit columns
- [x] 8.2 Create `src/pages/journal_entry_create.rs` — form with entry date, reference type, dynamic lines table, running debit/credit totals, balance validation
- [x] 8.3 Add journal entry routes to navigation/routing
- [x] 8.4 Add journal entry API client functions in `api.rs`

## 9. Salary Payment History

- [x] 9.1 Add `GET /api/employees/{id}/salary-payments` endpoint — list salary payments ordered by date descending
- [x] 9.2 Add salary payment history tab to `employee_detail.rs` — table with Date and Amount columns, total summary
- [x] 9.3 Add salary payment list API client function in `api.rs`
- [ ] 9.4 (Optional) Add `DELETE /api/employees/{id}/salary-payments/{payment_id}` endpoint

## 10. Dashboard Layout UI

- [x] 10.1 Create `src/pages/dashboard_layouts.rs` — list layouts with Name, Active, Created At columns
- [x] 10.2 Add layout create/edit form — name field, blocks JSON editor, save button
- [x] 10.3 Add layout delete functionality — confirmation dialog, API call
- [x] 10.4 Add dashboard layout route to navigation
- [ ] 10.5 Wire `dashboard.rs` to load and apply saved layout if one exists

## 11. Verification

- [ ] 11.1 Verify customer ledger entries are created on invoice/payment/return
- [ ] 11.2 Verify supplier ledger entries are created on PO/receipt/purchase/payment
- [ ] 11.3 Verify journal entries are created for all transaction types
- [ ] 11.4 Verify financial reports (trial balance, BS, P&L) show non-zero values after seeding
- [ ] 11.5 Verify salary payment history is viewable
- [ ] 11.6 Verify dashboard layout management UI works end-to-end
