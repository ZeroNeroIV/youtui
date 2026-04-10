# Learnings - youtui-rs

## Project Context
- YouTube TUI in Rust with ratatui
- Hybrid API: Invidious/Piped + yt-dlp fallback
- SQLite persistence for history, saved, playlists
- 6 themes, full mouse support
- Cross-platform (Linux-primary)

## Key Decisions
- Uses tracing for logging with file rotation
- XDG base directories for config/data
- spawn_blocking for SQLite from async context

## Conventions
- snake_case for function/variable names
- Conventional commits (feat:, fix:, etc.)
- Error handling via thiserror + anyhow

## Gotchas
- ratatui mouse support requires terminal capability detection
- Invidious/Piped instances can be unreliable - need fallback
- yt-dlp parsing can be fragile - pin versions

## Task 3: Config module (XDG dirs, settings)
- Settings struct with all required fields (quality, format, path, player, API instances, theme, auto_play)
- Uses dirs crate for XDG base directories
- Config stored at ~/.config/youtui-rs/config.json
- load() returns defaults if config doesn't exist
- save() creates directory if needed
## Task 2: B_DEBUG_FLAG + logging setup
- Uses tracing + tracing-appender with daily rotation
- XDG location: dirs::data_local_dir() -> youtui-rs/logs/
- LOG_TO_FILE and LOG_TO_TERMINAL via env vars
- No terminal output in release mode
Implemented mouse support in src/ui/app.rs:
- Enabled/Disabled mouse capture in App::run.
- Implemented mouse click selection for sidebar and content lists using stored Rects.
- Implemented mouse scroll navigation for lists.
- Implemented right-click context menu and left-click interaction to close/select options.
Implemented settings panel using a mode-based rendering approach (AppMode). Integrated with existing config module for persistence and theme system for immediate visual updates.
Task 24: Edge case handling and error messages:
- Created ErrorCategory enum in src/ui/components.rs for categorizing errors
- Implemented user-friendly messages for each error type (network, data, playback, input)
- Added render_error(), render_empty_state(), render_loading() UI components
- Added error state fields to App struct (current_error, current_suggestion, is_loading, loading_message)
- Added helper methods to App: set_error(), set_error_with_suggestion(), clear_error(), set_loading(), clear_loading(), is_empty(), current_view()
- Implemented render_error_overlay(), render_empty_state(), render_loading_overlay() for displaying states
- Error overlay takes priority over loading, which takes priority over empty state
- Removed comments from app.rs that were flagged by lint
Task 25: Startup behavior:
- Added APP_VERSION constant to config/settings.rs (uses env! macro to get cargo package version)
- Added AppState struct to persist last sidebar index, content index, and settings index across sessions
- AppState stored in ~/.config/youtui-rs/state.json
- On startup: check mpv availability (async), check API instance health (with 3s timeout)
- Startup warnings printed to stderr if mpv or API unavailable (non-blocking)
- Added startup banner showing version
- On exit: save current sidebar, content, and settings positions to state file
- On startup: restore previous positions from saved state

## Code Quality Review (F2) - 2026-04-10
- **Formatting**: Ran `cargo fmt` to ensure consistent style across the project.
- **Clippy Fixes**:
    - `src/download/downloader.rs`: Replaced `tx.send()` with `tx.blocking_send()` in synchronous threads to avoid unused future warnings.
    - `src/player/mpv.rs`: Simplified `match child.id() { Some(_) => true, None => false }` to `child.id().is_some()`.
    - `src/ui/settings.rs`: Replaced `vec![...]` with arrays `[...]` for static configuration lists.
- **Bug Fixes**:
    - `src/api/piped.rs`: Fixed `api_url` to include a slash between `base_url` and `endpoint`, resolving a failing test in `test_api_url_building`.
- **Verification**: All `cargo clippy`, `cargo fmt --check`, and `cargo test` checks now pass.
## Integration Testing Report - Fri Apr 10 03:52:14 PM +03 2026

### Verified Scenarios
- **App Launch**: Verified. App starts correctly.
- **Basic Navigation**: Verified. Sidebar and Settings navigation work as expected.
- **Theme Switching**: Verified. Themes cycle correctly and update the UI.
- **Persistence**: Verified. Settings (Theme, Auto Play) are persisted across restarts.

### Missing Implementations (Critical)
The following requested integration scenarios could not be tested as they are not yet implemented in the UI:
- **Search**: No input handling for search terms.
- **Playback**: No trigger to start video playback from the UI.
- **History**: No logic to view or manage watch history.
- **Playlists**: No logic to create or manage playlists.

The current implementation is a skeleton with hardcoded video lists and basic settings management.
Integrated Saved Videos feature into TUI. Added AppMode::Saved, updated App state, implemented render_saved and event handling for navigation, playback, and unsaving. Verified with cargo build.
Fixed playback trigger in Search mode:
- Implemented play_search_video helper.
- Added Up/Down navigation for search results.
- Enter now plays selected video if results are present, otherwise triggers search.
- Typing (Char/Backspace) now clears search results to ensure Enter triggers a new search.
- Fixed several pre-existing syntax and borrow checker errors in src/ui/app.rs to allow successful build.
Implemented Playlist UI handlers in src/ui/app.rs. Updated App.db to Arc<Database> to allow sharing the database connection across tokio tasks for async API calls (import/refresh).
Implemented Download UI integration in src/ui/app.rs. Added download channels to App struct and implemented asynchronous download triggering with UI feedback.
### Wire up add_to_history in App playback methods
- Updated `play_history_video`, `play_saved_video`, `play_search_video`, and `play_playlist_video` in `src/ui/app.rs`.
- Each method now clones `self.db` and the video metadata (ID, title, channel) and calls `db.add_to_history` within a `tokio::spawn` block to avoid blocking the UI thread.
- Verified with `lsp_diagnostics` and `cargo check`.
Implemented async 'Save' logic in context menu using a dedicated mpsc channel (saved_tx/rx) to notify the UI loop to refresh saved results and display feedback.
Added 'd' keyboard shortcut for downloading videos across Search, History, Saved, and Playlist modes. In Playlist mode, 'd' was previously used for removing videos, so that functionality was moved to 'x' and the UI was updated accordingly.
