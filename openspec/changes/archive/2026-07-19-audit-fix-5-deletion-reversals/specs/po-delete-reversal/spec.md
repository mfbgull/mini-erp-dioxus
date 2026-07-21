## ADDED Requirements

### Requirement: PO deletion reverses supplier ledger
When a purchase order is deleted, the system SHALL INSERT a supplier ledger entry with `type='PO_CANCELLATION'`, `debit=0`, `credit=po_total_amount` to reverse the original PO creation entry.

#### Scenario: Delete PO reverses supplier balance
- **WHEN** a PO of 5000.00 for supplier ID 3 is deleted
- **THEN** a supplier ledger entry is created: supplier_id=3, type='PO_CANCELLATION', debit=0, credit=5000.00
- **AND** the supplier's outstanding balance decreases by 5000.00

### Requirement: PO deletion reverses journal entry
When a purchase order is deleted, the system SHALL find the associated journal_entry (reference_type='purchase_order', reference_id=po_id) and delete it along with all its journal_lines.

#### Scenario: Delete PO removes GL entry
- **WHEN** a PO with id=10 is deleted
- **THEN** the journal_entry where reference_type='purchase_order' AND reference_id=10 is deleted
- **AND** all journal_lines for that entry are deleted
