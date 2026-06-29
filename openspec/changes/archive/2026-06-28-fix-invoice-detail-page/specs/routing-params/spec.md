## ADDED Requirements

### Requirement: Dynamic route segments use `:param` syntax

All Dioxus frontend route definitions SHALL use Dioxus 0.7 `:param` syntax for dynamic segments instead of `{param}` curly-brace syntax.

#### Scenario: Invoice detail route matches

- **WHEN** user navigates to `/sales/invoices/1`
- **THEN** the `InvoiceDetailPage` component renders with `id = "1"`

#### Scenario: All parameterized routes work after fix

- **WHEN** user navigates to any URL matching a parameterized route (e.g. `/inventory/items/5`, `/customers/3`, `/sales/quotations/2/print`)
- **THEN** the corresponding page component renders correctly instead of showing a 404

### Requirement: Catch-all route remains unchanged

The catch-all route `/:..route` SHALL continue working as before.

#### Scenario: Unknown route falls through to 404

- **WHEN** user navigates to `/nonexistent/path`
- **THEN** the `NotFoundPage` component renders
