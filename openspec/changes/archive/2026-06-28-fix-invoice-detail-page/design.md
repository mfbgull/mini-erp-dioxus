## Context

The Dioxus 0.7 router (`dioxus-router@0.7.9`) uses `:param` syntax for dynamic route segments (e.g., `/users/:id`), but the existing codebase uses `{param}` curly-brace syntax (e.g., `/users/{id}`) which was from an earlier Dioxus version or a different router convention. This causes all parameterized routes to produce 404 — the router doesn't match them.

## Goals / Non-Goals

**Goals:**
- Fix all 20 parameterized route attributes so they match incoming URLs
- Make the invoice detail page (and all detail/list pages) navigable

**Non-Goals:**
- No component logic changes
- No API endpoint changes (these are Axum, unaffected)
- No router version change or architectural refactor

## Decisions

- **Simple find-and-replace**: `{id}` → `:id` and `{item_id}` → `:item_id` in `#[route(...)]` strings. The `{param}` syntax has no valid meaning in Dioxus 0.7 routes (it's treated as literal text), so the replacement is the entire fix.
- **No need to touch catch-all**: `#[route("/:..route")]` already uses the correct `:..segment` syntax, so it works.
- **No type changes needed**: Dioxus 0.7 reads `:id` as a `String` parameter by default, which matches the existing `id: String` in component signatures.

## Risks / Trade-offs

None. Pure syntax fix, no behavioral change.
