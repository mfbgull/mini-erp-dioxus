## Why

The `money.rs` module provides a `Money` type based on `rust_decimal::Decimal` with exact arithmetic, comprehensive operators, serde support, and rusqlite integration. **It is completely unused.** Every monetary calculation in the application uses raw `f64`, which is subject to IEEE 754 floating-point rounding errors. These errors compound across calculations (invoice totals, COGS, P&L, inventory valuation, customer balances).

This is audit finding **F1** — the most architecturally significant issue in the audit.

Additionally, even the `Money` type itself has a flaw: its `ToSql`/`FromSql` implementations convert to/from `f64` at the storage boundary, defeating Decimal precision.

## What Changes

1. **Fix Money storage:** Change `ToSql`/`FromSql` to store as TEXT (Decimal string representation) instead of REAL. This preserves full precision in SQLite.
2. **Convert model fields:** Change all monetary fields in `models.rs` from `f64` to `Money`.
3. **Convert route handlers:** Replace `f64` arithmetic with `Money` arithmetic in all server route files.
4. **Convert SQL parameters:** Use `Money`'s `ToSql` for binding parameters instead of `f64`.
5. **Database migration:** Add a migration to convert existing REAL columns to TEXT (or handle both during transition).

## Capabilities

### New Capabilities

- `money-type-adoption`: All monetary calculations use the `Money` type with exact Decimal arithmetic.
- `decimal-storage`: Monetary values are stored as TEXT in SQLite, preserving full precision.

## Impact

- **Server routes**: All 8 `*_routes.rs` files — change f64 arithmetic to Money arithmetic
- **Models**: `src/models.rs` — change ~30 monetary fields from `f64` to `Money`
- **UI pages**: Multiple pages — change `format!` and display logic to use Money formatting
- **API client**: `src/api.rs` — update deserialization for Money fields
- **Database**: New migration to convert monetary REAL columns to TEXT
- **Breaking**: All API responses will serialize Money as a string (e.g., "1234.56") instead of a float. Frontend must handle string-to-number conversion. Alternatively, serialize as f64 for backward compatibility.
- **Risk**: HIGH — touches nearly every file in the codebase

## Capabilities

### Modified Capabilities

- All existing monetary calculations gain Decimal precision.

## Scope Note

This is the highest-risk change because it touches every file. It should be implemented LAST, after all other fixes are stable. The other fixes (Changes 1-6) correct business logic bugs that exist regardless of the f64 vs Decimal choice. This change adds precision on top of correct logic.
