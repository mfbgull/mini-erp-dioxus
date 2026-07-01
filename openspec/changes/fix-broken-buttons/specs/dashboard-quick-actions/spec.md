## ADDED Requirements

### Requirement: Dashboard action buttons navigate to their pages
The 4 quick-action buttons in the dashboard SHALL navigate to the corresponding page when clicked.

#### Scenario: New Invoice navigates to invoice create
- **GIVEN** the user is on the dashboard
- **WHEN** they click "New Invoice"
- **THEN** the app navigates to `/sales/invoices/new`

#### Scenario: New Item navigates to item create
- **GIVEN** the user is on the dashboard
- **WHEN** they click "New Item"
- **THEN** the app navigates to `/inventory/items/new`

#### Scenario: New Customer navigates to customer create
- **GIVEN** the user is on the dashboard
- **WHEN** they click "New Customer"
- **THEN** the app navigates to `/customers/new`

#### Scenario: View Reports navigates to reports dashboard
- **GIVEN** the user is on the dashboard
- **WHEN** they click "View Reports"
- **THEN** the app navigates to `/reports`

### Non-functional

- Navigation MUST use `use_navigator()` for client-side routing (no full page reload)
