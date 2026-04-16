# Fix: String Truncation Panic with Unicode

## Problem
Panic at `src/ui/components.rs:140:32` when rendering video titles.

**Root Cause**: The code uses byte slicing `&title[..37]` to truncate titles. When titles contain Unicode characters (emojis, non-ASCII), this slices in the middle of a multi-byte character, causing a panic.

## Current Code (line 139-143)
```rust
let title_truncated = if title.len() > DesignTokens::TRUNCATE_LEN {
    format!("{}...", &title[..DesignTokens::TRUNCATE_LEN - 3])
} else {
    title.to_string()
};
```

**Problem**: `title.len()` returns bytes, but we need to truncate by characters.

## Solution
Use character-based truncation instead of byte-based:

```rust
let title_truncated: String = if title.chars().count() > DesignTokens::TRUNCATE_LEN {
    title.chars().take(DesignTokens::TRUNCATE_LEN - 3).collect::<String>() + "..."
} else {
    title.clone()
};
```

**Changes:**
1. `title.chars().count()` - counts Unicode characters, not bytes
2. `title.chars().take(n)` - takes n characters safely
3. `.clone()` instead of `.to_string()` for consistency

## Implementation Steps

1. Open `/home/zeroneroiv/projects/personal/youtui/src/ui/components.rs`
2. Find `render_item_card` function
3. Replace lines 139-143 with the character-based truncation code above
4. Run `cargo build` to verify
5. Test with `cargo run`

## Success Criteria
- No panic when rendering video titles with emojis/Unicode
- Titles still truncate correctly
- Build passes
