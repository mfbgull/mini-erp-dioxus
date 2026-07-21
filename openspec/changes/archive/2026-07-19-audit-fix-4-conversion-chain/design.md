## Context

The conversion chain has two broken links. Both `convert_quotation` and `convert_sales_order` create the target document header but never copy line items from the source.

## Goals / Non-Goals

**Goals:**
- Quotation → SO copies all quotation_items to sales_order_items
- SO → Invoice copies all sales_order_items to invoice_items
- SO → Invoice creates stock movements (OUT) for each item
- Total amounts are derived from copied line items (consistent with Change 2)

**Non-Goals:**
- Partial conversions (convert specific items only)
- Bidirectional conversion tracking (which SO came from which quotation)
- Price override during conversion

## Decisions

### D1: Copy line items directly

**Decision:** When converting, SELECT all source line items and INSERT them into the target table with the same item_id, quantity, unit_price, and amount.

**Rationale:** Direct copy preserves the exact pricing from the source document. The target document's total is derived from the copied items, not from the source's stored total (which may have been computed differently).

### D2: SO → Invoice also creates stock movements

**Decision:** When converting SO to invoice, create stock movements (OUT) for each item, matching the behavior of `create_invoice`. This ensures inventory is decremented when an SO becomes an invoice.

**Rationale:** An SO is a commitment; an invoice is a realization. Stock should only move when the invoice is created.

### D3: Both conversions wrapped in transactions

**Decision:** Both conversion handlers will use transaction wrapping (from Change 1) to ensure atomicity.

## Risks / Trade-offs

- **Source document items may reference deleted items:** If a quotation item references an item that was later soft-deleted (is_active=0), the conversion should still work but the item won't appear in active item lists. Mitigation: Copy the item_id as-is; the foreign key still references a valid (inactive) item.
