## Why

The invoice detail page (and all parameterized routes) return 404 because Dioxus 0.7 uses `:param` syntax for dynamic segments, but the routes use `{param}` curly brace syntax which isn't recognized by the router.

## What Changes

- Change all `#[route(".../{param}")]` attributes in `src/main.rs` to use `#[route(".../:param")]` syntax
- This affects 20 route definitions across inventory, sales, purchasing, manufacturing, customers, suppliers, employees, users, and roles

## Capabilities

### New Capabilities

- `routing-params`: Correct route parameter syntax for Dioxus 0.7 router — all dynamic segments use `:param` instead of `{param}`

### Modified Capabilities

<!-- No existing specs are changing — this is a pure implementation fix -->

## Impact

- Single file change: `src/main.rs`
- 20 route attributes updated:
  - `{id}` → `:id` (19 occurrences)
  - `{item_id}` → `:item_id` (1 occurrence)
- No API, model, or component logic changes
- Fixes all detail pages, print pages, and parameterized routes across the app
