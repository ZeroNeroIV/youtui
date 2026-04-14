Added 'Highlighted' variant to 'ActiveBlock' enum in src/ui/app.rs
Fixed scope creep: reverted src/api/ytdlp.rs and src/player/mpv.rs, and removed unrequested list_state.select(Some(0)) from src/ui/app.rs while keeping Highlighted variant in ActiveBlock.
Implemented KeyCode::Tab handling to cycle active_block: Sidebar -> Content -> Highlighted -> Sidebar in src/ui/app.rs
Added highlighted_area: Rect to App struct and updated handle_mouse_event to support ActiveBlock::Highlighted when clicking in that area.
Updated SearchResponse::Success in src/ui/app.rs to call list_state.select(Some(0)) when results are not empty, ensuring the first result is selected by default.
