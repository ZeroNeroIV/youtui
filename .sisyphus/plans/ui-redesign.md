# Full UI Redesign — Modern/Minimal

## TL;DR

> **Quick Summary**: Complete visual overhaul of youtui-rs terminal UI with modern/minimal aesthetics — whitespace-driven layouts, minimal borders, clean typography hierarchy. All existing functionality preserved. Views migrated one-by-one.
>
> **Deliverables**:
> - New component library (header, item card, divider, tab bar, progress bar)
> - Updated theme palettes (all 6 themes modernized)
> - Redesigned views (Main, Search, History, Saved, Playlist, Settings)
> - Zero functional regressions
>
> **Estimated Effort**: Large
> **Parallel Execution**: YES — 3 waves
> **Critical Path**: Foundation → Components → Views → Integration

---

## Context

### Original Request
Full UI redesign of youtui-rs (YouTube terminal client) with modern/minimal style.

### Interview Summary
- **Style**: Modern/Minimal — clean lines, generous spacing, subtle/no borders, focus on content
- **Scope**: Full redesign of all views (Main, Search, History, Saved, Playlist, Settings)
- **Keep**: All existing functionality (video playback, queue, search, history, saved, playlists, settings)
- **No interaction changes**: Navigation/keybindings remain identical
- **Reference**: No specific design reference provided

### Research Findings
- **Current Layout**: Sidebar 20% + Content 80%, heavy borders via Block::default().borders(Borders::ALL)
- **Current Components**: render_error, render_empty_state, render_loading, render_status_bar (basic)
- **Modern Ratatui**: Constraint-based layouts, whitespace separation, typography hierarchy, Tailwind-inspired palettes
- **Component trait**: Modular approach with co-located state/events/render
- **Ratatui recipes**: Grid, centered rect, collapse borders, dynamic layouts available

### Metis Review

**Identified Gaps (addressed)**:
- No visual regression tests → Added per-view buffer assertions and cargo test regression checks
- Risk of fragmented "minimal" interpretation → Strict Component Library enforcement, no one-off styling
- Performance risk from complex nesting → Added render-time profiling guidance
- Long title truncation → Explicit truncation at 40 chars with "..." in QA scenarios

**Guardrails Applied**:
- NO changes to backend/API logic (`src/api/`, `src/db/`, `src/player/`)
- NO new features — redesign only
- NO theme engine overhaul — update existing Theme struct values only
- NO config file changes
- Strict Component Library usage — no one-off Block styling in view files
- NO Borders::ALL on content blocks — use whitespace/dividers instead

---

## Work Objectives

### Core Objective
Redesign the entire UI layer of youtui-rs with a modern/minimal aesthetic while preserving 100% of existing functionality.

### Concrete Deliverables
- `src/ui/components.rs` — New component library (6 new components)
- `src/ui/theme.rs` — Modernized theme palettes (all 6 themes)
- `src/ui/app.rs` — Redesigned Main, Search, History, Saved, Playlist views
- `src/ui/settings.rs` — Redesigned Settings view
- `src/ui/mod.rs` — Updated module exports
- All existing functionality (playback, search, history, saved, playlists) unchanged

### Definition of Done
- [ ] `cargo test` passes with 0 failures (existing + any new tests)
- [ ] `cargo clippy` reports 0 warnings
- [ ] `cargo fmt` applied
- [ ] All 6 views render correctly (agent-verified via Playwright screenshots)
- [ ] No `Borders::ALL` on content blocks (grep verified)
- [ ] All 6 themes render without crashes

### Must Have
- Modern/minimal visual style (whitespace-driven, minimal borders)
- All existing views fully functional
- All 6 themes work with the new design
- Responsive layout (works 80-200 columns)
- No functional regressions

### Must NOT Have
- New features beyond UI redesign
- Changes to backend, API, database, or player logic
- Heavy box-drawing borders on content areas
- AI slop (excessive comments, over-abstraction, generic names)
- One-off styling outside the Component Library

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### QA Policy
Every task includes agent-executed QA scenarios. Evidence saved to `.sisyphus/evidence/`.

**Frontend/UI**: Use Playwright via `/playwright` skill — Navigate app, verify views render, capture screenshots.
**Build**: Use `cargo check` / `cargo test` / `cargo clippy` for logic verification.
**Grep**: Use `ast_grep_search` to verify no `Borders::ALL` violations.

### Regression Policy
After every view redesign task:
- Run `cargo test` — must pass all existing tests
- Run `cargo clippy` — must report 0 new warnings
- Verify view renders with Playwright screenshots

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — scaffolding + component library):
├── Task 1: Audit current Border usage + create design tokens
├── Task 2: Create modern component library (6 components)
├── Task 3: Modernize theme palettes (all 6 themes)
└── Task 4: Update mod.rs exports

Wave 2 (Views — parallel redesign, max throughput):
├── Task 5: Redesign Main view (sidebar + content list)
├── Task 6: Redesign History + Saved views
├── Task 7: Redesign Playlist view
├── Task 8: Redesign Search view
└── Task 9: Redesign Settings view

Wave 3 (Integration + polish):
├── Task 10: Final integration + regression testing
├── Task 11: Clippy + fmt + final quality pass
└── Task 12: Commit + push

Critical Path: Task 1 → Task 2 → Task 3 → Task 4 → Tasks 5-9 (parallel) → Task 10 → Task 11 → Task 12
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 4 (Wave 1) then 5 (Wave 2)
```

### Dependency Matrix

- **1**: - → 2, 3
- **2**: 1 → 5, 6, 7, 8, 9
- **3**: 1 → 5, 6, 7, 8, 9
- **4**: 2, 3 → 5, 6, 7, 8, 9
- **5**: 2, 3, 4 → 10
- **6**: 2, 3, 4 → 10
- **7**: 2, 3, 4 → 10
- **8**: 2, 3, 4 → 10
- **9**: 2, 3, 4 → 10
- **10**: 5, 6, 7, 8, 9 → 11
- **11**: 10 → 12
- **12**: 11 → (done)

### Agent Dispatch Summary

- **1**: **4** — T1 → `deep`, T2 → `visual-engineering`, T3 → `visual-engineering`, T4 → `quick`
- **2**: **5** — T5 → `visual-engineering`, T6 → `visual-engineering`, T7 → `visual-engineering`, T8 → `visual-engineering`, T9 → `visual-engineering`
- **3**: **3** — T10 → `unspecified-high`, T11 → `quick`, T12 → `quick`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

> Every task MUST have: Recommended Agent Profile + QA Scenarios + Acceptance Criteria.
> A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.

- [x] 1. **Audit Current Border Usage + Create Design Tokens** — `deep`

  **What to do**:
  - Use `ast_grep_search` to find all instances of `Block::default().borders(...)` and `Borders::ALL` across `src/ui/`
  - Catalog every border usage: which file, which view, what type of border, what the block contains
  - Classify each as: REMOVE (replace with whitespace), REDUCE (use single side), KEEP (e.g., focused elements)
  - Create a `DesignTokens` struct or constants in `components.rs`:
    - `PADDING_SM: u16 = 1` (1-char padding)
    - `PADDING_MD: u16 = 2` (2-char padding)
    - `PADDING_LG: u16 = 3` (3-char padding)
    - `ITEM_GAP: u16 = 1` (gap between list items)
    - `SIDEBAR_WIDTH: u16 = 15` (15% sidebar)
    - `TRUNCATE_LEN: usize = 40` (title truncation length)
    - `MIN_TERMINAL_WIDTH: u16 = 80`
    - `MIN_TERMINAL_HEIGHT: u16 = 20`
  - Document the "Modern/Minimal" design rules as comments in `components.rs`:
    - Rule 1: No `Borders::ALL` on content blocks
    - Rule 2: Use `PADDING_MD` for item interiors
    - Rule 3: Use `ITEM_GAP` between list items
    - Rule 4: Accent color for selection/focus only
    - Rule 5: Secondary color for metadata/timestamps

  **Must NOT do**:
  - Do NOT modify any render logic yet
  - Do NOT change theme.rs values yet
  - Do NOT touch `src/player/`, `src/api/`, `src/db/` directories

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: This task requires thorough codebase analysis, pattern classification, and design system thinking. It's foundational work that affects all subsequent tasks.
  - **Skills**: []
    - `ast_grep_search`: Used to find all Block/border patterns across UI files

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Tasks 2, 3, 4
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/ui/app.rs:1-100` — Main UI file, contains all view render functions
  - `src/ui/components.rs:1-50` — Existing components, reference for existing patterns
  - `src/ui/settings.rs:1-50` — Settings view, has border usage to audit
  - `src/ui/theme.rs:1-50` — Theme struct definition

  **Acceptance Criteria**:
  - [ ] `ast_grep_search` ran on all UI files for `Borders::ALL` and `Block::default()`
  - [ ] Audit report created in comments at top of `src/ui/components.rs`
  - [ ] `DesignTokens` struct or module created with all spacing constants
  - [ ] Design rules documented as comments

  **QA Scenarios**:

  ```
  Scenario: Audit finds all border usage
    Tool: ast_grep_search
    Preconditions: Clean codebase
    Steps:
      1. Run ast_grep_search pattern='Borders::ALL' lang=rust paths=['src/ui/']
      2. Run ast_grep_search pattern='Block::default()' lang=rust paths=['src/ui/']
      3. Count matches per file
    Expected Result: Complete list of all border usages with file:line locations
    Evidence: .sisyphus/evidence/task-1-audit.md

  Scenario: DesignTokens constants are defined
    Tool: Bash
    Preconditions: DesignTokens struct/module created
    Steps:
      1. grep -n "PADDING_SM\|PADDING_MD\|PADDING_LG\|ITEM_GAP\|SIDEBAR_WIDTH\|TRUNCATE_LEN" src/ui/components.rs
    Expected Result: All 6 constants defined with correct values
    Evidence: .sisyphus/evidence/task-1-tokens.md
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-1-audit.md` — Full border usage audit
  - `.sisyphus/evidence/task-1-tokens.md` — Design tokens definition

  **Commit**: YES
  - Message: `refactor(ui): audit and create design tokens for modern layout`
  - Files: `src/ui/components.rs`
  - Pre-commit: `cargo check`

- [x] 2. **Create Modern Component Library** — `visual-engineering`

  **What to do**:
  Add 6 new components to `src/ui/components.rs`. All use `DesignTokens` from Task 1.

  **A. `render_header(area, title, subtitle, theme)`**
  - Full-width title bar
  - `title`: Bold, accent color, left-aligned
  - `subtitle`: Muted/secondary color, right-aligned (optional — show only if provided)
  - No borders — uses `Paragraph` with padding
  - Height: `PADDING_LG + 1` lines

  **B. `render_item_card(area, title, meta, theme, is_selected, is_focused)`**
  - Video/playlist item card with structured layout
  - `title`: `theme.foreground`, truncated at `TRUNCATE_LEN` + "..."
  - `meta`: `theme.secondary` — e.g., "Channel • 1.2K views • 3:45"
  - Layout: Title on top, meta below, both padded with `PADDING_MD`
  - `is_selected`: Background = `theme.accent` at 20% opacity, title = accent color
  - `is_focused`: Thin left border (1 char) in `theme.accent`
  - Gap between cards: `ITEM_GAP` (achieved via padding or spacing)
  - Single item per `area`

  **C. `render_divider(area, theme, direction)`**
  - Subtle horizontal or vertical divider line
  - Uses `Line::raw("─")` (horizontal) or `│` (vertical) in `theme.border` color
  - No block/border — just a styled `Paragraph` or `Line`
  - Height/width: 1 line/column

  **D. `render_tab_bar(area, tabs, selected_index, theme)`**
  - Horizontal tab navigation bar
  - `tabs`: slice of tab names (e.g., ["History", "Saved", "Playlists"])
  - Selected tab: `theme.accent` text, underlined
  - Unselected tabs: `theme.secondary` text
  - Padding: `PADDING_SM` above and below
  - Height: `PADDING_LG + 1` lines
  - Tab separator: single space character

  **E. `render_progress_bar(area, current, total, theme)`**
  - Thin horizontal progress indicator
  - Shows video playback progress (e.g., "━━━━━━░░░░░ 6:30 / 10:00")
  - Filled portion: `theme.accent`
  - Empty portion: `theme.border`
  - Text overlay: "current / total" in `theme.foreground`
  - Height: 1 line
  - Padding: `PADDING_SM` left/right

  **F. `render_info_bar(area, items, theme)`**
  - Horizontal bar showing key-value pairs (e.g., "Queue: 5 • Theme: Tokyo • Quality: 1080p")
  - Each item: label in `theme.secondary` + value in `theme.foreground`
  - Separator: " • " in `theme.border`
  - Height: 1 line
  - Padding: `PADDING_SM`

  **G. Update existing `render_empty_state`** — Make it use `PADDING_MD` and modern styling (no heavy borders, just centered content with icon in accent + text)

  **Must NOT do**:
  - Do NOT use `Borders::ALL` on any component
  - Do NOT use `Block` for backgrounds — use `Style::bg()` directly
  - Do NOT hardcode padding values — always use `DesignTokens`
  - Do NOT add components not listed above

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: This is pure UI design and implementation — building a component library with specific visual specifications. Needs both design sensibility and ratatui technical skill.
  - **Skills**: []
    - Ratatui widget patterns: Required for building the components correctly

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Tasks 5, 6, 7, 8, 9
  - **Blocked By**: Task 1 (DesignTokens must exist first)

  **References**:
  - `src/ui/components.rs` — Existing component file, extend this
  - `src/ui/theme.rs` — Theme struct with colors to use
  - `ratatui.rs/recipes/layout/` — Layout recipes for centered content patterns
  - `ratatui.rs/concepts/application-patterns/component-architecture/` — Component trait patterns
  - `src/ui/app.rs:render_main_view()` — Current list item rendering for reference on data structure

  **Acceptance Criteria**:
  - [ ] All 6 new components implemented in `src/ui/components.rs`
  - [ ] `render_empty_state` updated with modern styling
  - [ ] All components use `DesignTokens` constants (no hardcoded values)
  - [ ] `cargo check` passes
  - [ ] `ast_grep_search` pattern='Borders::ALL' lang=rust paths=['src/ui/components.rs'] → 0 matches

  **QA Scenarios**:

  ```
  Scenario: All components render without panic
    Tool: Bash
    Preconditions: Components implemented
    Steps:
      1. cargo check src/ui/components.rs
    Expected Result: Compiles with 0 errors
    Evidence: .sisyphus/evidence/task-2-check.md

  Scenario: render_item_card renders correctly for selected item
    Tool: interactive_bash
    Preconditions: App built and running
    Steps:
      1. Start the app
      2. Navigate to a view with list items
      3. Verify selected item has accent background
      4. Verify unselected items have no background
      5. Verify title truncation at 40 chars
    Expected Result: Selected item clearly distinguished, titles truncated with "..."
    Failure Indicators: All items look identical, no visual selection indicator
    Evidence: .sisyphus/evidence/task-2-card-selected.png

  Scenario: render_header displays title and subtitle
    Tool: interactive_bash
    Preconditions: App built
    Steps:
      1. Start app, navigate to any view
      2. Verify header shows view title (left) and subtitle info (right)
    Expected Result: Title left-aligned in accent color, subtitle right-aligned in secondary color
    Evidence: .sisyphus/evidence/task-2-header.png

  Scenario: render_progress_bar shows playback progress
    Tool: interactive_bash
    Preconditions: Video playing
    Steps:
      1. Play a video
      2. Verify progress bar shows filled/empty portions with time overlay
    Expected Result: "━━━━━━░░░░░ 6:30 / 10:00" format
    Evidence: .sisyphus/evidence/task-2-progress.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-2-check.md` — Cargo check output
  - `.sisyphus/evidence/task-2-card-selected.png` — Selected item screenshot
  - `.sisyphus/evidence/task-2-header.png` — Header component screenshot
  - `.sisyphus/evidence/task-2-progress.png` — Progress bar screenshot

  **Commit**: YES
  - Message: `feat(ui): add modern component library`
  - Files: `src/ui/components.rs`
  - Pre-commit: `cargo check`

- [x] 3. **Modernize Theme Palettes (All 6 Themes)** — `visual-engineering`

  **What to do**:
  Update the color values in `src/ui/theme.rs` for all 6 themes to fit a modern palette. The goal is a cohesive dark/light base with a distinctive accent per theme.

  **Approach**: Keep the existing `Theme` struct fields. Update ONLY the color values to be more modern:

  **A. `terminal` theme** (green/black terminal classic):
  - background: #0d1117 (near-black, not pure black)
  - foreground: #c9d1d9 (light gray, not pure white)
  - accent: #3fb950 (GitHub green)
  - secondary: #8b949e (muted gray)
  - highlight: #238636 (darker green for backgrounds)
  - border: #21262d (subtle dark gray)
  - focused_border: #3fb950
  - error: #f85149
  - success: #3fb950

  **B. `tokyo_night` theme** (blue/purple night sky):
  - background: #1a1b26
  - foreground: #c0caf5
  - accent: #7aa2f7 (blue)
  - secondary: #565f89 (muted purple)
  - highlight: #283457
  - border: #292e42
  - focused_border: #7aa2f7
  - error: #f7768e
  - success: #9ece6a

  **C. `monokai_pro` theme** (warm/dark):
  - background: #2d2a2e
  - foreground: #f8f8f2
  - accent: #a6e22e (lime green)
  - secondary: #75715e (muted olive)
  - highlight: #49483e
  - border: #3e3d32
  - focused_border: #a6e22e
  - error: #f92672
  - success: #a6e22e

  **D. `nord` theme** (ice blue):
  - background: #2e3440
  - foreground: #eceff4
  - accent: #88c0d0 (cyan)
  - secondary: #4c566a (muted gray-blue)
  - highlight: #3b4252
  - border: #434c5e
  - focused_border: #88c0d0
  - error: #bf616a
  - success: #a3be8c

  **E. `catppuccin_mocha` theme** (popular modern dark):
  - background: #1e1e2e
  - foreground: #cdd6f4
  - accent: #cba6f7 (mauve/purple)
  - secondary: #6c7086 (gray)
  - highlight: #313244
  - border: #45475a
  - focused_border: #cba6f7
  - error: #f38ba8
  - success: #a6e3a1

  **F. `gruvbox` theme** (retro warm):
  - background: #282828
  - foreground: #ebdbb2 (cream)
  - accent: #fabd2f (gold)
  - secondary: #928374 (muted brown)
  - highlight: #3c3836
  - border: #504945
  - focused_border: #fabd2f
  - error: #fb4934
  - success: #b8bb26

  **Must NOT do**:
  - Do NOT change the `Theme` struct fields (only update color values)
  - Do NOT add new themes
  - Do NOT change how themes are loaded
  - Do NOT touch any other files

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Color palette work requires design sensibility — choosing cohesive, readable color combinations for terminal use. Not purely logical, needs aesthetic judgment.
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Tasks 5, 6, 7, 8, 9
  - **Blocked By**: Task 1 (DesignTokens audit must complete)

  **References**:
  - `src/ui/theme.rs` — Theme struct and existing theme definitions
  - `tailwindcss.com/colors` — Reference for Tailwind color naming conventions used in modern themes
  - `catppuccin.com` — Catppuccin palette reference (Mocha theme)
  - `github.com/folke/tokyo-night.nvim` — Tokyo Night theme reference

  **Acceptance Criteria**:
  - [ ] All 6 themes updated with modern color values
  - [ ] `cargo check` passes
  - [ ] Each theme has distinct, readable accent + background contrast
  - [ ] All themes compile without color errors

  **QA Scenarios**:

  ```
  Scenario: All themes compile and load
    Tool: Bash
    Preconditions: Theme values updated
    Steps:
      1. cargo check src/ui/theme.rs
      2. Iterate each theme name (terminal, tokyo_night, monokai_pro, nord, catppuccin_mocha, gruvbox)
      3. Manually verify color values look reasonable in code
    Expected Result: Compiles, all themes have valid Color values
    Evidence: .sisyphus/evidence/task-3-themes.md

  Scenario: Themes render in app without crash
    Tool: interactive_bash
    Preconditions: App built with new themes
    Steps:
      1. Start app
      2. Go to Settings
      3. Cycle through all 6 themes
      4. Verify each theme renders without crashes or invisible text
    Expected Result: All 6 themes render correctly, no crashes
    Failure Indicators: Theme switch causes panic or unreadable text
    Evidence: .sisyphus/evidence/task-3-theme-switch.gif
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-3-themes.md` — Theme color definitions
  - `.sisyphus/evidence/task-3-theme-switch.gif` — All themes cycling

  **Commit**: YES
  - Message: `style(ui): modernize all 6 theme palettes`
  - Files: `src/ui/theme.rs`
  - Pre-commit: `cargo check`

- [x] 4. **Update Module Exports** — `quick`

  **What to do**:
  - In `src/ui/mod.rs`, ensure all public exports are correct for the new component library
  - Add `pub use components::{render_header, render_item_card, render_divider, render_tab_bar, render_progress_bar, render_info_bar, render_empty_state}` if not already exported
  - Ensure `theme` module is still properly exported
  - Run `cargo check` to verify all exports work

  **Must NOT do**:
  - Do NOT change any logic, only exports

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple export maintenance, minimal logic changes.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Tasks 5-9
  - **Blocked By**: Tasks 2, 3 (components and themes must be ready)

  **References**:
  - `src/ui/mod.rs` — Current module exports

  **Acceptance Criteria**:
  - [ ] All new components exported from mod.rs
  - [ ] `cargo check` passes with full project

  **QA Scenarios**:

  ```
  Scenario: All components accessible from ui module
    Tool: Bash
    Preconditions: mod.rs updated
    Steps:
      1. cargo check 2>&1 | head -50
    Expected Result: 0 errors related to missing exports
    Evidence: .sisyphus/evidence/task-4-exports.md
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-4-exports.md` — Cargo check output

  **Commit**: YES
  - Message: `chore(ui): update module exports for new components`
  - Files: `src/ui/mod.rs`
  - Pre-commit: `cargo check`

- [ ] 5. **Redesign Main View** — `visual-engineering`

  **What to do**:
  Rewrite the `render_main_view` function in `src/ui/app.rs` using the new component library. This is the most complex view.

  **New Layout**:
  ```
  ┌─[SIDEBAR 15%]──[CONTENT 85%]────────────────────────┐
  │  Logo/Title   │ [render_header: "Browse" + queue info]│
  │  ─────────    │ ─────────────────────────────────────│
  │  > Browse     │ [render_item_card × N]                │
  │    History    │   - Title (40 chars truncated)       │
  │    Saved      │   - Meta: Channel • Views • Duration │
  │    Playlists  │   - is_selected → accent bg          │
  │    Search     │   - is_focused → accent left border  │
  │    Settings   │                                      │
  │  ─────────    │ [render_divider if more items]        │
  │  Queue: 3     │                                      │
  │  Playing: ... │                                      │
  │  ─────────    │ [render_info_bar: Quality | Theme]   │
  │  [Theme name] │                                      │
  └───────────────┴──────────────────────────────────────┘
  ```

  **Sidebar changes**:
  - Width: `Constraint::Percentage(15)` (down from 20%)
  - Remove heavy `Block` borders — use `render_divider` between sections
  - Active item: `theme.accent` text + subtle background
  - Inactive items: `theme.secondary` text
  - Queue info at bottom: `render_info_bar` style

  **Content area changes**:
  - Header: `render_header` with "Browse" title + queue status
  - Items: `render_item_card` for each video, using `self.list_state` for selection
  - Bottom bar: `render_info_bar` showing current theme + quality setting
  - Remove all `Block` borders from the content area
  - Use `ITEM_GAP` between cards (via padding on each card or explicit gap)

  **Context Menu** (still needed for video actions):
  - Keep the floating overlay but style it minimally:
    - No heavy border, use `theme.accent` single border or shadow effect
    - Clean list of actions (Play, Add to Queue, Save, etc.)
    - Use `theme.background` with accent border

  **Must NOT do**:
  - Do NOT change any data fetching or state management logic
  - Do NOT change how `self.items` or `self.list_state` are populated
  - Do NOT use `Borders::ALL` on content blocks
  - Do NOT remove the context menu functionality

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: This is the core view redesign — needs careful layout composition, correct component usage, and visual judgment. Most complex view in the app.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8, 9)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 4

  **References**:
  - `src/ui/app.rs:render_main_view()` — Current implementation to replace
  - `src/ui/components.rs` — New component library to use
  - `src/ui/theme.rs` — Theme colors for styling
  - `src/state.rs:Video` — Video struct to understand data fields (title, channel, views, duration)
  - `ratatui.rs/recipes/layout/` — Layout recipes for nested layouts

  **Acceptance Criteria**:
  - [ ] `render_main_view` uses only new components (render_header, render_item_card, render_divider, render_info_bar)
  - [ ] Sidebar width is 15% (`Constraint::Percentage(15)`)
  - [ ] No `Borders::ALL` in main view rendering
  - [ ] Context menu still functions (Play, Add to Queue, Save actions)
  - [ ] `cargo check` passes
  - [ ] `cargo test` passes (no regressions)

  **QA Scenarios**:

  ```
  Scenario: Main view renders with modern layout
    Tool: interactive_bash
    Preconditions: App built, main view redesigned
    Steps:
      1. Start app
      2. Verify sidebar shows menu items (Browse, History, Saved, Playlists, Search, Settings)
      3. Verify active item is highlighted with accent color
      4. Verify content area shows video items with title + meta
      5. Verify selected item has accent background
      6. Verify queue info shows at sidebar bottom
    Expected Result: Clean two-panel layout with minimal borders, clear hierarchy
    Failure Indicators: Heavy borders visible, no selection highlight, items overflow
    Evidence: .sisyphus/evidence/task-5-main-view.png

  Scenario: Context menu appears on right-click
    Tool: interactive_bash
    Preconditions: App on main view with items
    Steps:
      1. Select a video item
      2. Press Enter or right-click
      3. Verify context menu appears with actions
      4. Verify menu has minimal border (accent color, not box-drawing)
      5. Press Escape to dismiss
    Expected Result: Menu appears and dismisses correctly
    Evidence: .sisyphus/evidence/task-5-context-menu.png

  Scenario: Long video title truncates at 40 characters
    Tool: interactive_bash
    Preconditions: App on main view
    Steps:
      1. Navigate to a video with title > 40 chars (or mock one)
      2. Verify title shows first 37 chars + "..."
    Expected Result: "How to build a TUI app in Rust usin..." (40 chars + "...")
    Evidence: .sisyphus/evidence/task-5-truncation.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-5-main-view.png` — Main view screenshot
  - `.sisyphus/evidence/task-5-context-menu.png` — Context menu screenshot
  - `.sisyphus/evidence/task-5-truncation.png` — Title truncation

  **Commit**: YES
  - Message: `refactor(ui): redesign main view with modern layout`
  - Files: `src/ui/app.rs`
  - Pre-commit: `cargo test`

- [x] 6. **Redesign History + Saved Views** — `visual-engineering`

  **What to do**:
  Rewrite `render_history_view` and `render_saved_view` in `src/ui/app.rs` with the modern component library.

  **New Layout** (applies to both views):
  ```
  ┌─[HEADER: "History" or "Saved"]─────────────────────────┐
  │ [render_header: title + item count]                    │
  ├────────────────────────────────────────────────────────┤
  │ [render_item_card × N]                                 │
  │   - Watched/Saved date in meta                        │
  │   - is_selected → accent bg                            │
  │                                                        │
  ├────────────────────────────────────────────────────────┤
  │ [render_info_bar: theme name]                          │
  └────────────────────────────────────────────────────────┘
  ```

  **Key changes**:
  - Remove `Block` borders — full area is clean
  - Header shows view title + total count (e.g., "History • 42 items")
  - Item cards use `render_item_card` with meta showing: "Channel • Watched 2 days ago"
  - `is_selected` for list navigation
  - Bottom bar with theme name

  **Must NOT do**:
  - Do NOT change how `history_results` or `saved_results` are populated
  - Do NOT change any database or API logic
  - Do NOT use `Borders::ALL`

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Visual redesign of two views with similar structure. Straightforward component composition using the new library.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8, 9)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 4

  **References**:
  - `src/ui/app.rs:render_history_view()` — Current history view
  - `src/ui/app.rs:render_saved_view()` — Current saved view
  - `src/ui/components.rs` — Component library
  - `src/state.rs:HistoryEntry` — Data struct for history
  - `src/state.rs:SavedVideo` — Data struct for saved

  **Acceptance Criteria**:
  - [ ] `render_history_view` uses new components
  - [ ] `render_saved_view` uses new components
  - [ ] No `Borders::ALL` in either view
  - [ ] `cargo check` passes
  - [ ] `cargo test` passes

  **QA Scenarios**:

  ```
  Scenario: History view renders correctly
    Tool: interactive_bash
    Preconditions: App with history entries
    Steps:
      1. Start app, navigate to History view
      2. Verify header shows "History" + count
      3. Verify items show title + channel + date watched
      4. Verify selection highlighting works
    Expected Result: Clean list with modern styling
    Evidence: .sisyphus/evidence/task-6-history.png

  Scenario: Saved view renders correctly
    Tool: interactive_bash
    Preconditions: App with saved videos
    Steps:
      1. Start app, navigate to Saved view
      2. Verify header shows "Saved" + count
      3. Verify items show title + channel + date saved
      4. Verify selection highlighting works
    Expected Result: Clean list with modern styling
    Evidence: .sisyphus/evidence/task-6-saved.png

  Scenario: Empty history shows empty state
    Tool: interactive_bash
    Preconditions: App with no history
    Steps:
      1. Start app, navigate to History view
      2. Verify empty state message shows (not a blank screen)
    Expected Result: Empty state component with message
    Evidence: .sisyphus/evidence/task-6-history-empty.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-6-history.png`
  - `.sisyphus/evidence/task-6-saved.png`
  - `.sisyphus/evidence/task-6-history-empty.png`

  **Commit**: YES
  - Message: `refactor(ui): redesign history and saved views`
  - Files: `src/ui/app.rs`
  - Pre-commit: `cargo test`

- [ ] 7. **Redesign Playlist View** — `visual-engineering`

  **What to do**:
  Rewrite `render_playlist_view` in `src/ui/app.rs` with modern styling.

  **New Layout** (two states — playlist list vs playlist videos):
  ```
  ┌─[PLAYLIST LIST STATE]─────────────────────────────────┐
  │ [render_header: "Playlists" + count]                   │
  │ [render_item_card × N]                                 │
  │   - Playlist name                                      │
  │   - Meta: "12 videos • 45 min total"                  │
  │                                                        │
  ├─[PLAYLIST VIDEOS STATE]────────────────────────────────┤
  │ [render_header: "Playlist: {name}" + back action]      │
  │ [render_item_card × N]                                 │
  │   - Video title                                        │
  │   - Meta: "Duration • Position in playlist"            │
  │                                                        │
  ├─[PLAYLIST PROMPT STATE]────────────────────────────────┤
  │ [render_header: "New Playlist" or "Import Playlist"]    │
  │ [Paragraph input with minimal styling]                 │
  │ [render_info_bar: "Enter to confirm • Esc to cancel"]  │
  └────────────────────────────────────────────────────────┘
  ```

  **Key changes**:
  - Remove `Block` borders throughout
  - `render_item_card` for playlist items with custom meta (video count, total duration)
  - `render_tab_bar` at top if switching between playlists frequently
  - Clean prompt input (no heavy border box)
  - `render_info_bar` for keyboard hints in prompt mode

  **Must NOT do**:
  - Do NOT change playlist creation/import/deletion logic
  - Do NOT change `playlist_results` or `playlist_videos` population logic
  - Do NOT use `Borders::ALL`

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Playlist view has multiple states (list, videos, prompt). Needs state-aware component composition.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8, 9)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 4

  **References**:
  - `src/ui/app.rs:render_playlist_view()` — Current playlist view
  - `src/ui/components.rs` — Component library
  - `src/state.rs:Playlist` — Data struct
  - `src/state.rs:PlaylistVideo` — Data struct

  **Acceptance Criteria**:
  - [ ] All 3 playlist states (list, videos, prompt) use new components
  - [ ] No `Borders::ALL` in playlist view
  - [ ] `cargo check` passes
  - [ ] `cargo test` passes

  **QA Scenarios**:

  ```
  Scenario: Playlist list renders with modern styling
    Tool: interactive_bash
    Preconditions: App with playlists
    Steps:
      1. Start app, navigate to Playlists view
      2. Verify header shows "Playlists" + count
      3. Verify playlist items show name + video count + total duration
      4. Verify selection highlighting works
    Expected Result: Clean playlist cards
    Evidence: .sisyphus/evidence/task-7-playlist-list.png

  Scenario: Playlist videos render when playlist selected
    Tool: interactive_bash
    Preconditions: App with playlist selected
    Steps:
      1. Select a playlist from the list
      2. Verify view changes to show playlist videos
      3. Verify header shows playlist name
      4. Verify video items show title + duration
    Expected Result: Modern video list within playlist
    Evidence: .sisyphus/evidence/task-7-playlist-videos.png

  Scenario: Playlist prompt shows clean input
    Tool: interactive_bash
    Preconditions: App on new playlist creation
    Steps:
      1. Create new playlist
      2. Verify prompt appears with clean styling
      3. Verify keyboard hints at bottom
      4. Press Escape to cancel
    Expected Result: Clean prompt without heavy borders
    Evidence: .sisyphus/evidence/task-7-prompt.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-7-playlist-list.png`
  - `.sisyphus/evidence/task-7-playlist-videos.png`
  - `.sisyphus/evidence/task-7-prompt.png`

  **Commit**: YES
  - Message: `refactor(ui): redesign playlist view`
  - Files: `src/ui/app.rs`
  - Pre-commit: `cargo test`

- [x] 8. **Redesign Search View** — `visual-engineering`

  **What to do**:
  Rewrite `render_search_view` in `src/ui/app.rs` with modern styling.

  **New Layout**:
  ```
  ┌─[HEADER: Search + current query]───────────────────────┐
  │ [render_header: "Search" + query string if active]     │
  │ [Search input bar: minimal, no border]                │
  ├────────────────────────────────────────────────────────┤
  │ [render_item_card × N]  (results or loading/error)     │
  │   - Search result video                               │
  │   - Meta: Channel • Views • Duration • Upload date    │
  │                                                        │
  ├────────────────────────────────────────────────────────┤
  │ [render_info_bar: "{count} results • Quality setting]  │
  └────────────────────────────────────────────────────────┘
  ```

  **Key changes**:
  - Search input: Inline text input at top of content area — NO separate block with border
  - Use `Paragraph` with styling to create a clean input appearance
  - Input has `theme.border` bottom line to indicate it's editable
  - Results use `render_item_card` with rich metadata
  - Loading state: `render_loading` (updated in Task 2)
  - Error state: `render_error` (updated in Task 2)
  - Empty state: `render_empty_state` (updated in Task 2)
  - Bottom bar: result count + quality setting

  **Must NOT do**:
  - Do NOT change search API logic or `search_results` population
  - Do NOT use `Borders::ALL` on the search input
  - Do NOT change search event handling

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Search view has multiple states (input, loading, results, error, empty). Needs careful state-aware component composition.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7, 9)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 4

  **References**:
  - `src/ui/app.rs:render_search_view()` — Current search view
  - `src/ui/components.rs` — Component library
  - `src/state.rs:Video` — Search result data struct

  **Acceptance Criteria**:
  - [ ] Search input renders as clean inline bar (no border block)
  - [ ] All states (input, loading, results, error, empty) use new components
  - [ ] No `Borders::ALL` in search view
  - [ ] `cargo check` passes
  - [ ] `cargo test` passes

  **QA Scenarios**:

  ```
  Scenario: Search input shows with clean styling
    Tool: interactive_bash
    Preconditions: App on search view, no query
    Steps:
      1. Navigate to Search view
      2. Verify input appears as clean text bar (no box border)
      3. Verify subtle underline indicates it's editable
    Expected Result: Clean, minimal search input
    Evidence: .sisyphus/evidence/task-8-search-input.png

  Scenario: Search results render with rich metadata
    Tool: interactive_bash
    Preconditions: App on search with results
    Steps:
      1. Type a query, wait for results
      2. Verify results show title + channel + views + duration + upload date
      3. Verify selection works with arrow keys
    Expected Result: Rich item cards with comprehensive metadata
    Evidence: .sisyphus/evidence/task-8-search-results.png

  Scenario: Search error shows clean error message
    Tool: interactive_bash
    Preconditions: App on search with network error
    Steps:
      1. Trigger a search error (offline)
      2. Verify error shows with modern styling (no heavy red border)
      3. Verify hint text is visible
    Expected Result: Clean, readable error state
    Evidence: .sisyphus/evidence/task-8-search-error.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-8-search-input.png`
  - `.sisyphus/evidence/task-8-search-results.png`
  - `.sisyphus/evidence/task-8-search-error.png`

  **Commit**: YES
  - Message: `refactor(ui): redesign search view`
  - Files: `src/ui/app.rs`
  - Pre-commit: `cargo test`

- [x] 9. **Redesign Settings View** — `visual-engineering`

  **What to do**:
  Rewrite `render_settings` in `src/ui/settings.rs` with modern styling.

  **New Layout**:
  ```
  ┌─[HEADER: "Settings"]───────────────────────────────────┐
  │ [render_header: "Settings"]                            │
  ├────────────────────────────────────────────────────────┤
  │ ┌─[GENERAL SECTION]───────────────────────────────┐    │
  │ │ Theme        [current: catppuccin_mocha] ▾   │    │
  │ │ Quality      [1080p] ▾                        │    │
  │ │ Download Dir [/home/user/downloads]          │    │
  │ └──────────────────────────────────────────────┘    │
  │                                                        │
  │ ┌─[PLAYBACK SECTION]─────────────────────────────┐    │
  │ │ Autoplay     [ON] ▾                           │    │
  │ │ Volume       [75%] ████████░░                │    │
  │ └──────────────────────────────────────────────┘    │
  │                                                        │
  ├────────────────────────────────────────────────────────┤
  │ [render_info_bar: "↑↓ Navigate • Enter Select • Esc] │
  └───────────────────────────────────────────────────────┘
  ```

  **Key changes**:
  - Section headers: `render_header` with section name (small, accent color)
  - Setting rows: Two-column layout — label (secondary color, left) + value (foreground, right)
  - Use `render_divider` between sections
  - Value selector: Show current value in `theme.accent`, use `▾` indicator
  - Volume bar: Use `render_progress_bar` for volume slider
  - Bottom bar: `render_info_bar` with keyboard shortcuts
  - Remove heavy `Block` borders — use whitespace and `render_divider` for separation
  - No `Borders::ALL` on any element

  **Must NOT do**:
  - Do NOT change any settings logic or data structures
  - Do NOT change how settings are loaded/saved
  - Do NOT use `Borders::ALL`

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Settings is a form-like view with custom two-column layout. Needs careful alignment and component composition.

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7, 8)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 4

  **References**:
  - `src/ui/settings.rs` — Current settings view to replace
  - `src/ui/components.rs` — Component library
  - `src/config.rs` — Settings data structures

  **Acceptance Criteria**:
  - [ ] Settings view uses only new components
  - [ ] Two-column layout (label + value) for all settings
  - [ ] Section dividers between setting groups
  - [ ] No `Borders::ALL` in settings view
  - [ ] `cargo check` passes
  - [ ] `cargo test` passes

  **QA Scenarios**:

  ```
  Scenario: Settings renders with modern two-column layout
    Tool: interactive_bash
    Preconditions: App on settings view
    Steps:
      1. Start app, navigate to Settings
      2. Verify section headers in accent color
      3. Verify each setting shows label (muted) + value (bright)
      4. Verify section dividers between groups
      5. Verify keyboard hints at bottom
    Expected Result: Clean settings form with clear label/value separation
    Failure Indicators: Labels and values misaligned, heavy borders visible
    Evidence: .sisyphus/evidence/task-9-settings.png

  Scenario: Theme selector shows all 6 themes
    Tool: interactive_bash
    Preconditions: App on settings
    Steps:
      1. Navigate to Theme setting
      2. Cycle through themes
      3. Verify each theme changes the UI colors immediately
    Expected Result: All 6 themes selectable and apply correctly
    Evidence: .sisyphus/evidence/task-9-theme-select.gif

  Scenario: Volume slider renders as progress bar
    Tool: interactive_bash
    Preconditions: App on settings
    Steps:
      1. Navigate to Volume setting
      2. Verify it shows as a progress bar (filled + empty portions)
      3. Adjust volume and verify bar updates
    Expected Result: Visual volume slider matching render_progress_bar style
    Evidence: .sisyphus/evidence/task-9-volume.png
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-9-settings.png`
  - `.sisyphus/evidence/task-9-theme-select.gif`
  - `.sisyphus/evidence/task-9-volume.png`

  **Commit**: YES
  - Message: `refactor(ui): redesign settings view`
  - Files: `src/ui/settings.rs`
  - Pre-commit: `cargo test`

- [x] 10. **Final Integration + Regression Testing** — `unspecified-high`

  **What to do**:
  - Run full `cargo test` — all existing tests must pass
  - Run `cargo clippy` — fix any new warnings
  - Verify all views are reachable and render:
    1. Main view (browse mode)
    2. Search view (with results, empty, error states)
    3. History view
    4. Saved view
    5. Playlist view (list, videos, prompt states)
    6. Settings view
  - Verify all 6 themes work in all views
  - Verify at minimum terminal size (80 cols × 20 rows) — no panics or layout collapses
  - Verify at typical size (150 cols × 40 rows) — layout looks good
  - Check for any remaining `Borders::ALL` in UI files that shouldn't be there
  - Verify the Component Library is used consistently (no one-off styling in view files)
  - Check event handling still works: keyboard navigation, input, context menu

  **Must NOT do**:
  - Do NOT add new features
  - Do NOT change backend logic
  - Do NOT skip any view or state

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Comprehensive integration testing across all views and themes. Requires broad coverage and judgment on what constitutes a regression.

  **Parallelization**:
  - **Can Run In Parallel**: NO (must run after all views complete)
  - **Blocks**: Task 11
  - **Blocked By**: Tasks 5, 6, 7, 8, 9

  **References**:
  - All view files: `src/ui/app.rs`, `src/ui/settings.rs`
  - `src/ui/components.rs` — Component library
  - `src/ui/theme.rs` — All 6 themes

  **Acceptance Criteria**:
  - [ ] `cargo test` passes all tests
  - [ ] `cargo clippy` reports 0 new warnings
  - [ ] All 6 views render at 80 cols minimum
  - [ ] All 6 themes work without crashes
  - [ ] No `Borders::ALL` on content areas (grep verified)
  - [ ] All keyboard navigation works

  **QA Scenarios**:

  ```
  Scenario: Full regression test at minimum terminal size
    Tool: interactive_bash
    Preconditions: App built, resize to 80x20
    Steps:
      1. Resize terminal to 80 columns × 20 rows
      2. Start app
      3. Navigate through ALL 6 views
      4. Verify no panics, no layout collapses
    Expected Result: All views render within 80x20 without crashing
    Evidence: .sisyphus/evidence/task-10-min-size.txt

  Scenario: All 6 themes verified in Main view
    Tool: interactive_bash
    Preconditions: App on Main view
    Steps:
      1. Start app on Main view
      2. Go to Settings
      3. Cycle through all 6 themes
      4. Return to Main view after each theme switch
      5. Verify Main view renders correctly in each theme
    Expected Result: All 6 themes render Main view correctly
    Evidence: .sisyphus/evidence/task-10-all-themes.gif

  Scenario: grep verifies no Borders::ALL in content blocks
    Tool: Bash
    Preconditions: All views redesigned
    Steps:
      1. grep -rn "Borders::ALL" src/ui/
    Expected Result: 0 matches OR matches only in justified contexts (e.g., popups)
    Evidence: .sisyphus/evidence/task-10-borders-check.md
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-10-min-size.txt`
  - `.sisyphus/evidence/task-10-all-themes.gif`
  - `.sisyphus/evidence/task-10-borders-check.md`

  **Commit**: NO (part of Task 12 commit)

- [x] 11. **Clippy + Fmt + Final Quality Pass** — `quick`

  **What to do**:
  - Run `cargo fmt` — apply standard Rust formatting
  - Run `cargo clippy -- -D warnings` — fail on any warnings
  - Review all changed files for AI slop patterns:
    - Excessive comments (every line commented)
    - Over-abstraction (tiny functions for obvious things)
    - Generic names (data, result, item, temp)
    - Empty doc comments (`///`)
    - Commented-out code
  - Verify no `as any` or `@ts-ignore` (not applicable in Rust but check for `.unwrap()` abuse)
  - Verify no `println!` in production code (should use logging)
  - Clean up any debug prints added during development

  **Must NOT do**:
  - Do NOT add new functionality
  - Do NOT change logic

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Quality enforcement — formatting, linting, cleanup. Straightforward mechanical work.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 12
  - **Blocked By**: Task 10

  **References**:
  - All changed UI files

  **Acceptance Criteria**:
  - [ ] `cargo fmt` applied
  - [ ] `cargo clippy` reports 0 warnings
  - [ ] No AI slop patterns found
  - [ ] No debug prints in production code

  **QA Scenarios**:

  ```
  Scenario: Clippy reports zero warnings
    Tool: Bash
    Preconditions: All changes made
    Steps:
      1. cargo clippy -- -D warnings 2>&1
    Expected Result: 0 warnings, 0 errors
    Evidence: .sisyphus/evidence/task-11-clippy.md

  Scenario: Fmt check passes
    Tool: Bash
    Preconditions: All changes made
    Steps:
      1. cargo fmt --check 2>&1
    Expected Result: No formatting differences
    Evidence: .sisyphus/evidence/task-11-fmt.md
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-11-clippy.md`
  - `.sisyphus/evidence/task-11-fmt.md`

  **Commit**: NO (part of Task 12 commit)

- [ ] 12. **Commit + Push** — `quick`

  **What to do**:
  - Stage all changed files
  - Create commit with multi-line body describing the redesign
  - Push to remote
  - Verify push succeeded

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple git operations. Use git-master skill for proper commit messages.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: None (final step)
  - **Blocked By**: Task 11

  **References**:
  - `src/ui/` — All changed files

  **Acceptance Criteria**:
  - [ ] Commit created with descriptive message
  - [ ] Pushed to remote successfully
  - [ ] GitHub shows the commit

  **QA Scenarios**:

  ```
  Scenario: Changes committed and pushed
    Tool: Bash
    Preconditions: All tasks complete
    Steps:
      1. git log -1 --stat
      2. git push
    Expected Result: Commit visible on GitHub
    Evidence: .sisyphus/evidence/task-12-push.md
  ```

  **Evidence to Capture**:
  - `.sisyphus/evidence/task-12-push.md`

  **Commit**: YES (the commit itself)

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy` + `cargo fmt --check` + `cargo test`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop.
  Output: `Clippy [PASS/FAIL] | Fmt [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill)
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test all 6 themes. Test at 80 columns (minimum) and 150 columns (typical). Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Themes [N/6 render] | Size [80col OK, 150col OK] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance. Detect cross-task contamination.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **1**: `refactor(ui): audit and remove legacy Borders::ALL patterns` — src/ui/app.rs, src/ui/components.rs
- **2**: `feat(ui): add modern component library (header, item_card, divider, tab_bar, progress_bar)` — src/ui/components.rs
- **3**: `style(ui): modernize all 6 theme palettes` — src/ui/theme.rs
- **4**: `refactor(ui): redesign main view with modern layout` — src/ui/app.rs
- **5**: `refactor(ui): redesign history and saved views` — src/ui/app.rs
- **6**: `refactor(ui): redesign playlist view` — src/ui/app.rs
- **7**: `refactor(ui): redesign search view` — src/ui/app.rs
- **8**: `refactor(ui): redesign settings view` — src/ui/settings.rs
- **9**: `chore(ui): final integration and quality pass` — all changed files

---

## Success Criteria

### Verification Commands
```bash
cargo test      # Expected: all existing tests pass
cargo clippy    # Expected: 0 warnings
cargo fmt --check  # Expected: no formatting issues
grep -r "Borders::ALL" src/ui/  # Expected: 0 matches in content blocks
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] All 6 themes render correctly
- [ ] No Borders::ALL on content areas
- [ ] Component Library used for all new styling
- [ ] Playwright screenshots captured for all views
