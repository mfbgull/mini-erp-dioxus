## 1. Quotation detail view

- [x] 1.1 Replace the `"coming soon"` `rsx!` body in `quotation_detail.rs` with a
  render of `q`: back button, title row with quotation_no + status badge
  (`qstatus_class`), info grid (customer, date, valid_until), line-items table
  (line_no, code, name, qty, unit_price, discount, tax_rate, net_amount), and a
  total row.
- [x] 1.2 Add an action bar: "Convert to Invoice" button wired to
  `client.convert_quotation(id)` in a spawned future with success/error toast
  and navigate back to `/sales/quotations` on success.
- [x] 1.3 Remove the unused `show_delete_modal` signal (no delete endpoint).

## 2. Sales order detail view

- [x] 2.1 Replace the `"coming soon"` `rsx!` body in `sales_order_detail.rs` with
  a render of `so`: back button, title row with order_no + status badge
  (`sostatus_class`), info grid (customer, order_date, delivery_date),
  line-items table (line_no, code, name, qty, unit_price, net_amount), total row.
- [x] 2.2 Add an action bar: "Convert to Invoice" (`convert_sales_order`) and
  "Cancel" (`cancel_sales_order`), each spawned with success/error toast; on
  success navigate back to `/sales/sales-orders`.
- [x] 2.3 Remove the unused `show_delete_modal` signal.

## 3. Verify

- [x] 3.1 `cargo check` compiles with no new errors and fewer warnings (the
  previously-unused `toast`/`navigator` in both files are now consumed).
- [~] 3.2 (needs runtime/GUI) Manually confirm both pages: existing id renders detail, unknown id
  shows the not-found state, and the convert/cancel actions toast + navigate.
