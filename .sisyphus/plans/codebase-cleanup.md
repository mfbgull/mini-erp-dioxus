# Codebase Cleanup Plan

**Generated**: After `cargo fix --lib -p mini-erp` + `cargo fmt`
**Current state**: 300 lib warnings + 2 bin warnings. `cargo fmt` clean.

---

## Tier 1: Warning Cleanup (Mechanical)

### 1.1 Fix `variable does not need to be mutable` (239 instances)

**Files to touch** (15 files account for ~200 of 239):

| File | Count | Difficulty |
|---|---|---|
| `src/pages/production_create.rs` | 28 | Easy |
| `src/pages/employee_create.rs` | 28 | Easy |
| `src/pages/expense_create.rs` | 26 | Easy |
| `src/pages/bom_create.rs` | 18 | Easy |
| `src/pages/physical_count_detail.rs` | 15 | Easy |
| `src/pages/user_create.rs` | 14 | Easy |
| `src/components/data_grid/data_grid.rs` | 14 | Medium (big file) |
| `src/pages/user_edit.rs` | 12 | Easy |
| `src/pages/direct_purchase_detail.rs` | 11 | Easy |
| `src/pages/customer_create.rs` | 11 | Easy |
| Others (8-5 each) | ~20 | Easy |

**How**: `let mut x` → `let x` where compiler says it's never mutated.
**⚠️ Risk**: About 10-15% of these will be false positives (mutated through Dioxus signal patterns the compiler doesn't track). Revert those with `#[allow(unused_mut)]` or keep `mut`.

### 1.2 Unused variables (~25 instances)

| Category | Count | Fix |
|---|---|---|
| `e`, `d`, `c_name`, `n`, `nav`, `loc`, `lines`, `tax_str`, `ref_str` | 14 | Prefix with `_` |
| `navigator` | 2 | Dead code, remove or prefix |
| `toast`, `today`, `sel_mode`, `pinned_cls`, `cell_width`, `left_pinned_keys`, `right_pinned_keys` | 7 | Prefix with `_` |
| `filled_count` | 2 | Prefix with `_` |
| `errors` (in expense_create, user_create) | 2 | Prefix with `_` |
| `layout_clone` | 1 | Remove binding |
| `show_reset_modal` | 1 | Prefix with `_` |
| `total_system`, `total_debit`, `total_credit`, `total_content_height` | 4 | Dead code or prefix |
| `item_map` | 1 | Prefix with `_` |
| `on_warehouse_change` | 1 | Prefix with `_` |
| `customer_name`, `form`, `empty_msg`, `key`, `cb` | 5 | Prefix with `_` |

### 1.3 Unused imports (~6 instances)

| Import | File |
|---|---|
| `delete` | `src/...` |
| `AllowHeaders` | `src/...` |
| `crate::models` (×2) | Two files |
| `BadgeColor`, `TextAlign` | `src/components/data_grid/...` |
| `StockBalance` | `src/...` |

### 1.4 Deprecated API

| File | Change |
|---|---|
| Wherever used | `Decimal::is_positive()` → `is_sign_positive()` |

### 1.5 `App` function naming

`src/main.rs:340`: `fn App()` → `#[allow(non_snake_case)]` or rename.

---

## Tier 2: Dead Code Removal (Moderate)

### 2.1 Calculate dead code in `src/calculations/`

| Symbol | File | Notes |
|---|---|---|
| `DiscountScope` | `calculations/mod.rs` | Unused enum |
| `DiscountType` | `calculations/mod.rs` | Unused enum |
| `InvoiceMetrics` | `calculations/mod.rs` | Unused struct |
| `CustomerMetrics` | `calculations/mod.rs` | Unused struct |
| `InvoiceSummary` | `calculations/mod.rs` | Only used in doc/tests |
| `LedgerEntry` | `calculations/mod.rs` | Unused struct |
| `CustomerProfile` | `calculations/mod.rs` | Unused struct |
| `HasFilled` + `impl` | `calculations/quotation.rs` | Trait + impl, only used internally |

**Action**: Search `use` references across codebase. Delete or move to test modules.

### 2.2 Unused data_grid types

Many `ChartData`, `PieData`, `GaugeData`, `Breadcrumb*`, `FabAction`, `CompactCard*`, `ShortcutItem`, `SelectOption`, `LoaderSize` type definitions are referenced at definition site or only in tests.

**Action**: Each file in `src/components/common/` - grep for usage. Remove types with zero import references.

### 2.3 Unused macro

`protected_page!` in `src/main.rs:25` - defined but never invoked.

---

## Tier 3: Architecture Improvements (Significant)

### 3.1 Break up `api.rs` (3,531 lines)

Current state: monolithic `impl ApiClient` block with all endpoints.
**Plan**: Split by domain → `src/api/mod.rs` + `src/api/invoice.rs` + `src/api/inventory.rs` + `src/api/customer.rs` + `src/api/purchase.rs` + `src/api/report.rs` + `src/api/manufacturing.rs` + `src/api/accounting.rs`.

**Value**: Each file ~400-500 lines. Parallel compilation. Easier to find endpoints.

### 3.2 Break up `src/server/db.rs` (1,803 lines)

Split `seed_data` (711 lines) → `src/server/seed_data.rs`.
Split `get_db`, migrations → keep in `db.rs`.
**Value**: seed_data is self-contained and huge.

### 3.3 Break up `src/pages/invoice_create.rs` (1,208 lines)

The `InvoiceCreatePage` function (856 lines) is a single render function.
**Plan**: Extract sub-components: `InvoiceItemTable`, `InvoiceCustomerSelector`, `InvoiceSummaryPanel`, `InvoiceCreateForm`. Extract state into custom hooks.

**Same pattern** applies to: `ItemCreatePage` (781 lines), `CustomerDetailPage` (880 lines), `ProductionCreatePage` (672 lines).

### 3.4 Break up `src/components/data_grid/data_grid.rs` (1,291 lines, 1,221-line function)

The `DataGrid` function is the largest single function in the codebase.
**Plan**: Extract row rendering, header rendering, cell rendering, filter bar, sort indicators into separate sub-components. Extract state management into custom hooks.

### 3.5 Seeds data extraction

`seed_data` at 711 lines in `db.rs`. Extract to `src/server/seed_data.rs` or `src/seed/`.

### 3.6 Test infrastructure

Create `tests/` directory. Move inline test helpers. Add integration tests for page flows.

---

## Estimated Effort

| Tier | Items | Effort | Risk |
|---|---|---|---|
| 1 | ~200 line changes across 20 files | 2-3 hours | Low (mechanical) |
| 2 | ~30-50 type/function deletions | 2-3 hours | Low (dead code) |
| 3 | 6 refactors (api.rs, db.rs, DataGrid, invoice_create, item_create, customer_detail) | 10-15 hours | Medium (touch many callers) |

**Total**: ~15-20 hours for full cleanup. Tier 1 alone gets from 300 warnings to <10.

---

## Suggested Order

1. **Tier 1** first (quick wins, big warning count reduction)
2. **Tier 2** next (dead code removal makes Tiers 3 cleaner)
3. **Tier 3** if/when time allows (takes most effort)

## Acceptance Criteria

- `cargo build` produces < 10 warnings (or zero)
- `cargo fmt --check` passes (already done)
- `cargo test` passes (all existing tests)
- No functional changes in any cleanup
