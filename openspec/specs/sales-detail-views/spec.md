# sales-detail-views

Read-only detail views for quotations and sales orders — displaying header
fields, computed KPIs, line items, and status-appropriate actions bound to
existing sales APIs.

## Requirements

### Requirement: Quotation detail view renders loaded record

The Quotation Detail page SHALL render the fetched quotation's header,
status, KPI summary, and line items once loaded, instead of a placeholder.

#### Scenario: Quotation loads successfully
- **WHEN** a user opens `/sales/quotations/:id` for an existing quotation
- **THEN** the page displays the quotation number, customer name, quotation
  date, valid-until date, and a status badge
- **AND** it displays a line-items table with each item's code, name, quantity,
  unit price, and net amount
- **AND** it displays the total amount

#### Scenario: Quotation still loading
- **WHEN** the fetch has not yet resolved
- **THEN** the page shows a loading spinner

#### Scenario: Quotation not found
- **WHEN** the requested quotation id does not resolve to a record
- **THEN** the page shows a "Quotation Not Found" empty state

### Requirement: Sales order detail view renders loaded record

The Sales Order Detail page SHALL render the fetched sales order's header,
status, KPI summary, and line items once loaded, instead of a placeholder.

#### Scenario: Sales order loads successfully
- **WHEN** a user opens `/sales/sales-orders/:id` for an existing order
- **THEN** the page displays the order number, customer name, order date,
  delivery date, and a status badge
- **AND** it displays a line-items table with each item's code, name, quantity,
  unit price, and net amount
- **AND** it displays the total amount

#### Scenario: Sales order not found
- **WHEN** the requested sales order id does not resolve to a record
- **THEN** the page shows a "Sales Order Not Found" empty state

### Requirement: Detail actions bind only to existing sales APIs

The detail pages SHALL expose actions that call existing API methods and
provide user feedback via toast; no new backend endpoints are introduced.

#### Scenario: Convert quotation to invoice
- **WHEN** a user clicks "Convert to Invoice" on a quotation detail page
- **THEN** the page calls `convert_quotation` and shows a success or error toast
  based on the result

#### Scenario: Convert sales order to invoice
- **WHEN** a user clicks "Convert to Invoice" on a sales order detail page
- **THEN** the page calls `convert_sales_order` and shows a success or error
  toast based on the result

#### Scenario: Cancel sales order
- **WHEN** a user clicks "Cancel" on a sales order detail page
- **THEN** the page calls `cancel_sales_order` and shows a success or error toast
  based on the result

### Requirement: KPI fields degrade honestly when data is absent

Where the server response provides only a total (no subtotal/discount/tax
breakdown), the page SHALL render the total and MUST NOT fabricate breakdown
values.

#### Scenario: Server returns only total_amount
- **WHEN** the sales order or quotation response has no discount or tax
  breakdown
- **THEN** the total is displayed from `total_amount` and breakdown KPIs are
  either omitted or shown as zero, not invented
