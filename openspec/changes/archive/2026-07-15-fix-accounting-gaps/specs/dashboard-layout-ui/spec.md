## ADDED Requirements

### Requirement: Dashboard layout management page
The system SHALL provide a UI page at `src/pages/dashboard_layouts.rs` that lists all saved dashboard layouts with columns: Name, Active, Created At. The page SHALL include buttons to create new layouts, edit existing ones, and delete layouts.

#### Scenario: View dashboard layouts list
- **WHEN** a user navigates to the dashboard layouts page
- **THEN** the page displays a list of all saved layouts with their names and active status

#### Scenario: Delete a dashboard layout
- **WHEN** a user clicks delete on a layout
- **THEN** the layout is removed from the list and the `DELETE /api/dashboard/layout/{id}` endpoint is called

### Requirement: Dashboard layout create/edit form
The system SHALL provide a form for creating and editing dashboard layouts. The form SHALL include fields for `layout_name` and `blocks` (JSON configuration). The form SHALL call `POST /api/dashboard/layout` for creation and `PUT /api/dashboard/layout/{id}` for updates.

#### Scenario: Create a new dashboard layout
- **WHEN** a user fills in the layout name and block configuration and submits
- **THEN** a new layout is created via the API and the user is redirected to the layouts list

#### Scenario: Edit an existing dashboard layout
- **WHEN** a user edits a layout's name or blocks and saves
- **THEN** the layout is updated via the API

### Requirement: Dashboard loads saved layout
The main `dashboard.rs` page SHALL check for a saved layout and apply it if one exists. If no layout is saved, the default layout SHALL be used.

#### Scenario: Dashboard applies saved layout
- **WHEN** a user with a saved layout navigates to the dashboard
- **THEN** the dashboard renders using the saved layout configuration

#### Scenario: Dashboard uses default when no layout saved
- **WHEN** a user with no saved layouts navigates to the dashboard
- **THEN** the dashboard renders using the default layout

### Requirement: Layout management route
The dashboard layouts page SHALL be accessible via a route in the navigation system.

#### Scenario: Navigate to layout management
- **WHEN** a user clicks "Dashboard Layouts" in the navigation
- **THEN** the dashboard layouts page is displayed
