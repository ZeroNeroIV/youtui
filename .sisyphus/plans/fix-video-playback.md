# Fix: Video Playback Not Working

## Problem
User presses Enter on any video but nothing plays. No errors visible.

## Investigation
- mpv IS installed at `/usr/bin/mpv`
- Logging goes to `~/.local/share/youtui-rs/logs/` but logs appear empty
- The `play_search_video` function silently ignores errors with `let _ = player.play()...`

## Root Cause
The `tokio::spawn` in `play_search_video` runs async and errors are silently discarded:
```rust
tokio::spawn(async move {
    let _ = db.add_to_history(&video_id, &title, channel.as_deref());
    let _ = player.play(&url, &quality, &[]).await;  // Error ignored!
});
```

## Solution: Show Error When Playback Fails

Modify `play_search_video` to show an error notification when playback fails:

```rust
fn play_search_video(&mut self, video: &crate::api::invidious::Video) {
    let player = self.player.clone();
    let db = self.db.clone();
    let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
    let video_id = video.video_id.clone();
    let title = video.title.clone();
    let channel = video.author.clone();
    let quality = self.settings.default_quality.clone();
    
    // Spawn async task
    tokio::spawn(async move {
        let _ = db.add_to_history(&video_id, &title, channel.as_deref());
        if let Err(e) = player.play(&url, &quality, &[]).await {
            eprintln!("Playback error: {}", e);
        }
    });
}
```

**Changes needed:**
1. Add `&mut self` to `play_search_video` function signature
2. Change `let _ =` to `if let Err(e) =` and print error
3. Update the call site in event handler (line 1028) - may need adjustment

Actually, simpler fix - just add error printing without changing signature:

```rust
fn play_search_video(&self, video: &crate::api::invidious::Video) {
    let player = self.player.clone();
    let db = self.db.clone();
    let url = format!("https://www.youtube.com/watch?v={}", video.video_id);
    let video_id = video.video_id.clone();
    let title = video.title.clone();
    let channel = video.author.clone();
    let quality = self.settings.default_quality.clone();
    tokio::spawn(async move {
        let _ = db.add_to_history(&video_id, &title, channel.as_deref());
        match player.play(&url, &quality, &[]).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error playing video '{}': {}", title, e);
            }
        }
    });
}
```

## Implementation Steps
1. Update `play_search_video` in `src/ui/app.rs` - change `let _ = player.play()` to `match` with `eprintln!` on Err
2. Update `play_main_video` similarly
3. Update `play_history_video` similarly  
4. Update `play_saved_video` similarly
5. Update `play_playlist_video` similarly
6. Build with `cargo build`

## QA Scenarios

### QA-1: Build Verification
- **Tool**: `cargo build`
- **Steps**: Run `cargo build --release` in project root
- **Expected**: Exit code 0, no compilation errors

### QA-2: Error Visibility Test (using stderr capture)
- **Tool**: `bash` + run app with stderr capture
- **Steps**: 
  1. `cd /home/zeroneroiv/projects/personal/youtui`
  2. Rename mpv to break it: `sudo mv /usr/bin/mpv /usr/bin/mpv.bak`
  3. Run app in background: `cargo run 2>&1 | tee /tmp/youtui_err.log`
  4. Navigate to a video and press Enter
  5. Check if error message appears in terminal output (should see "Error playing video" if fix is working)
  6. Restore mpv: `sudo mv /usr/bin/mpv.bak /usr/bin/mpv`
- **Expected**: User sees error like "Error playing video 'Video Title': mpv not found" in stderr output
- **Restore**: `sudo mv /usr/bin/mpv.bak /usr/bin/mpv` (CRITICAL - must restore)

### QA-3: Verify Normal Playback (after mpv restore)
- **Tool**: `bash`
- **Steps**: `cargo run` and play a video normally
- **Expected**: Video plays without any error messages (only success path runs)

## Success Criteria
- [ ] QA-1 passes (build succeeds)
- [ ] QA-2 passes (error messages visible when mpv unavailable)  
- [ ] QA-3 passes (normal playback works, no spurious errors)
