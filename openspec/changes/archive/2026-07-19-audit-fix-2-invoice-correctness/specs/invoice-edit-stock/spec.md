## ADDED Requirements

### Requirement: Invoice edit reverses and recreates stock movements
When an invoice is updated via `PUT /api/invoices/{id}`, the system SHALL compare old and new invoice items and adjust stock accordingly. For each item: if the quantity changed, a stock movement is created for the delta. Stock balances and `items.current_stock` are updated to reflect the net change.

#### Scenario: Quantity increased on edit
- **WHEN** an invoice item is changed from quantity=5 to quantity=10
- **THEN** a stock movement OUT of 5 units is created for that item
- **AND** stock_balances quantity decreases by 5
- **AND** items.current_stock decreases by 5

#### Scenario: Quantity decreased on edit
- **WHEN** an invoice item is changed from quantity=10 to quantity=3
- **THEN** a stock movement IN of 7 units is created for that item
- **AND** stock_balances quantity increases by 7
- **AND** items.current_stock increases by 7

#### Scenario: Item removed on edit
- **WHEN** an invoice item that existed before is not present in the updated item list
- **THEN** a stock movement IN of the full original quantity is created
- **AND** stock_balances and current_stock are restored

#### Scenario: New item added on edit
- **WHEN** a new item is added to the invoice during edit
- **THEN** a stock movement OUT of the new quantity is created
- **AND** stock_balances and current_stock are decreased

#### Scenario: No quantity change
- **WHEN** an invoice item's quantity and unit_price are unchanged during edit
- **THEN** no stock movement is created for that item
