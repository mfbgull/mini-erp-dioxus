## ADDED Requirements

### Requirement: Production deletion reverses output stock
When a production is deleted, the system SHALL remove the produced items from inventory by creating a stock movement OUT for the full output_quantity and updating stock_balances and items.current_stock.

#### Scenario: Delete completed production
- **WHEN** a production of 100 units of item A is deleted
- **THEN** a stock movement OUT of 100 units is created for item A
- **AND** stock_balances for item A decreases by 100
- **AND** items.current_stock for item A decreases by 100

### Requirement: Production deletion restores input stock
When a production is deleted, the system SHALL restore all consumed materials by creating stock movements IN for each production_input quantity and updating stock_balances and items.current_stock.

#### Scenario: Delete production with 3 inputs
- **WHEN** a production consumed 50 units of item B, 30 units of item C, and 20 units of item D
- **THEN** stock movements IN of 50, 30, and 20 are created for items B, C, D respectively
- **AND** stock_balances and current_stock increase by those quantities

### Requirement: Production deletion cleans up stock movements
When a production is deleted, all stock_movements with `reference_doctype='PRODUCTION'` and `reference_docno` matching the production_no SHALL be deleted.

#### Scenario: Production stock movements are removed
- **WHEN** a production with production_no 'PRD-2026-0001' is deleted
- **THEN** all stock_movements where reference_docno='PRD-2026-0001' are deleted
