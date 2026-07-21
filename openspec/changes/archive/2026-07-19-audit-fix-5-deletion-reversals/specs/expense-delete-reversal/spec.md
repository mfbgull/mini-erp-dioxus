## ADDED Requirements

### Requirement: Expense deletion reverses journal entry
When an expense is deleted, the system SHALL find the associated journal_entry (reference_type='expense', reference_id=expense_id) and delete it along with all its journal_lines.

#### Scenario: Delete expense removes GL entry
- **WHEN** expense ID 5 with expense_no 'EXP-2026-0001' is deleted
- **THEN** the journal_entry where reference_type='expense' AND reference_id=5 is deleted
- **AND** all journal_lines for that journal_entry are deleted
- **AND** the trial balance remains balanced (the debit and credit cancel out)
