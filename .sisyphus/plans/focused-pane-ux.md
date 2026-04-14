# Focused Pane UX Enhancement Plan

## TL;DR
> **Quick Summary**: Add visual feedback for focused panes - colored borders and highlight symbols.
> 
> **Deliverables**:
> - Active pane has accent-colored border (vs gray default)
> - Focused items show ">> " prefix, unfocused show "> "
> - Works for Sidebar, Content, and Highlighted panels
> 
> **Estimated Effort**: Short
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request
User requested: "change the color of the borders of the focused pane/panel/section + when focused on something use >> and when not use >"

### Technical Context
- ActiveBlock enum has: Sidebar, Content, Highlighted
- Theme struct has: accent, border colors
- render() function creates List widgets with Block::border_style

---

## Work Objectives

### Core Objective
Add visual feedback to show which pane is currently focused.

### Concrete Deliverables
- Border color changes to accent color when pane is focused
- Border stays gray when pane is not focused
- ">> " prefix for focused pane items
- "> " prefix for unfocused pane items

### Must Have
- Dynamic border styling based on active_block
- Dynamic highlight symbols

---

## TODOs

- [x] 1. Update sidebar List rendering to use active_block for border color and highlight_symbol
- [x] 2. Update content List rendering to use active_block for border color and highlight_symbol
- [x] 3. Add details panel border styling based on ActiveBlock::Highlighted
  
  **What to do**: When rendering details panel, use `if self.active_block == ActiveBlock::Highlighted` for border color
  
  **References**:
  - `src/ui/app.rs:410-412` - details_area assignment

---

## Verification

- [ ] cargo build passes
- [ ] Right Arrow opens details panel (from previous plan)
- [ ] Tab cycles focus between panels
- [ ] Border color changes when focus changes

---

## Commit Strategy
- Message: `feat(ui): add focused pane border highlighting`
- Files: src/ui/app.rs