## 1. Sidebar State Management

- [x] 1.1 Add `is_collapsed` signal to sidebar component
- [x] 1.2 Load initial state from localStorage on mount
- [x] 1.3 Save state to localStorage on change

## 2. Sidebar Toggle UI

- [x] 2.1 Add toggle button to sidebar header
- [x] 2.2 Style toggle button with appropriate icon (chevron left/right)
- [x] 2.3 Wire toggle button to `is_collapsed` signal

## 3. Collapsed State Styling

- [x] 3.1 Update sidebar width based on collapsed state (240px → 64px)
- [x] 3.2 Hide text labels when collapsed
- [x] 3.3 Show only icons when collapsed
- [x] 3.4 Add tooltips to icons in collapsed state

## 4. Hover Dropdown

- [x] 4.1 Create dropdown component for collapsed sidebar
- [x] 4.2 Position dropdown to the right of sidebar
- [x] 4.3 Show dropdown on module header hover when collapsed
- [x] 4.4 Hide dropdown on mouse leave
- [x] 4.5 Wire dropdown item clicks to navigation

## 5. Animation

- [x] 5.1 Add CSS transition for sidebar width
- [x] 5.2 Add CSS transition for content area margin
- [x] 5.3 Test smooth expand/collapse animation

## 6. Testing

- [x] 6.1 Test collapse/expand toggle
- [x] 6.2 Test hover dropdowns work correctly
- [x] 6.3 Test state persistence across page refresh
- [x] 6.4 Test navigation from dropdown items
