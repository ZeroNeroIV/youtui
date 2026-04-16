# Fix: Search Input Text Not Visible

## Problem
User can see the input box/border but cannot see the typed text inside.

## Root Cause
The text color (`self.theme.foreground`) might be invisible against the background color (`self.theme.highlight`).

## Solution: Force Bright White Text

Make the text color explicitly bright/white to ensure visibility:

```rust
use ratatui::style::Color;

// Replace the input Paragraph with:
let display_text = if self.search_query.is_empty() {
    "Type here...".to_string()
} else {
    format!("{}▌", self.search_query)
};

let input = Paragraph::new(display_text)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent))
            .padding(Padding::uniform(components::DesignTokens::PADDING_MD)),
    )
    .style(Style::default().fg(Color::White).bg(self.theme.highlight));
f.render_widget(input, chunks[1]);
```

**Key changes:**
1. Force `Color::White` for text - guaranteed visibility
2. Keep `self.theme.highlight` for background
3. If search_query is empty, show placeholder text "Type here..."
4. Use `▌` (block cursor) at end of text

## Implementation Steps

1. Read current `render_search` function in `/home/zeroneroiv/projects/personal/youtui/src/ui/app.rs`
2. Add `use ratatui::style::Color;` at top of file if not present
3. Replace the input Paragraph code with the new version above
4. Run `cargo build` to verify
5. Test with `cargo run`

## Success Criteria
- Text is visible as bright white against highlighted background
- Placeholder text shows when input is empty
- Typed characters appear immediately in white
