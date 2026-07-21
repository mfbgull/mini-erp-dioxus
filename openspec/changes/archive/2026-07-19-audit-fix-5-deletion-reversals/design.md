## Context

Each deletion handler needs to look up the record before deletion, then reverse all side effects, then delete the record. The pattern is consistent across all three.

## Goals / Non-Goals

**Goals:**
- Production deletion reverses all stock effects
- Expense deletion reverses GL entry
- PO deletion reverses supplier ledger and GL entry
- All wrapped in transactions (Change 1)

**Non-Goals:**
- Soft-delete with audit trail
- Reversal of GRN (goods receipt) — that's a separate, more complex workflow
- Cascading deletion of related documents

## Decisions

### D1: Production reversal pattern

**Decision:** Before deleting a production:
1. SELECT production (output_item_id, output_quantity, warehouse_id)
2. SELECT production_inputs (item_id, quantity, warehouse_id)
3. Reverse output: stock movement IN (-output_quantity), stock_balances += output_quantity, current_stock += output_quantity
4. Reverse each input: stock movement OUT (-quantity → actually IN, restoring stock), stock_balances += input_quantity, current_stock += input_quantity
5. Delete stock_movements referencing this production
6. Delete production_inputs, then production

**Wait — correction:** The output was an IN movement (stock added). To reverse: create an OUT movement for output_quantity (stock removed). The inputs were OUT movements (stock deducted). To reverse: create IN movements for each input quantity (stock restored).

### D2: Expense reversal pattern

**Decision:** Before deleting an expense:
1. SELECT the journal_entry where reference_type='expense' AND reference_id=expense_id
2. DELETE journal_lines WHERE journal_entry_id = ?
3. DELETE journal_entries WHERE id = ?
4. DELETE expenses WHERE id = ?

### D3: PO reversal pattern

**Decision:** Before deleting a PO:
1. SELECT PO (supplier_id, total_amount, po_no)
2. INSERT supplier_ledger reversal: type='PO_CANCELLATION', debit=0, credit=total_amount
3. SELECT journal_entry where reference_type='purchase_order' AND reference_id=po_id
4. DELETE journal_lines, DELETE journal_entries
5. DELETE purchase_order_items, then purchase_orders
