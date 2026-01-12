# UI Crate Analysis

## Overview

The UI crate provides an immediate-mode user interface framework for the Insiculous 2D game engine. It follows the immediate-mode paradigm where you describe the UI every frame rather than retaining UI state, making it simple and intuitive to use.

## Architecture

### Core Types

**UIContext** (`context.rs`)
- Main entry point for creating UI elements
- Manages interaction state and draw command generation
- Provides widget methods: `button()`, `slider()`, `label()`, `panel()`, `checkbox()`, `progress_bar()`
- Handles theming through `Theme` struct
- Integrates `FontManager` for text rendering

**FontManager** (`font.rs`)
- Loads TTF/OTF fonts via fontdue library
- Rasterizes glyphs on demand with caching
- Provides text layout and measurement
- Automatic fallback to placeholder rendering when no font loaded

**Rect** (`rect.rs`)
- Rectangle type for UI layout and hit detection
- Screen-space coordinates (0,0 = top-left)
- Methods: `contains()`, `intersects()`, `expand()`, `shrink()`, `offset()`

**Color and Theme** (`style.rs`)
- `Color` - RGBA color with common presets (WHITE, BLACK, RED, etc.)
- Color utilities: `from_hex()`, `lerp()`, `darken()`, `lighten()`
- `Theme` - Global styling with dark/light presets
- Widget styles: `ButtonStyle`, `PanelStyle`, `SliderStyle`, `TextStyle`

**Interaction** (`interaction.rs`)
- `WidgetId` - Unique identifier for widgets (from string or integer)
- `InteractionManager` - Tracks hot/active widgets and mouse state
- `WidgetState` - Normal, Hovered, Active, Disabled states
- Persistent state storage for widgets that need frame-to-frame data

**Draw Commands** (`draw.rs`)
- `DrawCommand` - Enum for all renderable primitives (Rect, Text, TextPlaceholder, Circle, Line)
- `TextDrawData` - Contains text with rasterized glyph data for rendering
- `GlyphDrawData` - Individual glyph bitmap with position
- `DrawList` - Collects draw commands for a frame
- Base depth of 1000.0 ensures UI renders on top of game content

## Font Rendering

The UI crate now includes fontdue-based font rendering:

### Loading Fonts
```rust
// In Game::init()
match ctx.ui.load_font_file("assets/fonts/myfont.ttf") {
    Ok(handle) => println!("Font loaded!"),
    Err(e) => println!("Font load failed: {}", e),
}
```

### Text Rendering Behavior
- If a font is loaded: `label()` renders actual glyph shapes
- If no font loaded: `label()` falls back to placeholder rectangles
- Each glyph is rendered as a separate sprite with the text color

### Font API
- `ctx.ui.load_font(bytes)` - Load from memory
- `ctx.ui.load_font_file(path)` - Load from file
- `ctx.ui.default_font()` - Get default font handle
- `ctx.ui.set_default_font(handle)` - Set default font
- `ctx.ui.font_manager()` - Access FontManager directly

## Integration with Engine Core

The UI system integrates with engine_core through:

1. **GameContext** - `ctx.ui: &mut UIContext` available in `Game::update()`
2. **RenderContext** - `ui_commands: &[DrawCommand]` passed to render
3. **Automatic rendering** - `render_ui_commands()` converts draw commands to sprites

### Frame Lifecycle

```
1. GameRunner::update_and_render()
   ↓
2. ui_context.begin_frame(&input, window_size)
   ↓
3. Game::update(&mut ctx)  // User creates UI via ctx.ui
   ↓
4. ui_context.end_frame()  // Garbage collect unused widget state
   ↓
5. Get ui_commands from draw_list
   ↓
6. Game::render(&mut ctx)  // render_ui_commands() converts to sprites
   ↓
7. Sprites rendered with other game content
```

## Coordinate System

- UI uses **screen coordinates** (0,0 = top-left, Y increases downward)
- Game world uses **world coordinates** (0,0 = center, Y increases upward)
- `render_ui_commands()` converts screen → world for sprite rendering

## Current Limitations

1. **Rounded Corners** - `corner_radius` parameter is stored but not rendered (sprites are rectangular).

2. **Circles** - Rendered as squares. Would need circle shader or texture.

3. **Clipping** - No scissor rect support for scrollable containers.

4. **Glyph Textures** - Glyphs currently render as solid rectangles (correct shape, correct position). Full bitmap rendering requires dynamic texture creation.

## Test Coverage

54 tests covering:
- Rect operations and hit testing
- Color manipulation and conversion
- Theme creation (dark/light)
- Widget ID generation
- Interaction manager state
- Draw command creation (including Text and TextPlaceholder)
- UIContext widget methods
- Font manager access and integration
- Label rendering with and without fonts

## Usage Example

```rust
fn init(&mut self, ctx: &mut GameContext) {
    // Load a font for text rendering
    ctx.ui.load_font_file("assets/fonts/font.ttf").ok();
}

fn update(&mut self, ctx: &mut GameContext) {
    // Panel background
    ctx.ui.panel(UIRect::new(10.0, 10.0, 200.0, 100.0));

    // Label - renders with font glyphs if loaded
    ctx.ui.label("Score: 100", Vec2::new(20.0, 30.0));

    // Button - returns true when clicked
    if ctx.ui.button("my_btn", "Click Me", UIRect::new(20.0, 50.0, 80.0, 30.0)) {
        println!("Button clicked!");
    }

    // Slider - returns new value when dragged
    self.volume = ctx.ui.slider("volume", self.volume, UIRect::new(20.0, 90.0, 160.0, 20.0));
}
```

## Future Enhancements

1. **Glyph Texture Atlases** - Create GPU textures from glyph bitmaps for proper text rendering
2. **Layout System** - Flex-like layout with rows, columns, anchoring
3. **Scrollable Containers** - With clipping support
4. **More Widgets** - Dropdown, text input, tabs, trees
5. **Animation** - Smooth transitions for hover/press states
