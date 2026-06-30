## Context

The mini-erp project has a `roles` table and a `users` table. Users link to roles via `role_id` (nullable integer FK) and `role` (text name). The Roles & Permissions page shows a "Users" column but the backend API never returned user counts, and the frontend hardcoded `0`. The role detail page used hardcoded static data instead of querying the database.

## Goals / Non-Goals

**Goals:**
- Return accurate `user_count` in `list_roles` and `get_role` API responses
- Provide an endpoint to list users for a specific role
- Frontend pages display real data from the database

**Non-Goals:**
- Migrating users to always have `role_id` set (backwards-compatible fallback is sufficient)
- Changing the user-role assignment flow

## Decisions

**SQL subquery for user count**: Use a correlated subquery in `list_roles` and `get_role` rather than a JOIN, since we only need the count (not user details) in list views. This avoids row multiplication.

**Dual match condition**: `WHERE u.role_id = r.id OR (u.role_id IS NULL AND u.role = r.role_name)` — handles both legacy users (no `role_id`) and new users (with `role_id`). Alternative: migrate all users to have `role_id` set. Rejected because it's a data migration with rollback risk for no functional gain.

**New endpoint for role users**: `GET /api/roles/{id}/users` returns `UserProfile[]`. Kept separate from the count subquery so the detail page can fetch full user info without loading all users.

## Risks / Trade-offs

- **[Performance]** Subquery per role in list view is O(n) queries internally. Mitigated by SQLite's efficient correlated subquery execution and small role count (<20 typical).
- **[Data inconsistency]** Users with both `role_id` set and `role` name mismatch could double-count. Mitigated by `OR` logic — if `role_id` matches, it counts; only falls back to name when `role_id` is NULL.
