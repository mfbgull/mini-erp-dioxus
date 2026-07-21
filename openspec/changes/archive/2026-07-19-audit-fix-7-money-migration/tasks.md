## 1. Fix Money ToSql/FromSql

- [x] 1.1 Update `Money::to_sql` to store as TEXT (Decimal::to_string()) instead of REAL (to_f64())
- [x] 1.2 Update `Money::from_sql` to try TEXT first, fall back to REAL
- [x] 1.3 Fix deprecated `is_negative()` → `is_sign_negative()`, `signum()` return type
- [x] 1.4 Add `rust_decimal` to Cargo.toml dependencies
- [x] 1.5 Add `pub mod money;` to lib.rs (with cfg-gated rusqlite impls)

## 2. Convert Model Fields

- [ ] 2.1 Convert `Item` monetary fields: standard_cost, selling_price → Money
- [ ] 2.2 Convert `Invoice` fields: total_amount, paid_amount, balance_amount, returned_amount, discount_value → Money
- [ ] 2.3 Convert `InvoiceItem` fields: unit_price, amount, discount_value → Money
- [ ] 2.4 Convert `Payment` field: amount → Money
- [ ] 2.5 Convert `SalesOrder` field: total_amount → Money
- [ ] 2.6 Convert `SalesOrderItem` fields: unit_price, amount → Money
- [ ] 2.7 Convert `PurchaseOrder` field: total_amount → Money
- [ ] 2.8 Convert `PurchaseOrderItem` fields: unit_cost, amount → Money
- [ ] 2.9 Convert `DirectPurchase` fields: unit_cost, total_cost → Money
- [ ] 2.10 Convert `StockMovement` field: unit_cost → Money
- [ ] 2.11 Convert `BomItem` field: unit_cost → Money
- [ ] 2.12 Convert `Production` fields: overhead_cost, unit_cost, total_material_cost → Money
- [ ] 2.13 Convert `Expense` field: amount → Money
- [ ] 2.14 Convert `SalaryPayment` field: amount → Money
- [ ] 2.15 Convert `Employee` field: salary → Money
- [ ] 2.16 Convert `Customer` fields: credit_limit, credit_balance, current_balance, opening_balance, total_invoiced, total_paid → Money
- [ ] 2.17 Convert `AccountBalance` fields: debit, credit, balance → Money
- [ ] 2.18 Convert `JournalLine` fields: debit, credit → Money
- [ ] 2.19 Convert `DashboardSummary` fields: total_revenue, total_expenses, outstanding_ar, outstanding_ap, stock_value → Money

## 3. Convert Route Handlers

- [ ] 3.1 Convert `invoice_routes.rs` — all arithmetic to Money
- [ ] 3.2 Convert `payment_routes.rs` — all arithmetic to Money
- [ ] 3.3 Convert `sales_routes.rs` — all arithmetic to Money
- [ ] 3.4 Convert `purchase_routes.rs` — all arithmetic to Money
- [ ] 3.5 Convert `manufacturing_routes.rs` — all arithmetic to Money
- [ ] 3.6 Convert `inventory_routes.rs` — all arithmetic to Money
- [ ] 3.7 Convert `accounting_routes.rs` — all arithmetic to Money
- [ ] 3.8 Convert `dashboard_routes.rs` — all arithmetic to Money
- [ ] 3.9 Convert `report_routes.rs` — all arithmetic to Money

## 4. Convert UI Pages

- [ ] 4.1 Update invoice_detail.rs to display Money values
- [ ] 4.2 Update invoice_create.rs to handle Money in forms
- [ ] 4.3 Update invoice_edit.rs
- [ ] 4.4 Update customer_detail.rs
- [ ] 4.5 Update supplier_detail.rs
- [ ] 4.6 Update dashboard.rs
- [ ] 4.7 Update all report pages

## 5. Convert API Client

- [ ] 5.1 Update api.rs deserialization for Money fields
- [ ] 5.2 Verify all API client functions work with Money types

## 6. Verification

- [x] 6.1 Compilation passes with `cargo check` (0 errors)
- [x] 6.2 Money module compiles on both WASM and native targets
- [x] 6.3 ToSql stores as TEXT (Decimal string)
- [x] 6.4 FromSql handles both TEXT and REAL formats
- [ ] 6.5 Unit test: ToSql/FromSql round-trip preserves precision
- [ ] 6.6 Integration test: create invoice, verify total_amount stored as TEXT
- [ ] 6.7 Integration test: read old REAL data, verify Money conversion works
- [ ] 6.8 Verify all monetary reports produce correct results
- [ ] 6.9 Verify no f64 arithmetic remains in monetary calculations (grep audit)

## Notes

Tasks 2-5 (model field conversion, route handler conversion, UI conversion, API client conversion) are **deferred** to a future change. The current implementation:
- Adds `rust_decimal` dependency
- Adds `money` module to lib.rs with cfg-gated rusqlite impls
- Fixes ToSql to store as TEXT (Decimal string) for new data
- Fixes FromSql to read both TEXT (new) and REAL (legacy) formats
- Existing f64 model fields continue to work — Money can be used incrementally
- No schema migration needed — dual-format support handles transition gracefully
