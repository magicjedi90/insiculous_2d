# Phase 1: Fix Broken Functionality

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the animation rendering bug and investigate UI slider/button issues in hello_world.rs

**Architecture:** Two independent bugs - animation system has rendering integration gap, UI may have value update pattern issues in example code

**Tech Stack:** Rust, ECS (ecs crate), Renderer (renderer crate), UI (ui crate)

---

## Task 1: Fix Animation Frames Not Applied During Rendering

**Files:**
- Modify: `crates/ecs/src/sprite_system.rs:64-81`
- Test: `crates/ecs/tests/sprite_components.rs` (add new test)

**Step 1: Write the failing test**

Add to `crates/ecs/tests/sprite_components.rs`:

```rust
#[test]
fn test_sprite_render_system_applies_animation_frame() {
    use ecs::{World, System};
    use ecs::sprite_system::SpriteRenderSystem;

    let mut world = World::new();
    let entity = world.create_entity();

    // Create sprite with default tex_region [0,0,1,1]
    world.add_component(&entity, Sprite::new(1)).ok();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).ok();

    // Create animation with specific frame regions
    let frames = vec![
        [0.0, 0.0, 0.25, 0.25],  // Frame 0: top-left quarter
        [0.25, 0.0, 0.25, 0.25], // Frame 1: next quarter
    ];
    let mut animation = SpriteAnimation::new(10.0, frames);
    animation.update(0.15); // Advance to frame 1
    assert_eq!(animation.current_frame, 1);
    world.add_component(&entity, animation).ok();

    // Run sprite render system
    let mut render_system = SpriteRenderSystem::new();
    render_system.update(&mut world, 0.016);

    // The rendered sprite should use animation frame 1's tex_region
    let render_data = render_system.render_data();
    assert_eq!(render_data.sprite_count(), 1);

    let rendered_sprite = &render_data.sprites[0];
    // Should be frame 1's region [0.25, 0.0, 0.25, 0.25], not sprite default [0,0,1,1]
    assert_eq!(rendered_sprite.tex_region, [0.25, 0.0, 0.25, 0.25]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ecs test_sprite_render_system_applies_animation_frame -- --nocapture`

Expected: FAIL - assertion fails because tex_region is `[0.0, 0.0, 1.0, 1.0]` (sprite default) instead of `[0.25, 0.0, 0.25, 0.25]` (animation frame)

**Step 3: Write minimal implementation**

Modify `crates/ecs/src/sprite_system.rs` - change `convert_sprite` method (lines 64-81):

```rust
/// Convert ECS sprite to renderer sprite data
fn convert_sprite(
    &self,
    entity_transform: &Transform2D,
    sprite: &Sprite,
    animation: Option<&SpriteAnimation>,
) -> RendererSprite {
    let world_position = entity_transform.position + entity_transform.transform_point(sprite.offset);
    let world_rotation = entity_transform.rotation + sprite.rotation;
    let world_scale = entity_transform.scale * sprite.scale;

    // Use animation frame's tex_region if animation exists, otherwise use sprite's tex_region
    let tex_region = animation
        .map(|anim| anim.current_frame_region())
        .unwrap_or(sprite.tex_region);

    // Create renderer sprite using builder pattern
    RendererSprite::new(renderer::TextureHandle { id: sprite.texture_handle })
        .with_position(world_position)
        .with_rotation(world_rotation)
        .with_scale(world_scale * 80.0) // Default size
        .with_color(sprite.color)
        .with_depth(sprite.depth)
        .with_tex_region(tex_region[0], tex_region[1], tex_region[2], tex_region[3])
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ecs test_sprite_render_system_applies_animation_frame -- --nocapture`

Expected: PASS

**Step 5: Run all ECS tests to ensure no regressions**

Run: `cargo test -p ecs`

Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/ecs/src/sprite_system.rs crates/ecs/tests/sprite_components.rs
git commit -m "$(cat <<'EOF'
fix: apply animation frame tex_region during sprite rendering

SpriteRenderSystem.convert_sprite() was accepting an animation parameter
but ignoring it. Now correctly applies the current animation frame's
texture region to the rendered sprite.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Investigate UI Slider Value Update Pattern

**Files:**
- Read: `examples/hello_world.rs:315-322`
- Read: `crates/ui/src/context.rs` (slider implementation)

**Step 1: Analyze hello_world.rs slider code**

Current code (lines 315-322):
```rust
// Volume slider
ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
let new_volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
if (new_volume - self.volume).abs() > 0.01 {
    self.volume = new_volume;
    ctx.audio.set_master_volume(self.volume);
}
```

**Step 2: Identify the bug**

The code has a threshold check `(new_volume - self.volume).abs() > 0.01` which:
- Only updates `self.volume` when the change exceeds 0.01
- This means small drags don't update the stored value
- On next frame, slider receives old `self.volume`, making it appear "stuck"

**Step 3: Document the fix needed**

The fix is to always update `self.volume` with the returned value:

```rust
// Volume slider
ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
self.volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
ctx.audio.set_master_volume(self.volume);
```

However, this calls `set_master_volume` every frame. Better approach:

```rust
// Volume slider
ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
let new_volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
if new_volume != self.volume {
    self.volume = new_volume;
    ctx.audio.set_master_volume(self.volume);
}
```

**Step 4: Apply the fix**

Modify `examples/hello_world.rs` lines 315-322:

```rust
// Volume slider
ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
let new_volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
if new_volume != self.volume {
    self.volume = new_volume;
    ctx.audio.set_master_volume(self.volume);
}
```

**Step 5: Commit**

```bash
git add examples/hello_world.rs
git commit -m "$(cat <<'EOF'
fix: update slider value on any change, not just > 0.01 threshold

The threshold check caused the slider to appear "stuck" because small
changes didn't update the stored value, causing the next frame to
reset the slider position.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Investigate UI Button Issues

**Files:**
- Read: `examples/hello_world.rs:324-341` (button code)
- Read: `crates/ui/src/context.rs` (button implementation)

**Step 1: Analyze hello_world.rs button code**

Current code (lines 324-341):
```rust
// Music toggle button
let music_btn_rect = UIRect::new(20.0, 100.0, 90.0, 30.0);
let music_label = if self.music_playing { "Pause" } else { "Play" };
if ctx.ui.button("music_btn", music_label, music_btn_rect) {
    if self.music_playing {
        ctx.audio.pause_music();
        self.music_playing = false;
    } else {
        ctx.audio.resume_music();
        self.music_playing = true;
    }
}

// Reset button
let reset_btn_rect = UIRect::new(120.0, 100.0, 90.0, 30.0);
if ctx.ui.button("reset_btn", "Reset", reset_btn_rect) {
    self.reset_player(ctx);
}
```

**Step 2: Check button implementation**

Read `crates/ui/src/context.rs` to verify button click detection works correctly.

**Step 3: Run hello_world to manually test**

Run: `cargo run --example hello_world`

Test:
1. Click the "Play/Pause" button - does it toggle?
2. Click the "Reset" button - does it reset player position?
3. Check console output for any errors

**Step 4: Document findings**

If buttons work: No code change needed, issue may have been perceived due to slider issue.

If buttons don't work: Document specific behavior and create follow-up task.

**Step 5: Commit documentation (if no code change)**

If no code change needed, skip this step.

---

## Task 4: Verify All Fixes Together

**Step 1: Run all tests**

Run: `cargo test --workspace`

Expected: All tests pass

**Step 2: Run hello_world example**

Run: `cargo run --example hello_world`

Verify:
1. Slider drags smoothly without getting stuck
2. Buttons respond to clicks
3. If there's an animated sprite, animation frames display correctly

**Step 3: Final commit (if any cleanup needed)**

If additional fixes were needed, commit them.

---

## Success Criteria

- [ ] Animation test passes: `test_sprite_render_system_applies_animation_frame`
- [ ] Slider in hello_world drags smoothly
- [ ] Buttons in hello_world respond to clicks
- [ ] All workspace tests pass: `cargo test --workspace`
