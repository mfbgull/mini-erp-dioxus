## Context

Both `src/pages/quotation_detail.rs` and `src/pages/sales_order_detail.rs` are
~95% complete: PAGE_CSS, the detail structs (`QuotationDetail` /
`SalesOrderDetail` + line-item structs), the `use_resource` fetch, the loading
and not-found branches, and the status-class helpers all exist. Only the final
`rsx!` body renders a `"coming soon"` empty-state.

The fix is to replace that body with markup that consumes the already-bound
`q` / `so` struct, reusing the CSS classes already defined in each file.

## Goals / Non-Goals

**Goals**
- Render header, status badge, KPI/summary, info grid, line-items table, and an
  action bar in each page.
- Bind actions to existing API methods only.

**Non-Goals**
- No new backend endpoints or API-client methods.
- No new subtotal/discount/tax computation — server exposes only `total_amount`.
- No delete action (no delete endpoint exists for these resources). The unused
  `show_delete_modal` signal will be removed to silence warnings.

## Decisions

### Reuse existing CSS classes, no new components
Each file defines a complete `qdetail-*` / `sodetail-*` class set. The render
body will use those classes directly rather than pulling in `StatCard` etc., to
keep the diff to a single function body per file and match the existing design
language.

### Actions available (verified in src/api.rs)
| Page | Action | API method |
|------|--------|-----------|
| Quotation | Convert to Invoice | `convert_quotation(id)` |
| Quotation | Back | navigator → `/sales/quotations` |
| Sales Order | Convert to Invoice | `convert_sales_order(id)` |
| Sales Order | Cancel | `cancel_sales_order(id)` |
| Sales Order | Back | navigator → `/sales/sales-orders` |

Convert/Cancel call the API in a spawned future and surface a toast; on success
they navigate back to the list. No optimistic UI.

### Honest KPIs
`SalesOrderDetail.subtotal` is already set to `total_amount` with a `ponytail:`
comment. The quotation struct zeroes the breakdown fields. The KPI row will show
Total (real) and Status; breakdown rows are omitted rather than shown as
misleading zeros where they'd read as real figures.

## Risks / Trade-offs

- **Duplication between the two pages** — accepted. They have separate CSS
  namespaces and slightly different fields (quotation has discount/tax columns,
  SO does not). Extracting a shared component is more code than it saves for two
  callers. `ponytail: extract shared detail layout if a 3rd sales-detail page appears`.
- **`toast` / `navigator` currently unused** — they become used once the action
  bar is wired, removing existing dead-variable warnings.

## Migration Plan

None — purely additive rendering. No schema, API, or route changes.

## Open Questions

None blocking. If the team later wants a Print view or Edit action, that's a
follow-up change (matches the deferred stubs noted in `plan.md`).
