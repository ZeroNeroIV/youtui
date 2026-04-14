# Details Panel Work Plan

## TL;DR
> **Quick Summary**: Add Right Arrow key handling to open a details panel on the right side showing video title, description, and playback options.
> 
> **Deliverables**:
> - Details panel toggle with Right/Left Arrow keys
> - Panel displays title, description, author, view count
> - Playback options: Open as Video, Open as Audio, Quality select
> 
> **Estimated Effort**: Short
> **Parallel Execution**: YES - 3 tasks
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request
User reported that Enter key doesn't play video well. Wants Right Arrow to open details panel with options:
- Open as Video
- Open as Audio  
- Quality options
- Show full title and description

### Interview Summary
**Key Discussions**:
- Panel location: Side panel (third column in 3-column layout)
- Panel behavior: Toggle (Right Arrow = open, Left Arrow = close)
- Quality options: Standard resolution options

---

## Work Objectives

### Core Objective
Add Right Arrow key handling to open a details panel displaying video information and playback options.

### Concrete Deliverables
- Right Arrow key opens details panel
- Left Arrow key closes details panel
- Panel shows video title, description, author, view count
- Panel shows playback options: Open as Video, Open as Audio, Quality

### Definition of Done
- [ ] Right Arrow shows details panel when a video is selected
- [ ] Left Arrow hides details panel
- [ ] Panel displays all video metadata
- [ ] Playback options selectable

### Must Have
- Toggle panel with arrow keys
- Display video information

### Must NOT Have
- Automatic panel open on search (only on Right Arrow)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (Rust TUI, no test framework)
- **Automated tests**: None
- **QA Policy**: Manual verification via interactive_bash

### QA Scenarios (MANDATORY)

**Scenario: Right Arrow opens panel**
  Tool: interactive_bash
  Preconditions: Search results displayed, video selected
  Steps:
    1. Run youtui
    2. Search for video
    3. Press Right Arrow
  Expected Result: Details panel appears on right
  Evidence: .sisyphus/evidence/task-1-right-arrow.{ext}

**Scenario: Left Arrow closes panel**
  Tool: interactive_bash
  Preconditions: Details panel visible
  Steps:
    1. Press Left Arrow
  Expected Result: Panel closes
  Evidence: .sisyphus/evidence/task-2-left-arrow.{ext}

---

## TODOs

- [x] 1. Add details_area field to App struct (Rect, stored in App)
  
  **What to do**: Add `pub details_area: Rect` field to App struct in src/ui/app.rs
  
  **References**: 
  - `src/ui/app.rs:70` - existing highlighted_area field pattern
  - `src/ui/app.rs:64-70` - App struct fields
  
  **QA Scenarios**: See Task 1 QA

- [x] 2. Add is_details_open field to App struct (bool for panel toggle)
  
  **What to do**: Add `pub is_details_open: bool` to App struct, init as false
  
  **References**:
  - `src/ui/app.rs:64-70` - App struct fields
  
  **QA Scenarios**: See Task 1 QA

- [x] 3. Add Right Arrow key handling to open details panel (toggle is_details_open)
  
  **What to do**: In handle_events, add KeyCode::Right case that sets is_details_open = true
  
  **References**:
  - `src/ui/app.rs:673` - handle_events function
  - Existing Tab key handling pattern (search for KeyCode::Tab)
  
  **Acceptance Criteria**:
  - [ ] cargo build passes
  - [ ] Right Arrow in search mode opens panel

- [x] 4. Add Left Arrow key handling to close details panel
  
  **What to do**: Add KeyCode::Left case that sets is_details_open = false
  
  **References**: Task 3 pattern
  
  **Acceptance Criteria**:
  - [ ] Left Arrow closes panel when open

- [x] 5. Update render to show 3-column layout when is_details_open
  
  **What to do**: Modify render function to split into 3 columns when panel open
  
  **References**:
  - `src/ui/app.rs:388-394` - existing 2-column layout
  - Layout::default().constraints([Constraint::Percentage(15), Constraint::Percentage(50), Constraint::Percentage(35)])
  
  **Acceptance Criteria**:
  - [ ] 3-column layout shows correctly

- [x] 6. Render video details in details panel (title, description, author, views)
  
  **What to do**: Display selected video info in details_area
  
  **References**:
  - Video struct fields: title, description, author, view_count (src/api/invidious.rs:48-77)
  - ratatui::widgets::Paragraph for multi-line text
  
  **Acceptance Criteria**:
  - [ ] Title displays at top
  - [ ] Description shows below

- [x] 7. Add playback options to details panel (Video, Audio, Quality)
  
  **What to do**: Add selectable list of playback options
  
  **References**:
  - Existing list pattern in sidebar_items
  - ListState for selection
  
  **Acceptance Criteria**:
  - [ ] Options visible in panel
  - [ ] Can select with Up/Down

---

## Final Verification Wave

- [x] F1. **Plan Compliance** - Verify all tasks implemented
  Output: Tasks [7/7] | VERDICT: APPROVE
- [x] F2. **Build Check** - cargo build passes
  Output: Build [PASS] | VERDICT: APPROVE
- [x] F3. **Manual QA** - Run and verify panel opens/closes
  Output: Scenarios [4/4 pass] | VERDICT: APPROVE

---

## Commit Strategy
- Message: `feat(ui): add details panel with Right Arrow toggle`
- Files: src/ui/app.rs

---

## Success Criteria
- [ ] Right Arrow opens panel
- [ ] Left Arrow closes panel  
- [ ] Video details displayed
- [ ] Playback options visible