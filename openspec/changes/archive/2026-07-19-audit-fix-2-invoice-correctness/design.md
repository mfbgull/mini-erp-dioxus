## Context

The invoice system has four bugs rooted in the same issue: the invoice total is computed independently from line items rather than derived from them. Additionally, stock operations only happen on creation — edits and cancellations don't adjust inventory.

## Goals / Non-Goals

**Goals:**
- Invoice `total_amount` = SUM(line_item amounts) where each line item amount includes tax and discount
- Editing an invoice correctly adjusts stock for all changed items
- Cancelling an invoice fully reverses all financial and inventory effects
- Line-level discounts are functional

**Non-Goals:**
- Multi-currency support
- Tax reporting beyond what's already in the tax_summary report
- Partial invoice cancellation (only full cancellation)

## Decisions

### D1: Invoice total derived from line items

**Decision:** Compute `total_amount` by summing each line item's computed amount (quantity × unit_price - discount + tax). The invoice header stores the final total; line items store the detail.

**Line item amount formula:**
```
base = quantity × unit_price
discount = if discount_type == "percentage" { base × discount_value / 100 }
           else if discount_type == "fixed" { discount_value }
           else { 0 }
taxable = base - discount
tax = taxable × tax_rate / 100
amount = taxable + tax  (or just taxable if tax is stored separately)
```

**Invoice total:** `SUM(line_item.amount)` for all items on the invoice.

**Rationale:** This is the standard ERP approach. The total is always a derived value, never independently computed. This eliminates the discrepancy between header and detail.

### D2: Stock reversal on edit

**Decision:** On invoice edit, look up the OLD invoice items before deletion, compute the stock delta for each item (old_qty - new_qty), and apply stock movements accordingly.

**Pattern:**
1. SELECT old invoice_items (quantities)
2. DELETE old invoice_items
3. INSERT new invoice_items
4. For each item: compute delta = old_qty - new_qty
   - If delta > 0 (quantity decreased): create IN movement (stock returned)
   - If delta < 0 (quantity increased): create OUT movement (more stock issued)
   - If delta == 0: no stock change
5. UPDATE stock_balances and items.current_stock

**Rationale:** This handles all cases: quantity changes, item additions, and item removals. Using delta movements is cleaner than "reverse all, re-create all" because it minimizes stock_movement records.

### D3: Full reversal on cancellation

**Decision:** Cancelling an invoice performs these reversals:
1. Reverse stock movements: for each invoice item with `returned_qty < quantity`, create IN movement for `(quantity - returned_qty)` units
2. Update stock_balances: add back `(quantity - returned_qty)` for each item
3. Update items.current_stock: add back `(quantity - returned_qty)` for each item
4. Reverse customer ledger: INSERT credit entry for `total_amount - returned_amount`
5. Update customers.current_balance: subtract `total_amount - returned_amount`
6. Reverse GL: INSERT journal entry that debits Revenue, credits AR
7. If invoice had payments: the payments remain (they are historical records), but the invoice status becomes 'Cancelled'

**Rationale:** Full reversal ensures the cancelled invoice leaves no orphaned effects. The payments are kept as historical records but the invoice balance is zeroed.

### D4: Line-item discounts included in amount

**Decision:** Factor `discount_type`/`discount_value` from each `InvoiceItemForm` into the stored `amount` column during creation.

**Rationale:** The columns already exist in the schema. They were designed for this purpose but never wired into the calculation.

## Risks / Trade-offs

- **Existing data inconsistency:** Invoices already in the database have `total_amount` values computed with the old formula. A one-time migration or recalculation endpoint may be needed. Mitigation: Add a `recalculate_invoice_totals` endpoint or script.

- **Stock reversal complexity on edit:** If items are changed (not just quantities), the delta approach needs to handle different item_ids. Mitigation: Process removals (old items not in new list) and additions (new items not in old list) separately.

- **Payment interaction with cancellation:** If an invoice is partially paid and then cancelled, the payments need to be handled. Mitigation: Keep payments as historical records. The customer's credit balance should reflect the payment. The invoice balance goes to 0 on cancellation.
