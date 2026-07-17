## Why

The current sidebar takes up significant horizontal space (240px) even when not in use. On smaller screens or when users want more content area, a collapsible sidebar provides better UX. Users should be able to collapse the sidebar to icon-only mode and access submenus via hover dropdowns.

## What Changes

- Sidebar becomes collapsible with a toggle button
- Collapsed state shows only icons (no text labels)
- Top-level menu items show dropdown on hover when collapsed
- Sidebar state persists across page refreshes (localStorage)
- Smooth animation for expand/collapse transitions

## Capabilities

### New Capabilities

- `collapsible-sidebar`: Sidebar collapse/expand functionality with hover dropdowns in collapsed state

### Modified Capabilities

(none)

## Impact

- `src/components/layout/sidebar.rs` — main sidebar component
- `src/components/layout/` — may need new dropdown component
- CSS transitions for smooth expand/collapse animation
