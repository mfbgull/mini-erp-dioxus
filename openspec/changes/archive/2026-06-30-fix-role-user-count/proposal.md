## Why

The Roles & Permissions page displays a "Users" column but always shows `0` for every role, even when users are assigned. This makes the page misleading and unusable for understanding role assignments at a glance.

## What Changes

- Backend `list_roles` and `get_role` SQL queries now include a `user_count` subquery counting users per role
- Backend `get_role` SQL query includes `user_count` in its response
- New `GET /api/roles/{id}/users` endpoint returns actual users assigned to a role
- Frontend `role_list.rs` reads `user_count` from the API response instead of hardcoding `0`
- Frontend `role_detail.rs` fetches real users from the API instead of using hardcoded static data
- SQL queries match users by `role_id` OR by `role` name (for users created before `role_id` was set)

## Capabilities

### New Capabilities
- `role-user-count`: Backend returns accurate user counts per role and provides an endpoint to list users for a given role

### Modified Capabilities

## Impact

- `src/models.rs` — `Role` struct gains `user_count` field
- `src/server/admin_routes.rs` — SQL queries updated, new endpoint added
- `src/api.rs` — New `list_role_users()` client method
- `src/pages/role_list.rs` — Uses real `user_count` from API
- `src/pages/role_detail.rs` — Fetches users from API, removes hardcoded data
