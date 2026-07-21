## Why

The Dioxus production pages were built as a work-order **lifecycle** UI
(Planned → In Progress → Completed, with scrap, efficiency, progress) on top of
a backend that only implements the reference app's **"record production"**
model. The result is a page full of fake and broken buttons:

- **Create** sends `inputs: vec![]` and hardcodes `warehouse_id: 1`, so a
  production run posts a finished-good stock IN but **never consumes any raw
  materials** — the opposite of the reference, where BOM-driven material
  calculation, stock validation, and consumption are the whole point.
- **Detail** hardcodes `item_name`/`item_code`/`bom_code`/`start_date`/
  `end_date` to empty and `completed_qty`/`scrap_qty` to `0`, throwing away the
  rich data `get_production` actually returns. Its Edit, Update Progress,
  Complete, and Cancel buttons only fire toasts, and **Delete never calls the
  API** — the order stays in the database.

The reference app (`/home/fawad/ai/minierp`) has **no** status, scrap,
efficiency, progress, or completion concept. Production is an immediate record
of a manufacturing event. This change realigns the Dioxus UI to that model.

## What Changes

- **Create page** (`production_create.rs`): rebuild around the reference flow —
  select finished good → auto-load its active BOM → enter quantity → expand BOM
  items × quantity into input materials → show per-material stock sufficiency →
  block save on insufficient stock → send real `inputs` and real warehouse ids.
  Add **raw-materials warehouse** and **finished-goods warehouse** selectors,
  overhead cost, and a live cost preview (material + overhead, cost/unit).
  Remove the unused Start/End date "Schedule" section (backend has no columns).
- **Detail page** (`production_detail.rs`): strip the work-order lifecycle
  entirely. Render a read-only **record** using real `get_production` data
  (output item, warehouses, BOM, materials with unit cost, total material cost,
  overhead, cost summary). Keep only **Delete** (wired to a real API) and Back.
  Remove Edit / Update Progress / Complete / Cancel buttons, the scrap and
  efficiency KPIs, and the empty issued-qty/status material columns.
- **List page** (`production_list.rs`): drop the always-zero `completed_qty`
  and `Progress %` columns; keep the honest columns the backend populates.
- **Dashboard** (`manufacturing_dashboard.rs`): drop the always-zero
  "Completed" run column and yield/completed KPIs that depend on non-existent
  status data; keep BOM/order counts and navigation.
- **API client** (`api.rs`): add the missing `delete_production(id)` method
  (the server route `DELETE /api/production/productions/{id}` already exists).

## Capabilities

### New Capabilities
- `production-record-flow`: A record-production workflow — create consumes
  BOM-derived raw materials from a chosen warehouse and produces finished goods
  into another, with stock validation and cost preview; detail is a read-only
  record with delete.

### Modified Capabilities
<!-- none: no existing spec's requirements change -->

## Impact

- Code: `src/pages/production_create.rs`, `src/pages/production_detail.rs`,
  `src/pages/production_list.rs`, `src/pages/manufacturing_dashboard.rs`,
  `src/api.rs` (add `delete_production`).
- APIs: consumes existing `list_items_catalog`, `list_warehouses`, `list_boms`,
  `get_bom`, `create_production`, `get_production`; adds client method
  `delete_production` for the existing server route. No new server endpoints.
- Backend `create_production` already consumes input stock and posts stock
  movements — no server change needed; the fix is sending real `inputs`.
- Non-goals: no work-order lifecycle (status/progress/scrap/efficiency), no
  production edit page, no start/end scheduling. Those diverge from the
  reference and are explicitly out of scope.
