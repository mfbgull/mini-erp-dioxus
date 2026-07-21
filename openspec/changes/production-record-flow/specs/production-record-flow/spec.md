## ADDED Requirements

### Requirement: Record production consumes BOM materials and produces output

The Production Create page SHALL let a user record a production run that
consumes raw materials derived from the finished good's active BOM and produces
finished goods, sending real input materials and warehouse ids to the API.

#### Scenario: Finished good with an active BOM
- **WHEN** a user selects a finished good that has an active BOM
- **THEN** the page loads that BOM and, once a production quantity is entered,
  expands each BOM component into an input material at `component_qty × output_qty`
- **AND** displays each input material with its name, computed quantity, unit of
  measure, and available stock

#### Scenario: Finished good without a BOM
- **WHEN** a user selects a finished good that has no active BOM
- **THEN** the page shows a message directing the user to create a BOM first
- **AND** the form cannot be saved as a production run

#### Scenario: Two warehouses selected
- **WHEN** a user records a production run
- **THEN** the page requires a raw-materials warehouse (materials consumed from)
  and a finished-goods warehouse (output produced into)
- **AND** submits each input material with the raw-materials warehouse id and the
  output with the finished-goods warehouse id

#### Scenario: Save sends real inputs
- **WHEN** a user saves a valid production run
- **THEN** the request includes the calculated input materials (not an empty
  list) and the selected warehouse ids
- **AND** on success shows a toast with the new production number and returns to
  the production list

### Requirement: Stock is validated before recording production

The Production Create page SHALL block submission when any calculated input
material exceeds available stock in the raw-materials warehouse.

#### Scenario: Insufficient stock
- **WHEN** a calculated input material's quantity exceeds the available stock
- **THEN** the page marks that material as insufficient
- **AND** prevents saving and informs the user which materials are short

#### Scenario: Sufficient stock
- **WHEN** all calculated input materials are within available stock
- **THEN** the page allows the production run to be saved

### Requirement: Production create shows a live cost preview

The Production Create page SHALL show a cost breakdown that updates as inputs
and overhead change.

#### Scenario: Cost preview
- **WHEN** input materials are calculated and/or an overhead cost is entered
- **THEN** the page displays material cost, overhead cost, total cost, and cost
  per unit based on the output quantity

### Requirement: Production detail is a read-only record

The Production Detail page SHALL render a completed production record from the
data returned by `get_production`, without any work-order lifecycle controls.

#### Scenario: Record loads successfully
- **WHEN** a user opens `/manufacturing/production/:id` for an existing record
- **THEN** the page displays the production number, date, output item, quantity
  produced, warehouse, overhead cost, and notes when present
- **AND** displays a materials-consumed table with each input's item, quantity,
  unit cost, and line total, plus the total material cost
- **AND** displays a cost summary (material cost, overhead, total, cost per unit)

#### Scenario: No lifecycle controls
- **WHEN** the record is displayed
- **THEN** the page does not show Complete, Cancel, Update Progress, or Edit
  actions, nor scrap or efficiency figures

#### Scenario: Record not found
- **WHEN** the requested production id does not resolve to a record
- **THEN** the page shows a "Production Not Found" empty state

### Requirement: Production can be deleted from the detail page

The Production Detail page SHALL delete a production record via the existing
server route through a new `delete_production` API client method.

#### Scenario: Confirm delete
- **WHEN** a user confirms deletion of a production record
- **THEN** the page calls `delete_production(id)` and, on success, shows a toast
  and navigates back to the production list

#### Scenario: Delete fails
- **WHEN** the delete request fails
- **THEN** the page shows an error toast and remains on the detail page

### Requirement: List and dashboard show only backend-populated fields

The Production List and Manufacturing Dashboard SHALL NOT present columns or
KPIs that depend on non-existent lifecycle data (completed quantity, progress,
scrap, efficiency, yield).

#### Scenario: List columns
- **WHEN** the production list renders
- **THEN** it shows production number, item, and other fields the backend
  populates, and does not show an always-zero completed-quantity or progress
  column

#### Scenario: Dashboard KPIs
- **WHEN** the manufacturing dashboard renders
- **THEN** it shows BOM and production-order counts and navigation, and does not
  show completed-quantity or yield KPIs derived from lifecycle status
