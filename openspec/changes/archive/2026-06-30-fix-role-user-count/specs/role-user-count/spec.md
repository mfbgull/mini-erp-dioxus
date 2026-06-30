## ADDED Requirements

### Requirement: Roles API returns user count
The `GET /api/roles` and `GET /api/roles/{id}` responses SHALL include a `user_count` integer field representing the number of users assigned to each role.

#### Scenario: List roles with user counts
- **WHEN** a client requests `GET /api/roles`
- **THEN** each role object in the response contains a `user_count` field with the correct number of users assigned to that role

#### Scenario: Get single role with user count
- **WHEN** a client requests `GET /api/roles/{id}`
- **THEN** the role object contains a `user_count` field with the correct number of users assigned

### Requirement: Role user count matches by role_id or role name
The user count SHALL match users where `role_id` equals the role's ID, OR where `role_id` is NULL and the user's `role` text matches the role's `role_name`.

#### Scenario: User with role_id set
- **WHEN** a user has `role_id` set to 3
- **THEN** they are counted under the role with ID 3

#### Scenario: User without role_id
- **WHEN** a user has `role_id` as NULL and `role` = "admin"
- **THEN** they are counted under the role with `role_name` = "admin"

### Requirement: Role users endpoint
The system SHALL provide a `GET /api/roles/{id}/users` endpoint returning the list of users assigned to that role.

#### Scenario: Fetch users for a role
- **WHEN** a client requests `GET /api/roles/{id}/users`
- **THEN** the response contains a `data` array of `UserProfile` objects for users assigned to that role

#### Scenario: Role with no users
- **WHEN** a client requests `GET /api/roles/{id}/users` for a role with no assigned users
- **THEN** the response contains an empty `data` array

### Requirement: Frontend roles list shows user count
The Roles & Permissions page SHALL display the actual user count from the API in the Users column.

#### Scenario: Roles list displays counts
- **WHEN** the Roles & Permissions page loads
- **THEN** the Users column shows the `user_count` value from the API response for each role

### Requirement: Frontend role detail shows assigned users
The Role Detail page SHALL fetch and display the actual list of users assigned to the role from the API.

#### Scenario: Role detail shows user table
- **WHEN** a user navigates to a role detail page
- **THEN** the "Users with this Role" section shows a table of users fetched from `GET /api/roles/{id}/users` with username, full name, and email columns
