## ADDED Requirements

### Requirement: "New PO" buttons navigate to purchase order create
The "New PO" button in the supplier detail page's Purchase Orders section and the "New Purchase Order" button in the supplier actions bar SHALL navigate to the purchase order creation page.

#### Scenario: Clicking New PO navigates
- **GIVEN** the user is on a supplier detail page
- **WHEN** they click "New PO" or "New Purchase Order"
- **THEN** the app navigates to `/purchases/orders/new`

### Non-functional

- Navigation MUST use client-side routing (no full page reload)
- The supplier is not pre-filled (out of scope for this change)
