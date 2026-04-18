# Fix: Search Input Visibility

## Problem
**User reports**: Header visible, but input area is blank/invisible.

The Paragraph widget at `chunks[1]` is not rendering the search input.

## Current Code (lines 838-848)
```rust
let display_text = format!("{}_", self.search_query);
let input = Paragraph::new(display_text)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.theme.accent)
            .padding(Padding::uniform(components::DesignTokens::PADDING_MD)),
    )
    .style(Style::default().fg(self.theme.accent));
f.render_widget(input, chunks[1]);
```

## Root Cause Analysis
The Paragraph widget style only sets foreground color. Without a visible background, the input blends into the terminal background. The bottom border might not be visible if terminal colors are similar.

## Solution: Add Background Highlight

Replace the input Paragraph with a styled version that has a visible background:

```rust
// Use a distinctive background color for the input
let display_text = format!("{}▌", self.search_query);  // ▌ is a block cursor
let input = Paragraph::new(display_text)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent).bg(self.theme.highlight))
            .padding(Padding::uniform(components::DesignTokens::PADDING_MD)),
    )
    .style(Style::default().fg(self.theme.foreground).bg(self.theme.highlight));
f.render_widget(input, chunks[1]);
```

**Changes:**
1. Changed `_` to `▌` (block cursor character) for a more visible cursor
2. Added `Borders::ALL` (full box) instead of just `Borders::BOTTOM`
3. Added `bg(self.theme.highlight)` to both border and text styles
4. Text now uses `self.theme.foreground` (bright) on `self.theme.highlight` (background) for contrast

## Implementation Steps

1. In `/home/zeroneroiv/projects/personal/youtui/src/ui/app.rs`, find `render_search` function
2. Replace lines 838-848 with the new input Paragraph code above
3. Run `cargo build` to verify compilation
4. Run `cargo run` and test search mode

## Success Criteria
- Input area has a visible box border (all 4 sides)
- Background of input area is highlighted (different from terminal background)
- Typed text is clearly visible (bright text on highlighted background)
- Cursor character `▌` is visible at end of input
