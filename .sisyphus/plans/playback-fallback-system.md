# Playback Fallback System

## Objective
Implement a robust fallback mechanism for video playback to ensure videos play even when the primary YouTube URL resolution fails.

## Background
Currently, `youtui-rs` passes the YouTube watch URL directly to `mpv`. If `mpv`'s internal `ytdl` fails, playback stops. We want to implement a sequence of fallback providers (yt-dlp, Invidious, Piped) to fetch direct stream URLs.

## TODOs

### Phase 1: Analysis & Infrastructure
- [x] Analyze `mpv` failure detection: Determine how to detect playback failure (stderr patterns or exit codes) in `src/player/mpv.rs`.
- [x] Define `StreamProvider` trait: Create a trait in `src/api/mod.rs` or a new file for fetching stream URLs.
- [x] Implement `YtdlpProvider`: Wrap `YtdlpWrapper::get_stream_url` in the `StreamProvider` trait.
- [x] Implement `InvidiousProvider`: Add logic to fetch stream URLs from Invidious instances.
- [x] Implement `PipedProvider`: Add logic to fetch stream URLs from Piped instances.

### Phase 2: Integration
- [ ] Update `MpvPlayer::play`: Implement the fallback loop:
    1. Try YouTube Watch URL.
    2. If fail, try `YtdlpProvider`.
    3. If fail, try `InvidiousProvider`.
    4. If fail, try `PipedProvider`.
- [ ] Implement failure detection logic in `MpvPlayer` to trigger the next fallback.
- [ ] Add UI notifications: Send a message to the UI when a fallback is being attempted.

### Phase 3: Verification
- [ ] Verify `yt-dlp` fallback: Force `mpv` to fail on watch URL and verify it uses the `yt-dlp` stream URL.
- [ ] Verify Invidious/Piped fallback: Simulate `yt-dlp` failure and verify it uses alternative providers.
- [ ] Verify UI notifications: Ensure the user is informed about fallback attempts.
- [ ] Regression test: Ensure normal playback still works efficiently.

## Final Verification Wave
- [ ] F1: All fallback providers are tried in sequence upon failure.
- [ ] F2: Direct stream URLs are correctly passed to `mpv`.
- [ ] F3: UI correctly notifies the user of fallback attempts.
- [ ] F4: No regressions in standard playback performance.

---

## Feature: Player Instance Mode (Single vs Multiple)

### Objective
Allow users to choose between using a single mpv instance or multiple instances for playback.

### TODOs
- [ ] Add `player_instance_mode` setting to `Settings`: enum { Single, Multiple }
- [ ] Update settings UI to show player instance mode toggle
- [ ] Implement single-instance logic: if mpv is playing, stop current and start new
- [ ] Implement multi-instance logic: allow spawning multiple mpv processes simultaneously
- [ ] Persist setting to config file

---

## Feature: Video Queue System

### Objective
Add a queue panel showing upcoming videos and the ability to add/remove from queue.

### TODOs
- [ ] Add `QueueItem` struct: { video_id, title, channel, added_at }
- [ ] Add `queue: Vec<QueueItem>` to `App` struct
- [ ] Add `queue_state: ListState` to `App` struct
- [ ] Add Queue mode to `AppMode` enum
- [ ] Add "Queue" to sidebar items
- [ ] Implement `add_to_queue(video)` method
- [ ] Implement `remove_from_queue(index)` method
- [ ] Implement `clear_queue()` method
- [ ] Implement `play_next_from_queue()` method (called when current video ends)
- [ ] Add queue rendering in `render()` function
- [ ] Add keyboard shortcuts: `q` to view queue, `Shift+Q` to add current to queue, `x` to remove from queue
- [ ] Add UI notifications when videos are added/removed from queue
