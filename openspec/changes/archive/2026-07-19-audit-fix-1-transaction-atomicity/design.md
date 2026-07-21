## Context

MiniERP uses `rusqlite` with a single global `Mutex<Connection>`. The Mutex provides serialization (only one SQL statement runs at a time across the app), but does NOT provide atomicity — each `db.execute()` call commits immediately. There are zero uses of `BEGIN`/`COMMIT`/`ROLLBACK` in the codebase. All errors from non-critical SQL operations are swallowed with `.ok()`.

## Goals / Non-Goals

**Goals:**
- Wrap every multi-step route handler in a SQLite transaction
- Ensure atomicity: all-or-nothing for each business operation
- Convert silent error swallowing to explicit error propagation within transactions
- Return proper error responses on failure instead of partial success

**Non-Goals:**
- Refactor into a service layer (out of scope for this change)
- Change the Mutex-based concurrency model
- Add savepoints (SQLite nested transactions)
- Modify the database schema

## Decisions

### D1: Transaction wrapping via execute_batch

**Decision:** Use `db.execute_batch("BEGIN IMMEDIATE")` / `db.execute_batch("COMMIT")` / `db.execute_batch("ROLLBACK")` for transaction control.

**Rationale:** `rusqlite`'s `Transaction` type requires borrowing the connection, which conflicts with the current pattern of calling `db.query_row()` and `db.execute()` interleaved throughout handler functions. `execute_batch` is compatible with the existing code structure and requires minimal refactoring.

**Alternatives considered:**
- `conn.transaction()` API: rejected — requires `&mut Connection` borrow that conflicts with interleaved query/execute calls
- Savepoints: rejected — not needed for current workflow complexity

### D2: Error propagation strategy

**Decision:** Inside transaction scope, replace `.ok()` with `map_err` that triggers rollback, then return `StatusCode::INTERNAL_SERVER_ERROR`. The rollback is explicit via a helper pattern.

**Pattern:**
```rust
db.execute_batch("BEGIN IMMEDIATE")?;
// ... all operations use ? instead of .ok()
// If any ? fails, the code below handles rollback
db.execute_batch("COMMIT")?;
// return success
```

For operations where the current `.ok()` is intentional (e.g., optional stock batch creation), keep `.ok()` but only within the transaction scope — the critical path operations use `?`.

### D3: Apply to all handlers, not just financial ones

**Decision:** Wrap ALL multi-step handlers, including non-financial ones (create_sales_order, create_quotation, etc.).

**Rationale:** Consistency. Every handler that performs multiple SQL statements should be atomic. The cost is minimal and the benefit is universal data integrity.

## Risks / Trade-offs

- **Lock contention:** `BEGIN IMMEDIATE` acquires a write lock. Under high concurrency, this could cause `SQLITE_BUSY` errors. Mitigation: the existing `Mutex<Connection>` already serializes all access, so there's no additional contention. The `busy_timeout=15000` pragma handles any edge cases.

- **Error message change:** Clients that previously received HTTP 200 with partial data will now receive HTTP 500 with an error message. This is a behavioral change but is strictly correct.

- **Performance:** Transactions add minimal overhead for SQLite. The Mutex serialization is the bottleneck, not transaction management.
