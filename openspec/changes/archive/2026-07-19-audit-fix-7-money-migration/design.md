## Context

MiniERP has a fully-implemented `Money` type in `src/money.rs` that wraps `rust_decimal::Decimal`. It supports arithmetic operators, serde, rusqlite integration, and formatting. However, it is completely unused — all business logic uses raw `f64`. The `Money` type's `ToSql`/`FromSql` also convert to/from f64 at the storage boundary, which defeats the purpose.

## Goals / Non-Goals

**Goals:**
- Store monetary values as TEXT in SQLite (full Decimal precision)
- Use `Money` type for all monetary fields in models
- Use `Money` arithmetic in all route handlers
- Maintain API backward compatibility (serialize as f64 in JSON, or migrate to string)

**Non-Goals:**
- Multi-currency support
- Currency conversion
- Changing the frontend to display Money objects differently
- Migrating existing data in-place (handle gracefully during transition)

## Decisions

### D1: Storage format — TEXT with Decimal string

**Decision:** Store monetary values as TEXT in SQLite using the Decimal string representation (e.g., "1234.56"). This preserves full precision without the f64 round-trip loss.

**Rationale:** TEXT storage in SQLite is flexible and the Decimal string is self-describing. The existing `ToSql` converts to f64, losing precision. Changing to TEXT preserves it.

**Alternatives considered:**
- INTEGER cents: rejected — requires multiplying all values by 100, invasive schema changes, and the application already uses decimal amounts
- Keep REAL with better f64 handling: rejected — fundamentally can't fix f64 precision

### D2: JSON serialization — keep f64 for backward compatibility

**Decision:** Continue serializing Money as f64 in JSON responses. This avoids breaking the frontend. The precision gain is in storage and calculation, not in display.

**Rationale:** The frontend uses Dioxus (Rust WASM), which parses JSON. Changing the JSON format would require frontend changes. f64 display precision is sufficient for UI display (2 decimal places).

**Alternatives considered:**
- Serialize as string: rejected — breaks frontend, significant effort
- Serialize as both f64 and string: rejected — over-engineered

### D3: Migration strategy — gradual, not big-bang

**Decision:** Do NOT create a single migration that converts all columns. Instead:
1. Fix `Money`'s `ToSql`/`FromSql` to handle both REAL (old) and TEXT (new) transparently
2. New data is written as TEXT
3. Old data is read from REAL and converted to Decimal
4. Optionally run a one-time data migration script later

**Rationale:** A big-bang migration risks data loss and requires downtime. The dual-read approach is safe and backward-compatible.

### D4: Model field types

**Decision:** Change monetary fields in `models.rs` from `f64` to `Money`. The `Money` type has `From<f64>` and `Into<f64>` for compatibility.

**Fields to change (partial list):**
- `Item`: `current_stock` (keep f64 — it's quantity, not money), `standard_cost`, `selling_price`
- `Invoice`: `total_amount`, `paid_amount`, `balance_amount`, `returned_amount`, `discount_value`
- `InvoiceItem`: `unit_price`, `amount`, `discount_value`
- `Payment`: `amount`
- `SalesOrder`: `total_amount`
- `SalesOrderItem`: `unit_price`, `amount`
- `PurchaseOrder`: `total_amount`
- `PurchaseOrderItem`: `unit_price`, `amount`
- `DirectPurchase`: `unit_cost`, `total_cost`
- `StockMovement`: `unit_cost`
- `BomItem`: `unit_cost`
- `Production`: `overhead_cost`, `unit_cost`, `total_material_cost`
- `Expense`: `amount`
- `SalaryPayment`: `amount`
- `Employee`: `salary`
- `Customer`: `credit_limit`, `credit_balance`, `current_balance`, `opening_balance`, `total_invoiced`, `total_paid`
- `AccountBalance`: `debit`, `credit`, `balance`
- `JournalLine`: `debit`, `credit`
- `DashboardSummary`: `total_revenue`, `total_expenses`, `outstanding_ar`, `outstanding_ap`, `stock_value`

### D5: Quantity fields stay as f64

**Decision:** Fields representing physical quantities (not money) stay as `f64`: `current_stock`, `reorder_level`, `quantity`, `returned_qty`, `delivered_quantity`, `received_quantity`, `output_quantity`, `completed_qty`, `consumed_quantity`, etc.

**Rationale:** Quantities are not monetary values. Decimal precision for quantities is less critical and would add unnecessary complexity.

## Risks / Trade-offs

- **Massive codebase change:** Touches nearly every file. Risk of introducing new bugs is high. Mitigation: Implement after all other fixes are stable and tested.

- **SQLite schema evolution:** Changing REAL to TEXT requires a migration. SQLite doesn't support ALTER COLUMN. Mitigation: Create new TEXT columns, copy data, drop old columns. Or use the dual-read approach (D3) and avoid schema changes entirely.

- **Performance:** TEXT comparisons are slower than REAL. Mitigation: Monetary columns are rarely used in WHERE clauses or ORDER BY. Index impact is minimal.

- **Float serialization round-trip:** Money::from(f64) -> to_f64() -> Money::from(f64) can lose precision. Mitigation: The frontend only displays 2 decimal places, so f64 display precision is sufficient. The precision gain is in storage and calculation.

- **Dependency on rust_decimal:** Already in Cargo.toml. No new dependency needed.
