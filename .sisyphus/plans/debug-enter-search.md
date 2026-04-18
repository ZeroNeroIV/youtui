# Debug: Enter Key Not Playing Search Video

## Problem
User presses Enter on search result but nothing happens.

## Investigation
The code logic looks correct (lines 1023-1033):
```rust
KeyCode::Enter => {
    if !self.search_results.is_empty() {
        if let Some(idx) = self.list_state.selected() {
            if let Some(video) = self.search_results.get(idx) {
                self.play_search_video(video);
            }
        }
    } else if !self.search_query.is_empty() {
        self.trigger_search();
    }
}
```

## Possible Causes
1. `search_results` is empty when user expects results
2. `list_state.selected()` returns None
3. Video playback silently fails

## Solution: Add Debug Logging

Add print statements to trace the Enter key flow:

```rust
KeyCode::Enter => {
    eprintln!("DEBUG: Enter pressed in Search mode");
    eprintln!("DEBUG: search_results.len() = {}", self.search_results.len());
    eprintln!("DEBUG: list_state.selected() = {:?}", self.list_state.selected());
    
    if !self.search_results.is_empty() {
        if let Some(idx) = self.list_state.selected() {
            eprintln!("DEBUG: selected idx = {}", idx);
            if let Some(video) = self.search_results.get(idx) {
                eprintln!("DEBUG: calling play_search_video for '{}'", video.title);
                self.play_search_video(video);
            } else {
                eprintln!("DEBUG: video not found at idx");
            }
        } else {
            eprintln!("DEBUG: no index selected");
        }
    } else if !self.search_query.is_empty() {
        eprintln!("DEBUG: triggering search");
        self.trigger_search();
    } else {
        eprintln!("DEBUG: nothing to do - empty query and empty results");
    }
}
```

Also add logging to `play_search_video`:
```rust
fn play_search_video(&self, video: &crate::api::invidious::Video) {
    eprintln!("DEBUG: play_search_video called for '{}'", video.title);
    let player = self.player.clone();
    // ... rest
}
```

## Implementation Steps
1. Add debug print statements to Enter handler in Search mode
2. Add debug print statements to play_search_video function
3. Build and run
4. Check terminal output when pressing Enter

## Expected Output
```
DEBUG: Enter pressed in Search mode
DEBUG: search_results.len() = 10
DEBUG: list_state.selected() = Some(0)
DEBUG: selected idx = 0
DEBUG: calling play_search_video for 'Video Title'
DEBUG: play_search_video called for 'Video Title'
```

## Success Criteria
Debug output appears in terminal, revealing where the flow breaks.
