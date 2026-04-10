# youtui-rs: YouTube TUI

## TL;DR

> **Quick Summary**: A feature-rich YouTube terminal client in Rust built with ratatui, featuring hybrid API (Invidious/Piped + yt-dlp), full mouse support, 6 themes, and SQLite persistence for watch history, saved videos, and playlists.
> 
> **Deliverables**:
> - Rust CLI application (`youtui-rs`)
> - Search, playback, history, playlists, downloads
> - 6 themed color schemes
> - Full mouse navigation
> - Cross-platform (Linux-primary, macOS/Windows/BSD)
> 
> **Estimated Effort**: XL (large project)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Project scaffold → API client → Search → Playback → Playlists → Downloads → Final polish

---

## Context

### Original Request
Build a YouTube TUI combining all features from existing tools (ytcui, yewtube, youtube-tui) plus full mouse input support.

### Interview Summary
**Key Discussions**:
- Tech Stack: Rust + Ratatui + tokio + rusqlite (async via spawn_blocking)
- API: Hybrid (Invidious/Piped primary + yt-dlp fallback)
- Playback: Both mpv (external) and native option
- Database: SQLite for History, Saved, Playlists, Settings
- UI: Sidebar navigation, 6 themes, full mouse support
- Features: Search, Playback, History, Saved, Playlists (local+YT), Downloads, Details, Quality
- Excluded: Library/Subscriptions, Channel Browser, Comments, Thumbnails

### Metis Review
**Identified Gaps** (addressed in this plan):
- MVP definition: Added explicit MVP scope
- Scope creep lockdown: Added "NOT building" section
- TDD acceptance criteria: Added for each feature
- Edge cases: Network, data, playback, interaction covered
- Terminal compatibility: Mouse detection + fallback
- Database schema: Migration strategy included

---

## Work Objectives

### Core Objective
Build a production-ready YouTube TUI that combines the best features of existing tools with a superior UI/UX experience.

### Concrete Deliverables
- `youtui-rs` binary (cargo build)
- SQLite database at XDG location
- 6 color themes (terminal, tokyo, monokai, light, dark, retro)
- ratatui-based UI with sidebar navigation

### Definition of Done
- [ ] `cargo build --release` succeeds
- [ ] Search returns results from Invidious/Piped
- [ ] Playback works via mpv
- [ ] History persists across restarts
- [ ] All 6 themes render correctly
- [ ] Mouse navigation functional
- [ ] Tests pass: `cargo test`

### Must Have
- Search and play YouTube videos
- Watch history with persistence
- Save/bookmark videos
- Local playlists + YouTube playlist sync
- Download to disk
- 6 themes with rich visual design
- Full mouse support (click, scroll, context menus)
- Configurable via CLI/settings

### Must NOT Have (Guardrails)
- Library/Subscriptions (explicitly excluded)
- Channel Browser (explicitly excluded)
- Comments (always off by requirement)
- Thumbnails in v1 (least priority)
- Queue manager for downloads (only "download current")
- Collaborative/smart playlists (local + YouTube sync only)
- Keybinding customization (opinionated - fixed shortcuts)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (new project)
- **Automated tests**: YES - Unit tests for core modules
- **Framework**: cargo test (built-in Rust)
- **TDD approach**: Tests first for API layer, models, database

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/`.

- **Search**: Use curl to test API endpoint directly
- **Playback**: Run mpv with test URL, verify it launches
- **Database**: Run SQLite queries to verify data persistence
- **UI**: Run terminal and verify theme rendering
- **Mouse**: Verify click events register

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - Foundation):
├── Task 1: Project scaffold + Cargo.toml dependencies
├── Task 2: B_DEBUG_FLAG + logging setup (tracing)
├── Task 3: Config module (XDG dirs, settings)
├── Task 4: Database module (rusqlite + migrations)
├── Task 5: Error handling (thiserror + anyhow)
└── Task 6: Basic ratatui setup + main loop

Wave 2 (After Wave 1 - API + Core):
├── Task 7: Invidious API client (reqwest)
├── Task 8: Piped API client
├── Task 9: yt-dlp fallback wrapper
├── Task 10: API Health check + instance rotation
├── Task 11: Models (Video, Playlist, SearchResult, etc.)
└── Task 12: Search feature implementation

Wave 3 (After Wave 2 - Features):
├── Task 13: Playback module (mpv integration)
├── Task 14: Watch history feature
├── Task 15: Saved videos feature
├── Task 16: Local playlists feature
├── Task 17: YouTube playlist sync
├── Task 18: Download feature
├── Task 19: Video details + quality selection
└── Task 20: Theme system (6 themes)

Wave 4 (After Wave 3 - UI Polish):
├── Task 21: Sidebar navigation UI
├── Task 22: Mouse support (click, scroll, context menu)
├── Task 23: Settings panel
├── Task 24: Edge case handling + error messages
├── Task 25: Startup behavior (check updates, remember position)
└── Task 26: CI/CD setup (GitHub Actions)

Wave FINAL (After ALL tasks):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (clippy + rustfmt)
├── Task F3: Integration testing
└── Task F4: Scope fidelity check
-> Present results -> Get explicit user okay
```

### Dependency Matrix (abbreviated)
- **1-6**: - - (Wave 1 - no deps)
- **7-12**: 1-6 - 13-20 (Wave 2 - depends on foundation)
- **13-20**: 7-12 - 21-26 (Wave 3 - depends on API layer)
- **21-26**: 13-20 - F1-F4 (Wave 4 - depends on features)

---

## TODOs

- [ ] 1. **Project scaffold + Cargo.toml dependencies**

  **What to do**:
  - Create new Rust project with `cargo new youtui-rs`
  - Add dependencies: ratatui, tokio (full), reqwest, serde_json, rusqlite, tracing, tracing-subscriber, arboard, url, chrono, thiserror, anyhow
  - Create basic project structure (src/main.rs, src/lib.rs)
  - Add .gitignore, README.md skeleton
  - Run `cargo check` to verify dependencies compile

  **Must NOT do**:
  - No test code yet (keep it minimal)
  - No feature implementation

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > - Reason: Project scaffolding is routine setup, no complex logic
  > **Skills**: []
  > - No special skills needed for scaffolding

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 2-6)
  > Blocks: 7-12 (API layer needs structure)
  > Blocked By: None (can start immediately)

  **References**:
  > - ratatui repo: https://github.com/ratatui/ratatui - Check examples for project setup
  > - tokio docs: https://tokio.rs - Use "full" feature set for TUI
  > - rusqlite: https://github.com/rusqlite/rusqlite - Check bundled feature for SQLite

  **Acceptance Criteria**:
  - [ ] `cargo new youtui-rs` creates project
  - [ ] `Cargo.toml` has all dependencies with compatible versions
  - [ ] `cargo check` passes without errors

  **QA Scenarios**:
  ```
  Scenario: Verify project builds
    Tool: Bash
    Preconditions: None
    Steps:
      1. Run cargo build --lib
      2. Verify no compilation errors
    Expected Result: Build succeeds
    Evidence: .sisyphus/evidence/task-1-build.{ext}
  ```

  **Commit**: YES
  - Message: `feat: initial project scaffold`
  - Files: `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `.gitignore`, `README.md`

---

- [ ] 2. **B_DEBUG_FLAG + logging setup (tracing)**

  **What to do**:
  - Create `src/config/mod.rs` with B_DEBUG_FLAG
  - Create `src/utils/logger.rs` using tracing + tracing-subscriber
  - Support ENV variable (development/production)
  - Support LOG_TO_FILE + LOG_TO_TERMINAL
  - Add file rotation for logs (max 5MB, keep 3 files)
  - Create logs directory at XDG location

  **Must NOT do**:
  - No console logging in release mode

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > - Reason: Logging setup is standard boilerplate
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 1, 3-6)
  > Blocks: All subsequent tasks (logging needed everywhere)
  > Blocked By: None

  **References**:
  > - tracing docs: https://docs.rs/tracing
  > - tracing-subscriber: https://docs.rs/tracing-subscriber
  > - AGENTS.md debug flag pattern

  **Acceptance Criteria**:
  - [ ] B_DEBUG_FLAG respects ENV variable
  - [ ] Logs written to file in development mode
  - [ ] LOG_TO_FILE and LOG_TO_TERMINAL work

  **QA Scenarios**:
  ```
  Scenario: Logging writes to file in development
    Tool: Bash
    Preconditions: ENV=development
    Steps:
      1. Run app
      2. Check log file exists
      3. Verify log contains startup message
    Expected Result: Log file created with entries
    Evidence: .sisyphus/evidence/task-2-logging.{ext}
  ```

  **Commit**: NO (group with task 1)

---

- [ ] 3. **Config module (XDG dirs, settings)**

  **What to do**:
  - Create `src/config/settings.rs` for app configuration
  - Use dirs crate for XDG base directories
  - Settings: default quality, default format, download path, player choice, API instances, theme, auto-play
  - Create default config on first run
  - Load config from JSON file

  **Must NOT do**:
  - No encrypted storage (use environment variables for secrets)

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > - Reason: Config management is straightforward
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 1, 2, 4-6)
  > Blocks: 7-12 (API needs config)
  > Blocked By: Task 2 (logging references config paths)

  **References**:
  > - dirs crate: https://crates.io/crates/dirs
  > - XDG spec: https://specifications.freedesktop.org/basedir-spec/

  **Acceptance Criteria**:
  - [ ] Config file created at XDG location
  - [ ] All settings loadable/saveable
  - [ ] Default values applied on first run

  **QA Scenarios**:
  ```
  Scenario: Config persists across restarts
    Tool: Bash
    Preconditions: Config created once
    Steps:
      1. Run app (creates config)
      2. Modify a setting
      3. Run app again
      4. Verify setting persisted
    Expected Result: Setting preserved
    Evidence: .sisyphus/evidence/task-3-config.{ext}
  ```

  **Commit**: NO (group with task 1)

---

- [ ] 4. **Database module (rusqlite + migrations)**

  **What to do**:
  - Create `src/db/mod.rs` with SQLite connection
  - Create schema: watch_history, saved_videos, playlists, playlist_videos, settings
  - Implement migrations (version tracking)
  - Use `tokio::spawn_blocking` for async DB operations
  - Create DAO methods: insert_video, get_history, add_to_playlist, etc.

  **Must NOT do**:
  - No external DB (only SQLite)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > - Reason: Database schema design + async wrapping requires attention
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 1-3, 5-6)
  > Blocks: 14-17 (History, Saved, Playlists need DB)
  > Blocked By: Task 3 (config provides DB path)

  **References**:
  > - rusqlite docs: https://docs.rs/rusqlite
  > - SQLite schema best practices

  **Acceptance Criteria**:
  - [ ] Database created at XDG location
  - [ ] Migrations run on startup
  - [ ] Basic CRUD operations work

  **QA Scenarios**:
  ```
  Scenario: Database operations work
    Tool: Bash
    Preconditions: None
    Steps:
      1. Run app (creates DB)
      2. Insert test video into history
      3. Query history
      4. Verify video present
    Expected Result: Data persisted
    Evidence: .sisyphus/evidence/task-4-db.{ext}
  ```

  **Commit**: NO (group with task 1)

---

- [ ] 5. **Error handling (thiserror + anyhow)**

  **What to do**:
  - Create `src/error.rs` with custom error types
  - Use thiserror for typed errors (ApiError, DbError, ConfigError, PlaybackError)
  - Use anyhow for application-level error handling
  - Implement Display for all error types
  - Add error context for debugging

  **Must NOT do**:
  - No panics in production code

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > - Reason: Error handling pattern is standard
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 1-4, 6)
  > Blocks: All tasks (error handling everywhere)
  > Blocked By: None

  **References**:
  > - thiserror: https://docs.rs/thiserror
  > - anyhow: https://docs.rs/anyhow

  **Acceptance Criteria**:
  - [ ] Custom error types cover all error sources
  - [ ] Errors display nicely in UI

  **Commit**: NO (group with task 1)

---

- [ ] 6. **Basic ratatui setup + main loop**

  **What to do**:
  - Create `src/ui/mod.rs` with basic TUI setup
  - Initialize ratatui terminal
  - Create basic event loop (input polling)
  - Create empty screen structure (sidebar + content area)
  - Add crossterm for input handling

  **Must NOT do**:
  - No complex widgets yet

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > - Reason: TUI framework setup requires UI knowledge
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 1)
  > Parallel Group: Wave 1 (with Tasks 1-5)
  > Blocks: 20-23 (UI features need base)
  > Blocked By: Task 1 (dependencies needed)

  **References**:
  > - ratatui getting started: https://ratatui.rs/
  > - crossterm: https://docs.rs/crossterm

  **Acceptance Criteria**:
  - [ ] App launches and shows terminal UI
  - [ ] Basic event loop runs
  - [ ] Clean shutdown on Ctrl+C

  **QA Scenarios**:
  ```
  Scenario: Terminal UI launches
    Tool: interactive_bash
    Preconditions: None
    Steps:
      1. Run app in terminal
      2. Verify TUI renders
      3. Press Ctrl+C to exit
    Expected Result: Clean render and exit
    Evidence: .sisyphus/evidence/task-6-tui.{ext}
  ```

  **Commit**: NO (group with task 1)

---

- [ ] 7. **Invidious API client (reqwest)**

  **What to do**:
  - Create `src/api/invidious.rs` client
  - Implement search endpoint (invidious.com/api/v1/search)
  - Implement video details endpoint
  - Implement trending/popular videos
  - Handle JSON parsing with serde_json
  - Add timeout and retry logic

  **Must NOT do**:
  - No API key required for Invidious

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > - Reason: API client with error handling requires care
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 8-11)
  > Blocks: 12 (Search needs client)
  > Blocked By: 1-6 (Foundation)

  **References**:
  > - Invidious API: https://docs.invidious.io/API.md
  > - reqwest: https://docs.rs/reqwest

  **Acceptance Criteria**:
  - [ ] Search returns video results
  - [ ] Video details retrievable
  - [ ] Error handling for network failures

  **QA Scenarios**:
  ```
  Scenario: Invidious search works
    Tool: Bash
    Preconditions: Network available
    Steps:
      1. Call Invidious search API
      2. Verify JSON response
      3. Parse into Video struct
    Expected Result: Results returned
    Evidence: .sisyphus/evidence/task-7-invidious.{ext}
  ```

  **Commit**: YES
  - Message: `feat: add Invidious API client`
  - Files: `src/api/invidious.rs`, `src/api/mod.rs`

---

- [ ] 8. **Piped API client**

  **What to do**:
  - Create `src/api/piped.rs` client
  - Implement search, video details, streams
  - Use Piped API (pipedapi.kavin.rocks or similar)
  - Handle different response format from Invidious
  - Standardize to common Video type

  **Must NOT do**:
  - No duplicate code - share models with Invidious

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 7, 9-12)
  > Blocks: 10 (Health check needs both clients)
  > Blocked By: 1-6

  **References**:
  > - Piped API: https://piped-docs.kavin.rocks/

  **Acceptance Criteria**:
  - [ ] Search returns results
  - [ ] Video streams available

  **Commit**: NO (group with task 7)

---

- [ ] 9. **yt-dlp fallback wrapper**

  **What to do**:
  - Create `src/api/ytdlp.rs` wrapper
  - Check if yt-dlp is installed
  - Parse output for video info
  - Handle format extraction
  - Fallback when API fails

  **Must NOT do**:
  - Don't use yt-dlp as primary (only fallback)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 7-8, 10-12)
  > Blocks: 10 (Fallback part of health check)
  > Blocked By: 1-6

  **References**:
  > - yt-dlp: https://github.com/yt-dlp/yt-dlp

  **Commit**: NO (group with task 7)

---

- [ ] 10. **API Health check + instance rotation**

  **What to do**:
  - Create `src/api/health.rs` 
  - Check instance availability before use
  - Implement instance list for Invidious/Piped
  - Rotate through instances on failure
  - User can configure preferred instances

  **Must NOT do**:
  - No hardcoded single instance

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 7-9, 11-12)
  > Blocks: 12 (Search uses health check)
  > Blocked By: 7-9 (Clients needed)

  **Commit**: NO (group with task 7)

---

- [ ] 11. **Models (Video, Playlist, SearchResult, etc.)**

  **What to do**:
  - Create `src/models/mod.rs`
  - Define Video struct with id, title, channel, duration, thumbnail
  - Define Playlist struct
  - Define SearchResult struct
  - Define StreamURL struct
  - Use serde for serialization

  **Must NOT do**:
  - No business logic in models

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 7-10, 12)
  > Blocks: 7-10 (Models used by clients)
  > Blocked By: 1-6

  **Commit**: NO (group with task 7)

---

- [ ] 12. **Search feature implementation**

  **What to do**:
  - Create search UI (input field + results list)
  - Connect to API client
  - Display results with title, channel, duration
  - Handle pagination (load more)
  - Add keyboard navigation

  **Must NOT do**:
  - No advanced filters in v1

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 2)
  > Parallel Group: Wave 2 (with Tasks 7-11)
  > Blocks: 13 (Playback uses search result)
  > Blocked By: 7-11

  **Acceptance Criteria**:
  - [ ] Search returns results in < 5s
  - [ ] Results show title, channel, duration
  - [ ] Arrow keys navigate results
  - [ ] Enter plays selected video

  **QA Scenarios**:
  ```
  Scenario: Search for "rust tutorial"
    Tool: interactive_bash
    Preconditions: Network available
    Steps:
      1. Type "rust tutorial" in search
      2. Press Enter
      3. Verify results appear
    Expected Result: Results displayed
    Evidence: .sisyphus/evidence/task-12-search.{ext}
  ```

  **Commit**: YES
  - Message: `feat: add search feature`
  - Files: `src/ui/search.rs`, update `src/ui/mod.rs`

---

- [ ] 13. **Playback module (mpv integration)**

  **What to do**:
  - Create `src/player/mod.rs` for playback
  - Integrate mpv via tokio::process
  - Support video and audio-only modes
  - Handle playback controls (play, pause, stop, seek)
  - Track playback state

  **Must NOT do**:
  - No native video rendering (use mpv or external player)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 14-20)
  > Blocks: 14 (History needs playback)
  > Blocked By: 12 (Search provides video to play)

  **References**:
  > - mpv manual: https://mpv.io/manual/

  **Acceptance Criteria**:
  - [ ] mpv launches with video URL
  - [ ] Audio-only mode works
  - [ ] Playback can be stopped

  **QA Scenarios**:
  ```
  Scenario: Play video via mpv
    Tool: interactive_bash
    Preconditions: mpv installed
    Steps:
      1. Select video from search
      2. Press Enter to play
      3. Verify mpv launches
    Expected Result: Video plays
    Evidence: .sisyphus/evidence/task-13-playback.{ext}
  ```

  **Commit**: YES
  - Message: `feat: add playback with mpv`
  - Files: `src/player/mod.rs`, `src/player/mpv.rs`

---

- [ ] 14. **Watch history feature**

  **What to do**:
  - Create `src/ui/history.rs` screen
  - Connect to database for history queries
  - Display reverse chronological list
  - Allow clearing history
  - Add to history on playback start

  **Must NOT do**:
  - No auto-add on failed playback

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13, 15-20)
  > Blocks: None (depends on 13)
  > Blocked By: 4 (DB), 13 (Playback)

  **Acceptance Criteria**:
  - [ ] History displays past watched videos
  - [ ] Clicking video replays it
  - [ ] History persists after restart

  **Commit**: NO (group with commit 7)

---

- [ ] 15. **Saved videos feature**

  **What to do**:
  - Create `src/ui/saved.rs` screen
  - Implement save/unsave video (bookmark)
  - Persist saved videos to database
  - Keyboard shortcut 's' to save

  **Must NOT do**:
  - No auto-save

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-14, 16-20)
  > Blocks: None
  > Blocked By: 4 (DB), 13 (Playback)

  **Commit**: NO (group with commit 7)

---

- [ ] 16. **Local playlists feature**

  **What to do**:
  - Create `src/ui/playlists.rs` screen
  - Create, rename, delete playlists
  - Add/remove videos from playlists
  - Reorder videos within playlist
  - Persist to database

  **Must NOT do**:
  - No collaborative playlists

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-15, 17-20)
  > Blocks: None
  > Blocked By: 4 (DB)

  **Commit**: NO (group with commit 9)

---

- [ ] 17. **YouTube playlist sync**

  **What to do**:
  - Import YouTube playlists via URL
  - Sync playlist items
  - Refresh on demand
  - Handle playlist edge cases (private, deleted)

  **Must NOT do**:
  - No real-time sync (manual only)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-16, 18-20)
  > Blocks: None
  > Blocked By: 7-8 (API clients)

  **Commit**: NO (group with commit 9)

---

- [ ] 18. **Download feature**

  **What to do**:
  - Create download function using yt-dlp
  - Show progress in UI
  - Allow quality/format selection
  - Save to configured download directory

  **Must NOT do**:
  - No queue manager or scheduling

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-17, 19-20)
  > Blocks: None
  > Blocked By: 9 (yt-dlp wrapper)

  **Acceptance Criteria**:
  - [ ] Video downloads to specified location
  - [ ] Progress shown during download

  **Commit**: YES
  - Message: `feat: add download feature`
  - Files: `src/download/mod.rs`, update `src/ui/`

---

- [ ] 19. **Video details + quality selection**

  **What to do**:
  - Create video details panel
  - Display title, channel, description, date
  - Show available quality options
  - Let user select before playback/download

  **Must NOT do**:
  - No chapters or transcript

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-18, 20)
  > Blocks: None
  > Blocked By: 7-8 (API for details)

  **Commit**: NO (group with commit 9)

---

- [ ] 20. **Theme system (6 themes)**

  **What to do**:
  - Create `src/ui/theme.rs` with 6 themes
  - Themes: terminal, tokyo, monokai, light, dark, retro
  - Implement theme switching
  - Apply themes to all UI components
  - Use coolors.co palettes

  **Must NOT do**:
  - No custom theme creation in v1

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 3)
  > Parallel Group: Wave 3 (with Tasks 13-19)
  > Blocks: 21 (Sidebar uses themes)
  > Blocked By: 6 (TUI base)

  **Acceptance Criteria**:
  - [ ] All 6 themes render correctly
  - [ ] Theme can be changed in settings
  - [ ] Colors apply to all components

  **QA Scenarios**:
  ```
  Scenario: Theme switching works
    Tool: interactive_bash
    Preconditions: App running
    Steps:
      1. Open Settings
      2. Change theme
      3. Verify colors change
    Expected Result: Theme applied
    Evidence: .sisyphus/evidence/task-20-theme.{ext}
  ```

  **Commit**: YES
  - Message: `feat: add theme system`
  - Files: `src/ui/theme.rs`

---

- [ ] 21. **Sidebar navigation UI**

  **What to do**:
  - Create sidebar with menu items
  - Items: Search, History, Saved, Playlists, Downloads, Settings
  - Highlight active section
  - Keyboard navigation (Tab, arrows)
  - Mouse click to switch sections

  **Must NOT do**:
  - No collapsible sidebar

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 22-26)
  > Blocks: 22 (Mouse needs sidebar)
  > Blocked By: 6 (TUI base), 20 (Theme)

  **Commit**: YES
  - Message: `feat: add sidebar navigation`
  - Files: `src/ui/sidebar.rs`

---

- [ ] 22. **Mouse support (click, scroll, context menu)**

  **What to do**:
  - Enable mouse events in ratatui
  - Handle click for selection
  - Handle scroll for navigation
  - Implement right-click context menus
  - Fallback to keyboard if terminal doesn't support

  **Must NOT do**:
  - No drag-and-drop in v1

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 21, 23-26)
  > Blocks: None
  > Blocked By: 6 (TUI base)

  **Acceptance Criteria**:
  - [ ] Click selects items
  - [ ] Scroll navigates lists
  - [ ] Right-click shows context menu

  **QA Scenarios**:
  ```
  Scenario: Mouse navigation works
    Tool: interactive_bash
    Preconditions: Terminal with mouse support
    Steps:
      1. Click on video in list
      2. Verify selection
      3. Scroll through list
    Expected Result: Mouse works
    Evidence: .sisyphus/evidence/task-22-mouse.{ext}
  ```

  **Commit**: YES
  - Message: `feat: add mouse support`
  - Files: Update input handling in `src/ui/`

---

- [ ] 23. **Settings panel**

  **What to do**:
  - Create settings UI
  - Configurable: quality, format, download path, player, API instances, theme
  - Save settings to config file
  - Changes apply immediately

  **Must NOT do**:
  - No keybinding customization

  **Recommended Agent Profile**:
  > **Category**: `visual-engineering`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 21-22, 24-26)
  > Blocks: None
  > Blocked By: 3 (Config module)

  **Commit**: NO (group with commit 11)

---

- [ ] 24. **Edge case handling + error messages**

  **What to do**:
  - Handle network errors gracefully
  - Handle API failures with fallback
  - Handle playback errors
  - Handle invalid input
  - Show user-friendly error messages in UI
  - Log errors for debugging

  **Must NOT do**:
  - No stack traces in UI

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 21-23, 25-26)
  > Blocks: None
  > Blocked By: 5 (Error handling), 7-10 (API)

  **Commit**: YES
  - Message: `fix: edge cases and errors`
  - Files: Update error handling across modules

---

- [ ] 25. **Startup behavior (check updates, remember position)**

  **What to do**:
  - Check for app updates on startup (optional, can skip)
  - Remember last active screen
  - Remember last scroll position
  - Load cached data on startup

  **Must NOT do**:
  - No auto-update installation

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > **Skills**: []

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 21-24, 26)
  > Blocks: None
  > Blocked By: 3 (Config), 4 (DB)

  **Commit**: NO (group with commit 13)

---

- [ ] 26. **CI/CD setup (GitHub Actions)**

  **What to do**:
  - Create `.github/workflows/ci.yml`
  - Run: cargo fmt, cargo clippy, cargo test
  - Run: cargo build (release)
  - Add security audit (cargo-audit)
  - Add status badges to README

  **Must NOT do**:
  - No deployment (not needed for CLI)

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > **Skills**: [git-master]

  **Parallelization**:
  > Can Run In Parallel: YES (Wave 4)
  > Parallel Group: Wave 4 (with Tasks 21-25)
  > Blocks: F1-F4 (Final verification needs CI)
  > Blocked By: All tasks

  **Acceptance Criteria**:
  - [ ] CI workflow runs on push
  - [ ] All checks pass
  - [ ] Badge visible in README

  **QA Scenarios**:
  ```
  Scenario: CI passes
    Tool: Bash
    Preconditions: Push to main
    Steps:
      1. Check GitHub Actions run
      2. Verify all jobs pass
    Expected Result: CI green
    Evidence: .sisyphus/evidence/task-26-ci.{ext}
  ```

  **Commit**: YES
  - Message: `ci: add GitHub Actions`
  - Files: `.github/workflows/ci.yml`

---

## Final Verification Wave (MANDATORY)

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify all "Must Have" implemented, "Must NOT Have" absent, evidence files exist.

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy`, `cargo fmt --check`, `cargo test`.

- [ ] F3. **Integration Testing** — `unspecified-high`
  Run search, playback, history, playlist scenarios end-to-end.

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no scope creep, no missing features from spec.

---

## Commit Strategy

- **1**: `feat: initial project scaffold` - Cargo.toml, main.rs, basic structure
- **2**: `feat: add logging and config` - tracing, B_DEBUG_FLAG, XDG config
- **3**: `feat: setup database with migrations` - rusqlite, schema
- **4**: `feat: implement API clients` - Invidious, Piped, yt-dlp fallback
- **5**: `feat: add search feature` - Search UI + API integration
- **6**: `feat: add playback with mpv` - Playback module + controls
- **7**: `feat: add watch history` - History persistence
- **8**: `feat: add saved videos` - Bookmark functionality
- **9**: `feat: add playlists` - Local + YouTube sync
- **10**: `feat: add download feature` - yt-dlp download
- **11**: `feat: add theme system` - 6 themes
- **12**: `feat: add mouse support` - Full mouse navigation
- **13**: `feat: add settings panel` - Configuration UI
- **14**: `fix: edge cases and errors` - Error handling
- **15**: `ci: add GitHub Actions` - CI/CD pipeline

---

## Success Criteria

### Verification Commands
```bash
cargo build --release  # Builds without errors
cargo test             # All tests pass
cargo clippy           # No warnings
cargo fmt --check      # Properly formatted
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] 6 themes render correctly
- [ ] Mouse support works
- [ ] Database persists data
- [ ] API fallback works
- [ ] Tests pass