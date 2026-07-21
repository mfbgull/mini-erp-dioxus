## MODIFIED Requirements

### Requirement: Monetary calculations use Money type
All monetary calculations in server route handlers SHALL use the `Money` type from `src/money.rs` instead of raw `f64`. This includes: invoice total computation, payment amount handling, purchase total computation, production cost computation, and all report aggregations.

#### Scenario: Invoice total uses Decimal arithmetic
- **WHEN** an invoice is created with items totaling 100.00 + 17% tax
- **THEN** the calculation is: Money::from(100) * Money::from(0.17) = Money("17.00") exactly
- **AND** NOT f64 100.0 * 0.17 which could produce 16.999999999999996

#### Scenario: Customer balance uses Money type
- **WHEN** a customer has multiple invoices and payments
- **THEN** the balance is computed using Money addition/subtraction
- **AND** the result is exact to 2 decimal places

### Requirement: Money type serializes as f64 for API compatibility
The `Money` type SHALL continue to serialize as f64 in JSON responses to maintain backward compatibility with the frontend. The precision gain is in storage and calculation, not in JSON representation.

#### Scenario: API response format unchanged
- **WHEN** a client requests an invoice via GET /api/invoices/1
- **THEN** the `total_amount` field is a JSON number (f64), not a string
- **AND** the value is accurate to at least 2 decimal places
