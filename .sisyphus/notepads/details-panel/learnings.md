Added details_area: Rect and is_details_open: bool to App struct and initialized them in App::new().
Added KeyCode::Right handling in handle_events to set is_details_open = true, allowing the details panel to be opened via the right arrow key.
Added KeyCode::Left handling in handle_events to set is_details_open = false, allowing the details panel to be closed via the left arrow key.
Updated render function in src/ui/app.rs to support a 3-column layout (15%, 50%, 35%) when is_details_open is true, and a 2-column layout (20%, 80%) otherwise.
Implemented render_details_panel to display video metadata (title, author, views, description) with wrapping. Updated render and render_search to support 3-column layout when details are open. Implemented focused pane UX using theme.accent for active borders and '>> ' as highlight symbol.
