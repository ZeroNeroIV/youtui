Added Left/Right arrow key handling in AppMode::Main to switch between Sidebar and Content blocks in src/ui/app.rs
Updated render_sidebar in src/ui/components.rs to support a collapsed state based on is_focused. When not focused, the title is hidden, item height is reduced to 2, and only icons are rendered. Active items use theme.accent for the icon and maintain the left accent border.
