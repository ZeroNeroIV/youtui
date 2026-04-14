# Fix: Search Enter Key + Tab Panel Navigation

## TL;DR

> **Quick Summary**: Fix two UX issues in youtui-rs - Enter key to play search results, and Tab key to switch between sidebar/content/right panel.

> **Deliverables**:
> - Enter key plays selected search result
> - Tab cycles through: Sidebar → Content → Right Panel (Highlighted)
> - Mouse click also updates focus correctly

> **Estimated Effort**: Short
> **Parallel Execution**: NO - sequential (small fixes)
> **Critical Path**: Fix ActiveBlock enum → Add Tab handling → Fix search Enter → Verify

---

## Context

### Original Request
User reported:
1. Pressing Enter on search results doesn't play the video
2. Tab key doesn't switch to the Highlighted videos section (right panel)

### Initial Investigation Findings
- **Search Enter Issue**: Code exists to handle Enter in Search mode (lines 769-779 in app.rs), but not working. Likely either:
  - `list_state.selected()` returning None despite my earlier fix
  - Focus/rendering issue where search input captures all keys
  
- **Tab Navigation Issue**: 
  - `ActiveBlock` enum only has `Sidebar` and `Content` (no right panel)
  - No Tab key handling exists in any key handler

---

## Work Objectives

### Core Objective
Enable proper keyboard navigation and video playback in search mode.

### Concrete Deliverables
1. Search results can be played with Enter key
2. Tab cycles between Sidebar, Content, and Right panel
3. Active block is visually indicated and functionally active

### Definition of Done
- [ ] Press Enter on selected search result triggers `play_search_video()`
- [ ] Press Tab cycles: Sidebar → Content → Highlighted → Sidebar
- [ ] Visual indicator shows which block has focus

### Must Have
- Enter key works on search results
- Tab navigation works between all three panels

### Must NOT Have
- No breaking changes to existing functionality
- No regression in other modes (History, Saved, Playlist)

---

## Verification Strategy

> **Manual verification required** - Build passes, but these are UI/UX fixes needing hands-on testing.

### QA Scenarios

**Scenario: Enter plays search result**
1. Start app, press `s` to enter Search mode
2. Type a search query, press Enter to search
3. Wait for results to load
4. Use Up/Down to navigate results
5. Press Enter on selected result
- Expected: Video plays (mpv launches)

**Scenario: Tab cycles panels**
1. In any mode, press Tab
2. Observe focus moves from Sidebar → Content → Right panel
3. Continue pressing Tab - should cycle back to Sidebar

---

## Execution Strategy

### Tasks (Sequential)

**Task 1: Add Highlighted variant to ActiveBlock enum**
- Location: `src/ui/app.rs` line ~58
- Add: `Highlighted` to `ActiveBlock` enum

**Task 2: Add Tab key handling**
- Location: `src/ui/app.rs` - in `handle_events` for each AppMode
- Add: `KeyCode::Tab` case that cycles `active_block`

**Task 3: Update mouse click handling**
- Location: `src/ui/app.rs` - around line 1524-1533
- Add: Handle clicks on right panel area to set `ActiveBlock::Highlighted`

**Task 4: Debug search Enter issue**
- Add debug logging to verify key is reaching handler
- Verify list_state is properly set when results arrive
- Fix any remaining issue in the Enter handler path

---

## TODOs

- [x] 1. Add Highlighted variant to ActiveBlock enum

  **What to do**: Add `Highlighted` to the `ActiveBlock` enum to represent the right panel (Highlighted videos section)

  **References**:
  - `src/ui/app.rs:58` - ActiveBlock enum definition

- [x] 2. Add Tab key handling to cycle blocks

  **What to do**: Add KeyCode::Tab handling that cycles: Sidebar → Content → Highlighted → Sidebar

  **References**:
  - `src/ui/app.rs:674+` - Event handler for each AppMode

- [x] 3. Update mouse handling for right panel

  **What to do**: Update mouse click detection to set ActiveBlock::Highlighted when clicking right panel area

  **References**:
  - `src/ui/app.rs:1524-1533` - Current mouse click handling

- [x] 4. Verify and fix search Enter

  **What to do**: Ensure Enter key in Search mode properly plays the selected video

  **References**:
  - `src/ui/app.rs:769-779` - Current Search mode Enter handling

---

## Success Criteria

### Verification Commands
- `cargo build` - Must pass
- Manual testing: Enter plays search result
- Manual testing: Tab cycles through all panels