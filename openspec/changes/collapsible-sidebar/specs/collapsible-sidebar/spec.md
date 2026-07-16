## ADDED Requirements

### Requirement: Sidebar collapse toggle
The sidebar SHALL have a toggle button that collapses/expands the sidebar. The toggle button SHALL be visible in both expanded and collapsed states.

#### Scenario: Toggle sidebar collapse
- **WHEN** user clicks the collapse toggle button
- **THEN** sidebar transitions from expanded (240px) to collapsed (64px) with animation

#### Scenario: Toggle sidebar expand
- **WHEN** sidebar is collapsed and user clicks the toggle button
- **THEN** sidebar transitions from collapsed (64px) to expanded (240px) with animation

### Requirement: Collapsed sidebar shows icons only
When collapsed, the sidebar SHALL display only icons for navigation items, hiding text labels. Icons SHALL have tooltips on hover showing the label text.

#### Scenario: Collapsed state displays icons
- **WHEN** sidebar is collapsed
- **THEN** each nav item shows only its icon, no text label

#### Scenario: Tooltip on hover
- **WHEN** user hovers over a nav item icon in collapsed state
- **THEN** a tooltip appears showing the item label

### Requirement: Hover dropdown for collapsed menus
When collapsed, hovering over a nav module header SHALL display a dropdown with its child nav items positioned to the right of the sidebar.

#### Scenario: Hover dropdown appears
- **WHEN** sidebar is collapsed and user hovers over a nav module header
- **THEN** a dropdown appears to the right of the sidebar showing the module's nav items with icons and labels

#### Scenario: Hover dropdown disappears
- **WHEN** user moves mouse away from the dropdown and the module header
- **THEN** the dropdown disappears

#### Scenario: Click item in dropdown
- **WHEN** user clicks a nav item in the hover dropdown
- **THEN** the application navigates to that route and the dropdown closes

### Requirement: Collapse state persistence
The sidebar collapse state SHALL be persisted in localStorage and restored on page load.

#### Scenario: State persists across refresh
- **WHEN** user collapses the sidebar and refreshes the page
- **THEN** sidebar loads in collapsed state

#### Scenario: Default state is expanded
- **WHEN** no localStorage value exists
- **THEN** sidebar loads in expanded state

### Requirement: Smooth animation
The sidebar expand/collapse transition SHALL use CSS animation for smooth visual feedback.

#### Scenario: Animated transition
- **WHEN** sidebar state changes (expand or collapse)
- **THEN** the width transition animates smoothly over approximately 200ms
