## 1. Production Deletion Reversal (F9)

- [x] 1.1 In `delete_production`, SELECT production record (output_item_id, output_quantity, warehouse_id, production_no)
- [x] 1.2 SELECT production_inputs for this production
- [x] 1.3 Create stock movement OUT for output_item_id with output_quantity (reverses the IN from creation)
- [x] 1.4 Update stock_balances: decrease output_item by output_quantity
- [x] 1.5 Update items.current_stock: decrease output_item by output_quantity
- [x] 1.6 For each input: create stock movement IN with input quantity (restores consumed stock)
- [x] 1.7 For each input: update stock_balances: increase input item by input quantity
- [x] 1.8 For each input: update items.current_stock: increase input item by input quantity
- [x] 1.9 DELETE production_inputs, then DELETE productions
- [x] 1.10 Wrap in transaction

## 2. Expense Deletion Reversal (F19)

- [x] 2.1 In `delete_expense`, SELECT journal_entry WHERE reference_type='expense' AND reference_id=expense_id
- [x] 2.2 DELETE journal_lines WHERE journal_entry_id = ?
- [x] 2.3 DELETE journal_entries WHERE id = ?
- [x] 2.4 DELETE expenses WHERE id = ?
- [x] 2.5 Wrap in transaction

## 3. PO Deletion Reversal (F20)

- [x] 3.1 In `delete_purchase_order`, SELECT PO record (supplier_id, total_amount, po_no, id)
- [x] 3.2 INSERT supplier_ledger entry: type='PO_CANCELLATION', debit=0, credit=total_amount
- [x] 3.3 SELECT journal_entry WHERE reference_type='purchase_order' AND reference_id=po_id
- [x] 3.4 DELETE journal_lines WHERE journal_entry_id = ?
- [x] 3.5 DELETE journal_entries WHERE id = ?
- [x] 3.6 DELETE purchase_order_items, then DELETE purchase_orders
- [x] 3.7 Wrap in transaction

## 4. Verification

- [x] 4.1 Compilation passes with `cargo check` (0 errors)
- [ ] 4.2 Test: create production, verify stock changes, delete production — verify stock restored to original
- [ ] 4.3 Test: create expense, verify GL entry, delete expense — verify GL entry removed, trial balance balanced
- [ ] 4.4 Test: create PO, verify supplier ledger, delete PO — verify supplier ledger reversed
