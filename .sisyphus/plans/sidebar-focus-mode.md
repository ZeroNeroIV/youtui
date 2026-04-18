# Sidebar Focus Mode Redesign

## TL;DR
> Left sidebar always visible. Right arrow focuses content area (sidebar collapses to icons-only). Left arrow focuses sidebar (full view). Content area dims when sidebar is focused.

## Context

**User Request**: Redesign UI navigation so pages render in the right area, not full-screen.

**Current State**:
- Left sidebar with icons (implemented)
- Pages render in content_area on the right
- No focus states between sidebar and content

**Desired State**:
- **Left Sidebar** (always visible):
  - When focused: Full view with icons + labels + active highlight
  - When NOT focused (right content focused): Collapsed to icons-only with active item in distinct color
- **Right Content**:
  - When focused: Full brightness
  - When NOT focused: Dimmed (lower opacity or muted colors)
- **Navigation**:
  - Left Arrow → Focus sidebar
  - Right Arrow → Focus content
  - ActiveBlock enum determines which side has keyboard focus

## Work Objectives

### Core Objective
Implement sidebar focus mode with collapse/dim behavior.

### Concrete Deliverables
- Update `ActiveBlock` handling in `handle_events()`
- Add `is_focused` parameter to `render_sidebar()`
- Implement sidebar collapse mode (icons-only)
- Implement content dimming when sidebar is focused
- Handle Left/Right arrow key navigation

### Must Have
- [ ] Left arrow switches focus to sidebar
- [ ] Right arrow switches focus to content
- [ ] Sidebar collapses when content is focused
- [ ] Content dims when sidebar is focused
- [ ] Visual distinction between focused/unfocused states

### Must NOT Have
- [ ] No full-screen page takeover
- [ ] No breaking existing navigation (Up/Down, Enter, etc.)

## Implementation Tasks

### TODO 1: Update ActiveBlock Handling
**What to do**:
- Modify `handle_events()` in `AppMode::Main` to handle Left/Right arrow keys
- Left Arrow: Set `active_block = ActiveBlock::Sidebar`
- Right Arrow: Set `active_block = ActiveBlock::Content`

**Acceptance Criteria**:
- [x] `cargo check` passes
- [x] Left/Right arrows change focus state

### TODO 2: Add Focus Parameter to render_sidebar
**What to do**:
- Update `render_sidebar()` signature to include `is_focused: bool`
- When `is_focused = true`: Full view with label text visible
- When `is_focused = false`: Icons only, 1-row height per item

**Acceptance Criteria**:
- [x] `cargo check` passes
- [x] Sidebar renders differently based on focus state

### TODO 3: Update render_main_view for Dimming
**What to do**:
- Add dimming logic to content area when sidebar is focused
- Use lower opacity/muted colors on content widgets
- Ensure sidebar always renders fully when active

**Acceptance Criteria**:
- [x] Content visually dims when sidebar is focused
- [x] Sidebar remains fully visible when focused

### TODO 4: Handle Sidebar Item Selection
**What to do**:
- When sidebar is focused and Enter is pressed: Select item and switch mode
- Maintain existing keyboard shortcuts (s=Search, h=History, etc.)
- Update `handle_events()` for sidebar selection in `ActiveBlock::Sidebar`

**Acceptance Criteria**:
- [x] Enter key on sidebar item navigates to that page
- [x] Page renders in right content area

## Verification Strategy

### QA Scenarios
```
Scenario: Left/Right arrow navigation
  Tool: Bash (manual test)
  Steps:
    1. cargo run
    2. Press Right Arrow - verify content is focused, sidebar collapsed
    3. Press Left Arrow - verify sidebar expanded, content dimmed
  Expected Result: Focus alternates between sidebar and content

Scenario: Sidebar collapse behavior
  Tool: Bash (manual test)
  Steps:
    1. Focus content (Right Arrow)
    2. Observe sidebar shows only icons
    3. Focus sidebar (Left Arrow)
    4. Observe sidebar shows icons + labels
  Expected Result: Collapsible sidebar behavior works

Scenario: Content dimming
  Tool: Bash (manual test)
  Steps:
    1. Focus sidebar (Left Arrow)
    2. Observe content area is dimmed
    3. Focus content (Right Arrow)
    4. Observe content area returns to full brightness
  Expected Result: Dimming effect visible
```

## Success Criteria

```bash
cargo check  # No errors
cargo build  # Compiles successfully
# App shows:
# - Left sidebar always visible
# - Right arrow collapses sidebar to icons
# - Left arrow expands sidebar with full text
# - Content dims when sidebar is focused
# - Navigation works with arrow keys
```
