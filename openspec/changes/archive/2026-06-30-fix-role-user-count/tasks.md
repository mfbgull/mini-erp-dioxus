## 1. Backend Model

- [x] 1.1 Add `user_count: i64` field with `#[serde(default)]` to `Role` struct in `src/models.rs`

## 2. Backend API

- [x] 2.1 Update `list_roles` SQL in `src/server/admin_routes.rs` to include `user_count` subquery with dual match (role_id OR role name fallback)
- [x] 2.2 Update `get_role` SQL in `src/server/admin_routes.rs` to include `user_count` subquery
- [x] 2.3 Add `GET /api/roles/{id}/users` endpoint handler in `src/server/admin_routes.rs`
- [x] 2.4 Register the new route in the admin router

## 3. Frontend API Client

- [x] 3.1 Add `list_role_users(role_id)` method to `ApiClient` in `src/api.rs`

## 4. Frontend Pages

- [x] 4.1 Update `role_list.rs` to use `r.user_count` from API response instead of hardcoded `0`
- [x] 4.2 Update `role_detail.rs` to fetch users from `list_role_users()` API instead of hardcoded static data
- [x] 4.3 Update `role_detail.rs` users table to display `UserProfile` fields (username, full name, email)

## 5. Verify

- [x] 5.1 Run `cargo check` to confirm no compile errors
