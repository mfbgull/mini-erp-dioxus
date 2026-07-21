## 1. API client

- [x] 1.1 Add `delete_production(&self, id: i64) -> Result<serde_json::Value, String>`
  in `src/api.rs`, calling `DELETE /api/production/productions/{id}` following
  the existing `create_production`/`update_production` pattern.

## 2. Create page (rebuild around reference flow)

- [x] 2.1 Load items (`list_items_catalog`), warehouses (`list_warehouses`), and
  BOMs (`list_boms`) on mount. Filter finished-good options to items where
  `is_finished_good`.
- [x] 2.2 When a finished good is selected, find its active BOM (filter
  `list_boms` by `finished_item_id` + `is_active`, or call `get_bom_by_item`).
  If none, show a "create a BOM first" message and block save.
- [x] 2.3 Fetch the selected BOM's items (`get_bom` → `items`). When the BOM or
  output quantity changes, compute input materials as `component_qty × output_qty`.
- [x] 2.4 Render the calculated materials list with name, computed quantity, UOM,
  and available stock (from the item's `current_stock`); mark each as sufficient
  or insufficient.
- [x] 2.5 Add two `SearchableSelect`s: raw-materials warehouse and finished-goods
  warehouse (both required).
- [x] 2.6 Add overhead cost input and a live cost preview box (material cost from
  input `standard_cost × qty`, overhead, total, cost per unit).
- [x] 2.7 On save: validate finished good + qty > 0 + both warehouses + at least
  one input; block if any input exceeds stock. Build `ProductionForm` with real
  `warehouse_id` (finished-goods), `bom_id`, `overhead_cost`, `notes`, and
  `inputs` (each `ProductionInputForm { item_id, quantity, warehouse_id: raw }`).
  Call `create_production`, toast the returned `production_no`, navigate to list.
- [x] 2.8 Keep the discard-confirm modal. Remove the Start/End date "Schedule"
  section (no backend columns).

## 3. Detail page (strip to read-only record)

- [x] 3.1 Replace `fetch_production_detail_from_api` mapping so it uses the real
  `get_production` fields: `output_item_name`, `output_item_code`,
  `warehouse_name`, `bom_name`, `overhead_cost`, `total_material_cost`,
  `notes`, `created_at`, and each input's `item_name`/`item_code`/`quantity`/
  `unit_cost`. Drop the hardcoded empty/zero placeholders.
- [x] 3.2 Replace the KPI/section layout: show Production Summary (number, date,
  output item, quantity produced, warehouse, overhead, notes), a Materials
  Consumed table (item, quantity + UOM, unit cost, line total, total material
  cost), and a Cost Summary (material, overhead, total, cost/unit).
- [x] 3.3 Remove Complete, Cancel, Update Progress, and Edit buttons and their
  modals/handlers; remove scrap/efficiency KPIs and the issued-qty/status
  material columns.
- [x] 3.4 Wire the Delete button to `delete_production(id)` in a spawned future:
  success → toast + navigate to `/manufacturing/production`; error → error toast,
  stay on page.

## 4. List + dashboard cleanup

- [x] 4.1 In `production_list.rs`, remove the `completed_qty` and `Progress %`
  columns (always zero). Keep production #, item, dates, status if backend
  provides it.
- [x] 4.2 In `manufacturing_dashboard.rs`, remove the always-zero "Completed"
  run column and the completed/yield KPIs that depend on lifecycle status; keep
  BOM/order counts and the quick-action/navigation buttons.

## 5. Verify

- [x] 5.1 `cargo check` compiles with no new errors and no newly-unused imports
  (remove now-dead `StatCard`/lifecycle imports on the detail page).
- [~] 5.2 (needs runtime/GUI) Record a production for a finished good with a BOM:
  confirm materials calculate, insufficient stock blocks save, a successful save
  consumes inputs and adds output stock, the detail shows real data, and delete
  removes the record.
