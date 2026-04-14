# Draft: Details Panel Feature

## Requirements (from user)
- Right Arrow key should open details panel on the right side
- Panel shows full title, description, and playback options
- Options to show:
  - Open as Video
  - Open as Audio  
  - Quality options
- Show full title and description in panel

## Technical Context
- ActiveBlock enum has: Sidebar, Content, Highlighted
- Tab cycles: Sidebar -> Content -> Highlighted -> Sidebar
- search_results: Vec<Video> stored in App

## Video struct (from invidious.rs - line 48)
- title: String
- description: Option<String>
- author: Option<String>
- view_count, like_count, length_seconds

## Open Questions
- What exact area should the side panel occupy?
- Should it be toggle (Right Arrow = open, Left Arrow = close)?
- What quality options format?

## Scope Boundaries
- INCLUDE: Right Arrow key handling, side panel UI, playback option selection
- EXCLUDE: Actually implementing playback in this plan (was done previously)

## Technical Decisions
- Layout: 3-column split (15% sidebar, 50% list, 35% details)
- Toggle: Right Arrow shows panel, Left Arrow hides
- Key handling in handle_events (line 673+)

## Status
All requirements clear - auto-transitioning to plan generation