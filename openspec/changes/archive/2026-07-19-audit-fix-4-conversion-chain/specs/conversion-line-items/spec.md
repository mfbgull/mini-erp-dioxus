## ADDED Requirements

### Requirement: Quotation to SO copies line items
When converting a quotation to a sales order via `POST /api/sales/quotations/{id}/convert`, the system SHALL copy all `quotation_items` to `sales_order_items` with the same item_id, quantity, unit_price, and amount. The SO's `total_amount` SHALL be derived from the sum of copied line items.

#### Scenario: Convert quotation with 3 items
- **WHEN** a quotation with 3 items (amounts 200, 350, 150) is converted to a SO
- **THEN** 3 sales_order_items are created with the same item_ids, quantities, unit_prices, and amounts
- **AND** the SO's total_amount = 700.00
- **AND** the SO has item_count = 3

#### Scenario: Convert quotation with tax
- **WHEN** a quotation item has quantity=10, unit_price=20.00, discount=0, tax=34.00, amount=234.00
- **THEN** the corresponding sales_order_item has quantity=10, unit_price=20.00, amount=234.00

### Requirement: SO to Invoice copies line items and creates stock movements
When converting a sales order to an invoice via `POST /api/sales/sales-orders/{id}/convert`, the system SHALL:
1. Copy all `sales_order_items` to `invoice_items` with the same item_id, quantity, unit_price, and amount
2. Apply per-item tax_rate if available (default 0)
3. Compute the invoice total from the sum of line items
4. Create stock movements (OUT) for each item
5. Update stock_balances and items.current_stock for each item

#### Scenario: Convert SO with 2 items to invoice
- **WHEN** a SO with 2 items (qty 5 each, unit_price 20.00) is converted
- **THEN** 2 invoice_items are created
- **AND** 2 stock movements OUT of 5 units each are created
- **AND** stock_balances decrease by 5 for each item
- **AND** items.current_stock decrease by 5 for each item
- **AND** the invoice total_amount = 200.00 (2 × 5 × 20.00)

#### Scenario: SO conversion marks SO as Converted
- **WHEN** a SO is successfully converted to an invoice
- **THEN** the SO status is set to 'Converted'
- **AND** the SO's updated_at is set to current datetime
