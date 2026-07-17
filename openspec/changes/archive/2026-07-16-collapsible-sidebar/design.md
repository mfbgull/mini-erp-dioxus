## Context

The MiniERP application uses a fixed sidebar layout with navigation modules (Inventory, Sales, Purchases, Manufacturing, etc.). Each module has a header and nested NavItems. The sidebar is currently always expanded at 240px width.

## Goals / Non-Goals

**Goals:**
- Collapsible sidebar with toggle button
- Icon-only mode when collapsed
- Hover dropdowns for submenus when collapsed
- Persist collapse state in localStorage
- Smooth CSS transitions

**Non-Goals:**
- Mobile responsive sidebar (hamburger menu) — separate feature
- Drag-to-resize sidebar
- Multiple sidebar positions (left/right)

## Decisions

### D1: Collapse Toggle Position
**Decision**: Place toggle button at the bottom of the sidebar header, next to the app title.
**Rationale**: Common pattern in ERP systems, easily discoverable.

### D2: Collapsed State Width
**Decision**: 64px when collapsed (enough for icons + padding).
**Rationale**: Standard icon-sidebar width in Material Design and similar frameworks.

### D3: Hover Dropdown Implementation
**Decision**: Use CSS hover on nav module headers to show absolute-positioned dropdown.
**Rationale**: No JavaScript state needed for hover behavior, performant.

### D4: State Persistence
**Decision**: Use localStorage to remember collapsed/expanded state.
**Rationale**: Simple, no backend needed, persists across sessions.

### D5: Animation
**Decision**: CSS transition on width property (200ms ease).
**Rationale**: Smooth visual feedback without complexity.

## Risks / Trade-offs

- [Hover dropdowns on touch devices] → May not work well on tablets; acceptable for desktop ERP
- [Icon recognition] → Users may not recognize icons; mitigate with tooltips on hover
