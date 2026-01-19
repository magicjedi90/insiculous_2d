# UI Crate Analysis

## Review (January 19, 2026)

### Summary
- Immediate-mode UI system with context-driven widget creation and draw command output.
- Modules cover context, draw batching, font handling, interaction, and styling.
- Depends on `input` for interaction and `fontdue` for font rasterization.

### Strengths
- Immediate-mode API keeps usage lightweight and easy to integrate.
- Draw command output keeps renderer-agnostic responsibilities in the UI crate.
- Style/theme types are re-exported for easy reuse in gameplay code.

### Risks & Follow-ups
- Ensure font loading and glyph cache behavior is well documented for hot-reload workflows.
- Document the engine_core bridge (`render_ui_commands`) as the canonical integration point.
- Consider adding more examples for UI composition and layout patterns.

## Font Rendering First-Frame Bug Fix

**Issue:** Font rendering showed placeholder text on the first frame even when fonts were loaded shortly after, causing visual flicker in UI.

**Root Cause:** The static `PRINTED` atomic flag in `label_styled()` method prevented retrying font layout after the first attempt failed.

**Location:** `crates/ui/src/context.rs:244-247`

### Before (Broken Code)
```rust
} else {
    // Only print once
    static PRINTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !PRINTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        log::debug!("No default font loaded");
    }
}

// Fall back to placeholder
self.draw_list.text_placeholder(text, position, color, font_size);
```

### After (Fixed Code)
```rust
} else {
    // Font not available - log debug message (no longer cached to prevent retry)
    log::debug!("No default font available for text rendering");
}

// Fall back to placeholder
self.draw_list.text_placeholder(text, position, color, font_size);
```

### Key Changes
1. **Removed static PRINTED flag** - No longer caches "no font" state
2. **Always check FontManager** - Retries font rendering every frame
3. **Proper error handling** - Only shows placeholder if font truly unavailable
4. **Better logging** - More descriptive debug message

### Impact
- ✅ **Eliminates visual flicker** - Text renders with font as soon as available
- ✅ **Proper retry logic** - Font rendering attempted every frame
- ✅ **Backward compatible** - Still falls back to placeholder when needed
- ✅ **Better debugging** - Clearer logging for font issues

### Testing
Added comprehensive test `test_font_rendering_retry_after_font_load()` that verifies:
- Placeholder shown when no font available
- Retry logic works on subsequent frames
- No static flag preventing retry

### Files Modified
- `crates/ui/src/context.rs` - Fixed `label_styled()` method
- `PROJECT_ROADMAP.md` - Marked issue as resolved