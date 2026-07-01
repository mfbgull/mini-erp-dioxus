## 1. Wire dashboard quick-action buttons

- [x] Add `use_navigator()` to `src/pages/dashboard.rs`
- [x] Wire "New Invoice" onclick → `navigator.push("/sales/invoices/new")`
- [x] Wire "New Item" onclick → `navigator.push("/inventory/items/new")`
- [x] Wire "New Customer" onclick → `navigator.push("/customers/new")`
- [x] Wire "View Reports" onclick → `navigator.push("/reports")`

## 2. Wire AR Aging Print button

- [x] Import `trigger_print` from `crate::pages::print_shared` in `src/pages/ar_aging.rs`
- [x] Replace `onclick: move |_| {}` with `onclick: move |_| trigger_print()`

## 3. Implement discount scope toggle

- [x] Add `use_signal(|| DiscountScope::BeforeTax)` for discount scope in `src/pages/invoice_create.rs`
- [x] Replace hardcoded `Discount { scope: DiscountScope::BeforeTax, ... }` with the signal
- [x] Wire button onclick to toggle between `BeforeTax` / `AfterTax`
- [x] Update button text dynamically ("Before Tax" / "After Tax")
- [x] Fix `scope_btn_class` to use the toggle state instead of `discount_pct > 0`

## 4. Wire supplier New PO buttons

- [x] Replace "Coming Soon" toast handlers with `navigator.push("/purchases/orders/new")` for both "New PO" and "New Purchase Order" buttons in `src/pages/supplier_detail.rs`

## 5. Verify

- [x] Run `cargo check` to confirm no compile errors
