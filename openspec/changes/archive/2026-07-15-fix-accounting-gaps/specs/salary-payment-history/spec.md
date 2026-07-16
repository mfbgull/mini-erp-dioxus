## ADDED Requirements

### Requirement: List salary payments API
The system SHALL expose `GET /api/employees/{id}/salary-payments` that returns all salary payments for an employee, ordered by `payment_date` descending. The response SHALL include `id`, `employee_id`, `amount`, and `payment_date`.

#### Scenario: List salary payments for employee
- **WHEN** a user requests salary payments for employee ID 7
- **THEN** the system returns a JSON array of salary payment records for that employee

#### Scenario: Empty list for employee with no payments
- **WHEN** a user requests salary payments for an employee with no payment history
- **THEN** the system returns an empty array

### Requirement: Salary payment history UI tab
The `employee_detail.rs` page SHALL include a "Salary History" tab that displays past salary payments in a table with columns: Date, Amount. The tab SHALL show the total amount paid across all records.

#### Scenario: View salary history on employee detail
- **WHEN** a user views employee ID 7's detail page and clicks the "Salary History" tab
- **THEN** the page displays a table of salary payments with date and amount columns
- **AND** a summary showing total paid

### Requirement: Delete salary payment API
The system SHALL expose `DELETE /api/employees/{id}/salary-payments/{payment_id}` that removes a salary payment record. This endpoint SHALL be optional.

#### Scenario: Delete a salary payment
- **WHEN** a user deletes salary payment ID 15 for employee ID 7
- **THEN** the row is removed from `salary_payments` and the response confirms deletion
