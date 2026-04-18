# Sidebar UI Improvement Plan

## TL;DR
> Redesign the left sidebar to be static (no scrolling items), show clear active page indicator with icon + label, and have a modern visual style with accent highlighting.

## Context

**User Request**: Improve the left sidebar UI in youtui-rs terminal client.

**Current State**:
- Sidebar shows page titles as scrollable list items
- Uses generic item_card rendering
- Selected item gets left border accent (just fixed)

**Desired State**:
- Static titles (no scrolling - items fixed in position)
- Clear active page indicator (background highlight + accent color)
- Icons alongside text labels
- App title/branding at top
- Modern, clean visual style

## Work Objectives

### Core Objective
Redesign sidebar to be static, visually clear, and modern.

### Concrete Deliverables
- `src/ui/components.rs`: Add `render_sidebar()` function with static positioning
- `src/ui/app.rs`: Replace loop-based sidebar with new function

### Must Have
- [ ] Static positioning (items don't scroll)
- [ ] Active item has background highlight + accent border
- [ ] Icons displayed alongside labels
- [ ] App title shown at top of sidebar

### Must NOT Have
- [ ] No scrolling behavior in sidebar
- [ ] No full-border boxes around items

## Implementation Tasks

### TODO 1: Add Sidebar Render Function
**What to do**:
- Add `SidebarItem` struct with `icon` and `label` fields
- Create `render_sidebar()` function in components.rs
- Function takes: Frame, area, items slice, selected_index, theme
- Each item is 3 rows tall, fixed positions starting at row 2
- Active item: background highlight color + left accent border
- Inactive items: secondary text color, no background

**Acceptance Criteria**:
- [x] `cargo check` passes
- [x] Sidebar items display with icons and labels
- [x] Active item clearly distinguishable

### TODO 2: Update App to Use New Sidebar
**What to do**:
- Define sidebar items with icons: 🔍 Search, 📜 History, 🔖 Saved, 📋 Playlists, ⬇️ Downloads, ⚙️ Settings
- Call `render_sidebar()` instead of manual loop in `render_main_view()`
- Remove old sidebar_items iteration code

**Acceptance Criteria**:
- [x] `cargo check` passes
- [x] All 6 sidebar items visible with icons
- [x] Arrow keys navigate between items (already working)

### TODO 3: Verify Visual Output
**What to do**:
- Run the app and verify sidebar renders correctly
- Check that active item has visible highlight

## Verification Strategy

### QA Scenarios
```
Scenario: Sidebar displays with icons and labels
  Tool: Bash
  Steps:
    1. cargo run
    2. Observe terminal output
  Expected Result: Sidebar shows "Youtui" title, then 6 items with icons
  Evidence: .sisyphus/evidence/sidebar-icons.txt

Scenario: Active item has visual highlight
  Tool: Bash
  Preconditions: App running
  Steps:
    1. Use arrow keys to move between sidebar items
  Expected Result: Each item changes when selected, active has highlight
  Evidence: .sisyphus/evidence/sidebar-active.png (if screenshot capability)
```

## Success Criteria

```bash
cargo check  # No errors
cargo build  # Compiles successfully
# App shows sidebar with:
# - "Youtui" title at top
# - 6 items with icons and labels
# - Active item has background highlight
```
