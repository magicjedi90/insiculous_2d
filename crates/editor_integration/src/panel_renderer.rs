//! Panel content rendering for editor dock panels.
//!
//! Extracted from editor_demo.rs — renders the content inside each dock panel
//! (scene view, hierarchy tree, inspector, asset browser).

use glam::Vec2;

use editor::{
    EditorContext, HierarchyPanel, InspectorStyle, PanelId,
    EditableInspector,
    edit_transform2d, edit_sprite, edit_rigid_body, edit_collider, edit_audio_source,
    inspect_component,
};
use engine_core::contexts::GameContext;

/// Render the content of a specific dock panel.
pub fn render_panel_content(
    editor: &mut EditorContext,
    ctx: &mut GameContext,
    panel_id: PanelId,
    bounds: common::Rect,
) {
    let padding = 8.0;
    let content_x = bounds.x + padding;
    let y = bounds.y + padding;

    match panel_id {
        PanelId::SCENE_VIEW => render_scene_view(editor, ctx, bounds),
        PanelId::HIERARCHY => render_hierarchy(editor, ctx, bounds),
        PanelId::INSPECTOR => render_inspector(editor, ctx, content_x, y),
        PanelId::ASSET_BROWSER => render_asset_browser(ctx, content_x, y),
        _ => render_default(ctx, content_x, y),
    }
}

/// Scene view — grid info, viewport origin crosshair, and play-state border.
fn render_scene_view(editor: &EditorContext, ctx: &mut GameContext, bounds: common::Rect) {
    let padding = 8.0;
    let content_x = bounds.x + padding;
    let y = bounds.y + padding;

    if editor.is_grid_visible() {
        ctx.ui.label(
            &format!("Grid: {}px", editor.grid_size()),
            Vec2::new(content_x, y),
        );
    }

    // Draw viewport origin crosshair
    let center = bounds.center();
    ctx.ui.circle(center, 5.0, ui::Color::new(0.3, 0.3, 0.3, 1.0));
    ctx.ui.line(
        Vec2::new(center.x - 20.0, center.y),
        Vec2::new(center.x + 20.0, center.y),
        ui::Color::new(0.4, 0.4, 0.4, 1.0),
        1.0,
    );
    ctx.ui.line(
        Vec2::new(center.x, center.y - 20.0),
        Vec2::new(center.x, center.y + 20.0),
        ui::Color::new(0.4, 0.4, 0.4, 1.0),
        1.0,
    );

    // Play-state border tint
    let border_color = editor.play_state().border_color();
    let w = if editor.in_play_session() { 3.0 } else { 1.0 };

    // Top
    ctx.ui.line(
        Vec2::new(bounds.x, bounds.y),
        Vec2::new(bounds.x + bounds.width, bounds.y),
        border_color, w,
    );
    // Bottom
    ctx.ui.line(
        Vec2::new(bounds.x, bounds.y + bounds.height),
        Vec2::new(bounds.x + bounds.width, bounds.y + bounds.height),
        border_color, w,
    );
    // Left
    ctx.ui.line(
        Vec2::new(bounds.x, bounds.y),
        Vec2::new(bounds.x, bounds.y + bounds.height),
        border_color, w,
    );
    // Right
    ctx.ui.line(
        Vec2::new(bounds.x + bounds.width, bounds.y),
        Vec2::new(bounds.x + bounds.width, bounds.y + bounds.height),
        border_color, w,
    );
}

/// Hierarchy — tree view with click-to-select and Ctrl toggle.
fn render_hierarchy(editor: &mut EditorContext, ctx: &mut GameContext, bounds: common::Rect) {
    let clicked = editor.hierarchy.render(
        ctx.ui,
        ctx.world,
        &mut editor.selection,
        bounds,
    );

    for entity_id in clicked {
        if ctx.input.keyboard().is_key_pressed(winit::keyboard::KeyCode::ControlLeft) {
            editor.selection.toggle(entity_id);
        } else {
            editor.selection.select(entity_id);
        }
        log::info!(
            "Selected entity: {} ({})",
            HierarchyPanel::entity_display_name(ctx.world, entity_id),
            entity_id.value()
        );
    }
}

/// Inspector — component inspection for the selected entity.
///
/// During Editing/Paused: renders editable fields with live writeback.
/// During Playing: renders read-only view via `inspect_component()`.
fn render_inspector(editor: &EditorContext, ctx: &mut GameContext, content_x: f32, mut y: f32) {
    let line_height = 20.0;

    let entity_id = match editor.selection.primary() {
        Some(id) => id,
        None => {
            ctx.ui.label("No selection", Vec2::new(content_x, y));
            return;
        }
    };

    ctx.ui.label(
        &format!("Entity: {}", entity_id.value()),
        Vec2::new(content_x, y),
    );
    y += line_height;

    if editor.is_playing() {
        render_inspector_readonly(ctx, entity_id, content_x, y);
    } else {
        render_inspector_editable(ctx, entity_id, content_x, y);
    }
}

/// Read-only inspector using `inspect_component()` (used during Playing).
fn render_inspector_readonly(
    ctx: &mut GameContext,
    entity_id: ecs::EntityId,
    content_x: f32,
    mut y: f32,
) {
    let style = InspectorStyle::default();
    let line_height = 20.0;

    if let Some(transform) = ctx.world.get::<common::Transform2D>(entity_id) {
        y += line_height * 0.5;
        y = inspect_component(ctx.ui, "Transform2D", transform, content_x, y, &style);
    }
    if let Some(sprite) = ctx.world.get::<ecs::sprite_components::Sprite>(entity_id) {
        y += line_height * 0.5;
        y = inspect_component(ctx.ui, "Sprite", sprite, content_x, y, &style);
    }
    if let Some(body) = ctx.world.get::<physics::components::RigidBody>(entity_id) {
        y += line_height * 0.5;
        y = inspect_component(ctx.ui, "RigidBody", body, content_x, y, &style);
    }
    if let Some(collider) = ctx.world.get::<physics::components::Collider>(entity_id) {
        y += line_height * 0.5;
        y = inspect_component(ctx.ui, "Collider", collider, content_x, y, &style);
    }
    if let Some(source) = ctx.world.get::<ecs::audio_components::AudioSource>(entity_id) {
        y += line_height * 0.5;
        let _ = inspect_component(ctx.ui, "AudioSource", source, content_x, y, &style);
    }
}

/// Editable inspector with live writeback (used during Editing/Paused).
fn render_inspector_editable(
    ctx: &mut GameContext,
    entity_id: ecs::EntityId,
    content_x: f32,
    mut y: f32,
) {
    let line_height = 20.0;
    let mut component_index: usize = 0;

    // --- Transform2D ---
    if let Some(transform) = ctx.world.get::<common::Transform2D>(entity_id).copied() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index);
        let result = edit_transform2d(&mut inspector, &transform);
        y = inspector.y();

        if result.position_changed || result.rotation_changed || result.scale_changed {
            if let Some(t) = ctx.world.get_mut::<common::Transform2D>(entity_id) {
                if result.position_changed { t.position = result.new_position; }
                if result.rotation_changed { t.rotation = result.new_rotation; }
                if result.scale_changed { t.scale = result.new_scale; }
            }
        }
        component_index += 1;
    }

    // --- Sprite ---
    if let Some(sprite) = ctx.world.get::<ecs::sprite_components::Sprite>(entity_id).cloned() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index);
        let result = edit_sprite(&mut inspector, &sprite);
        y = inspector.y();

        if result.offset_changed || result.rotation_changed || result.scale_changed
            || result.color_changed || result.depth_changed
        {
            if let Some(s) = ctx.world.get_mut::<ecs::sprite_components::Sprite>(entity_id) {
                if result.offset_changed { s.offset = result.new_offset; }
                if result.rotation_changed { s.rotation = result.new_rotation; }
                if result.scale_changed { s.scale = result.new_scale; }
                if result.color_changed { s.color = result.new_color; }
                if result.depth_changed { s.depth = result.new_depth; }
            }
        }
        component_index += 1;
    }

    // --- RigidBody ---
    if let Some(body) = ctx.world.get::<physics::components::RigidBody>(entity_id).cloned() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index);
        let result = edit_rigid_body(&mut inspector, &body);
        y = inspector.y();

        if result.velocity_changed || result.angular_velocity_changed
            || result.gravity_scale_changed || result.linear_damping_changed
            || result.angular_damping_changed || result.can_rotate_changed
            || result.ccd_enabled_changed
        {
            if let Some(rb) = ctx.world.get_mut::<physics::components::RigidBody>(entity_id) {
                if result.velocity_changed { rb.velocity = result.new_velocity; }
                if result.angular_velocity_changed { rb.angular_velocity = result.new_angular_velocity; }
                if result.gravity_scale_changed { rb.gravity_scale = result.new_gravity_scale; }
                if result.linear_damping_changed { rb.linear_damping = result.new_linear_damping; }
                if result.angular_damping_changed { rb.angular_damping = result.new_angular_damping; }
                if result.can_rotate_changed { rb.can_rotate = result.new_can_rotate; }
                if result.ccd_enabled_changed { rb.ccd_enabled = result.new_ccd_enabled; }
            }
        }
        component_index += 1;
    }

    // --- Collider ---
    if let Some(collider) = ctx.world.get::<physics::components::Collider>(entity_id).cloned() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index);
        let result = edit_collider(&mut inspector, &collider);
        y = inspector.y();

        if result.offset_changed || result.is_sensor_changed
            || result.friction_changed || result.restitution_changed
        {
            if let Some(c) = ctx.world.get_mut::<physics::components::Collider>(entity_id) {
                if result.offset_changed { c.offset = result.new_offset; }
                if result.is_sensor_changed { c.is_sensor = result.new_is_sensor; }
                if result.friction_changed { c.friction = result.new_friction; }
                if result.restitution_changed { c.restitution = result.new_restitution; }
            }
        }
        component_index += 1;
    }

    // --- AudioSource ---
    if let Some(source) = ctx.world.get::<ecs::audio_components::AudioSource>(entity_id).cloned() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index);
        let result = edit_audio_source(&mut inspector, &source);
        let _ = inspector.y();

        if result.volume_changed || result.pitch_changed || result.looping_changed
            || result.play_on_spawn_changed || result.spatial_changed
            || result.max_distance_changed || result.reference_distance_changed
            || result.rolloff_factor_changed
        {
            if let Some(a) = ctx.world.get_mut::<ecs::audio_components::AudioSource>(entity_id) {
                if result.volume_changed { a.volume = result.new_volume; }
                if result.pitch_changed { a.pitch = result.new_pitch; }
                if result.looping_changed { a.looping = result.new_looping; }
                if result.play_on_spawn_changed { a.play_on_spawn = result.new_play_on_spawn; }
                if result.spatial_changed { a.spatial = result.new_spatial; }
                if result.max_distance_changed { a.max_distance = result.new_max_distance; }
                if result.reference_distance_changed { a.reference_distance = result.new_reference_distance; }
                if result.rolloff_factor_changed { a.rolloff_factor = result.new_rolloff_factor; }
            }
        }
    }
}

/// Asset browser — placeholder.
fn render_asset_browser(ctx: &mut GameContext, content_x: f32, y: f32) {
    ctx.ui.label("(Asset browser not yet implemented)", Vec2::new(content_x, y));
}

/// Fallback for unknown panels.
fn render_default(ctx: &mut GameContext, content_x: f32, y: f32) {
    ctx.ui.label("Panel", Vec2::new(content_x, y));
}

#[cfg(test)]
mod tests {
    use ecs::World;
    use glam::Vec2;

    /// Verify Transform2D writeback applies position, rotation, and scale.
    #[test]
    fn test_transform_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();

        // Simulate edit result
        let result = editor::TransformEditResult {
            position_changed: true,
            new_position: Vec2::new(100.0, 200.0),
            rotation_changed: true,
            new_rotation: 1.5,
            scale_changed: false,
            new_scale: Vec2::ONE,
        };

        // Apply writeback (same pattern as render_inspector)
        if let Some(t) = world.get_mut::<common::Transform2D>(entity) {
            if result.position_changed { t.position = result.new_position; }
            if result.rotation_changed { t.rotation = result.new_rotation; }
            if result.scale_changed { t.scale = result.new_scale; }
        }

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.position, Vec2::new(100.0, 200.0));
        assert_eq!(t.rotation, 1.5);
        assert_eq!(t.scale, Vec2::ONE); // unchanged
    }

    /// Verify Sprite writeback applies changed fields only.
    #[test]
    fn test_sprite_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, ecs::sprite_components::Sprite::new(42)).ok();

        let result = editor::SpriteEditResult {
            color_changed: true,
            new_color: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
            depth_changed: true,
            new_depth: 5.0,
            ..Default::default()
        };

        if let Some(s) = world.get_mut::<ecs::sprite_components::Sprite>(entity) {
            if result.offset_changed { s.offset = result.new_offset; }
            if result.color_changed { s.color = result.new_color; }
            if result.depth_changed { s.depth = result.new_depth; }
        }

        let s = world.get::<ecs::sprite_components::Sprite>(entity).unwrap();
        assert_eq!(s.color, glam::Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(s.depth, 5.0);
        assert_eq!(s.texture_handle, 42); // unchanged
    }

    /// Verify RigidBody writeback applies physics properties.
    #[test]
    fn test_rigid_body_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, physics::components::RigidBody::default()).ok();

        let result = editor::RigidBodyEditResult {
            gravity_scale_changed: true,
            new_gravity_scale: 0.5,
            linear_damping_changed: true,
            new_linear_damping: 0.8,
            can_rotate_changed: true,
            new_can_rotate: false,
            ..Default::default()
        };

        if let Some(rb) = world.get_mut::<physics::components::RigidBody>(entity) {
            if result.gravity_scale_changed { rb.gravity_scale = result.new_gravity_scale; }
            if result.linear_damping_changed { rb.linear_damping = result.new_linear_damping; }
            if result.can_rotate_changed { rb.can_rotate = result.new_can_rotate; }
        }

        let rb = world.get::<physics::components::RigidBody>(entity).unwrap();
        assert_eq!(rb.gravity_scale, 0.5);
        assert_eq!(rb.linear_damping, 0.8);
        assert!(!rb.can_rotate);
    }

    /// Verify Collider writeback applies material properties.
    #[test]
    fn test_collider_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, physics::components::Collider::default()).ok();

        let result = editor::ColliderEditResult {
            friction_changed: true,
            new_friction: 0.9,
            is_sensor_changed: true,
            new_is_sensor: true,
            ..Default::default()
        };

        if let Some(c) = world.get_mut::<physics::components::Collider>(entity) {
            if result.friction_changed { c.friction = result.new_friction; }
            if result.is_sensor_changed { c.is_sensor = result.new_is_sensor; }
        }

        let c = world.get::<physics::components::Collider>(entity).unwrap();
        assert_eq!(c.friction, 0.9);
        assert!(c.is_sensor);
    }

    /// Verify AudioSource writeback applies audio properties.
    #[test]
    fn test_audio_source_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, ecs::audio_components::AudioSource::default()).ok();

        let result = editor::AudioSourceEditResult {
            volume_changed: true,
            new_volume: 0.5,
            spatial_changed: true,
            new_spatial: true,
            ..Default::default()
        };

        if let Some(a) = world.get_mut::<ecs::audio_components::AudioSource>(entity) {
            if result.volume_changed { a.volume = result.new_volume; }
            if result.spatial_changed { a.spatial = result.new_spatial; }
        }

        let a = world.get::<ecs::audio_components::AudioSource>(entity).unwrap();
        assert_eq!(a.volume, 0.5);
        assert!(a.spatial);
    }

    /// Verify writeback on non-existent entity is safe (no panic).
    #[test]
    fn test_writeback_missing_entity_is_safe() {
        let mut world = World::new();
        let fake_entity = ecs::EntityId::with_generation(999, 0);

        // Attempting writeback on a non-existent entity should return None, not panic
        let result = world.get_mut::<common::Transform2D>(fake_entity);
        assert!(result.is_none());
    }

    /// Verify writeback when component not present on entity is safe.
    #[test]
    fn test_writeback_missing_component_is_safe() {
        let mut world = World::new();
        let entity = world.create_entity();
        // Entity exists but has no Transform2D

        let result = world.get_mut::<common::Transform2D>(entity);
        assert!(result.is_none());
    }

    /// Verify rotation gizmo writeback applies rotation delta.
    #[test]
    fn test_rotation_gizmo_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();

        let rotation_delta = 0.5; // radians

        if let Some(transform) = world.get_mut::<common::Transform2D>(entity) {
            transform.rotation += rotation_delta;
        }

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.rotation, 0.5);
        assert_eq!(t.position, Vec2::ZERO); // unchanged
        assert_eq!(t.scale, Vec2::ONE); // unchanged
    }

    /// Verify scale gizmo writeback applies scale delta with clamping.
    #[test]
    fn test_scale_gizmo_writeback() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();

        let scale_delta = Vec2::new(0.5, 0.3);

        if let Some(transform) = world.get_mut::<common::Transform2D>(entity) {
            transform.scale += scale_delta;
            transform.scale = transform.scale.max(Vec2::splat(0.01));
        }

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.scale, Vec2::new(1.5, 1.3));
        assert_eq!(t.position, Vec2::ZERO); // unchanged
    }

    /// Verify scale gizmo writeback clamps to minimum (prevents zero/negative scale).
    #[test]
    fn test_scale_gizmo_writeback_clamps_minimum() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();

        // Apply a large negative delta that would make scale negative
        let scale_delta = Vec2::new(-5.0, -5.0);

        if let Some(transform) = world.get_mut::<common::Transform2D>(entity) {
            transform.scale += scale_delta;
            transform.scale = transform.scale.max(Vec2::splat(0.01));
        }

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.scale, Vec2::new(0.01, 0.01)); // clamped to minimum
    }
}
