//! Component-specific editable inspectors.
//!
//! Provides pre-built inspectors for common component types:
//! - Transform2D
//! - Sprite
//! - RigidBody
//! - Collider
//! - AudioSource

use ecs::sprite_components::Sprite;
use common::Transform2D;
use physics::components::{Collider, RigidBody, RigidBodyType, ColliderShape};
use ecs::audio_components::AudioSource;
use glam::{Vec2, Vec4};

use crate::editable_inspector::{EditResult, EditableInspector};

/// Inspector result containing which fields changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformEditResult {
    pub position_changed: bool,
    pub rotation_changed: bool,
    pub scale_changed: bool,
    pub new_position: Vec2,
    pub new_rotation: f32,
    pub new_scale: Vec2,
}

/// Edit a Transform2D component.
///
/// # Arguments
/// * `inspector` - The editable inspector builder
/// * `transform` - The current transform values
///
/// # Returns
/// TransformEditResult indicating which fields changed and their new values.
pub fn edit_transform2d(
    inspector: &mut EditableInspector<'_>,
    transform: &Transform2D,
) -> TransformEditResult {
    let mut result = TransformEditResult {
        new_position: transform.position,
        new_rotation: transform.rotation,
        new_scale: transform.scale,
        ..Default::default()
    };

    inspector.header("Transform2D");

    // Position (range: -1000 to 1000, typically covers most game worlds)
    if let EditResult::Changed(new_pos) = inspector.vec2("Position", transform.position, -1000.0, 1000.0) {
        result.position_changed = true;
        result.new_position = new_pos;
    }

    // Rotation (range: -π to π, displayed in radians)
    if let EditResult::Changed(new_rot) = inspector.f32("Rotation", transform.rotation, -std::f32::consts::PI, std::f32::consts::PI) {
        result.rotation_changed = true;
        result.new_rotation = new_rot;
    }

    // Scale (range: 0.01 to 10.0, prevents negative/zero scale)
    if let EditResult::Changed(new_scale) = inspector.vec2("Scale", transform.scale, 0.01, 10.0) {
        result.scale_changed = true;
        result.new_scale = new_scale;
    }

    result
}

/// Inspector result for Sprite component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpriteEditResult {
    pub offset_changed: bool,
    pub rotation_changed: bool,
    pub scale_changed: bool,
    pub color_changed: bool,
    pub depth_changed: bool,
    pub new_offset: Vec2,
    pub new_rotation: f32,
    pub new_scale: Vec2,
    pub new_color: Vec4,
    pub new_depth: f32,
}

/// Edit a Sprite component.
pub fn edit_sprite(
    inspector: &mut EditableInspector<'_>,
    sprite: &Sprite,
) -> SpriteEditResult {
    let mut result = SpriteEditResult {
        new_offset: sprite.offset,
        new_rotation: sprite.rotation,
        new_scale: sprite.scale,
        new_color: sprite.color,
        new_depth: sprite.depth,
        ..Default::default()
    };

    inspector.header("Sprite");

    // Offset
    if let EditResult::Changed(new_offset) = inspector.vec2("Offset", sprite.offset, -100.0, 100.0) {
        result.offset_changed = true;
        result.new_offset = new_offset;
    }

    // Rotation
    if let EditResult::Changed(new_rot) = inspector.f32("Rotation", sprite.rotation, -std::f32::consts::PI, std::f32::consts::PI) {
        result.rotation_changed = true;
        result.new_rotation = new_rot;
    }

    // Scale
    if let EditResult::Changed(new_scale) = inspector.vec2("Scale", sprite.scale, 0.01, 10.0) {
        result.scale_changed = true;
        result.new_scale = new_scale;
    }

    // Color
    if let EditResult::Changed(new_color) = inspector.color("Color", sprite.color) {
        result.color_changed = true;
        result.new_color = new_color;
    }

    // Depth (range: -100 to 100, covers typical depth sorting range)
    if let EditResult::Changed(new_depth) = inspector.f32("Depth", sprite.depth, -100.0, 100.0) {
        result.depth_changed = true;
        result.new_depth = new_depth;
    }

    // Texture handle (read-only)
    inspector.u32("Texture", sprite.texture_handle);

    result
}

/// Inspector result for RigidBody component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RigidBodyEditResult {
    pub body_type_changed: bool,
    pub velocity_changed: bool,
    pub angular_velocity_changed: bool,
    pub gravity_scale_changed: bool,
    pub linear_damping_changed: bool,
    pub angular_damping_changed: bool,
    pub can_rotate_changed: bool,
    pub ccd_enabled_changed: bool,
    pub new_body_type: RigidBodyType,
    pub new_velocity: Vec2,
    pub new_angular_velocity: f32,
    pub new_gravity_scale: f32,
    pub new_linear_damping: f32,
    pub new_angular_damping: f32,
    pub new_can_rotate: bool,
    pub new_ccd_enabled: bool,
}

/// Edit a RigidBody component.
pub fn edit_rigid_body(
    inspector: &mut EditableInspector<'_>,
    body: &RigidBody,
) -> RigidBodyEditResult {
    let mut result = RigidBodyEditResult {
        new_body_type: body.body_type,
        new_velocity: body.velocity,
        new_angular_velocity: body.angular_velocity,
        new_gravity_scale: body.gravity_scale,
        new_linear_damping: body.linear_damping,
        new_angular_damping: body.angular_damping,
        new_can_rotate: body.can_rotate,
        new_ccd_enabled: body.ccd_enabled,
        ..Default::default()
    };

    inspector.header("RigidBody");

    // Body type (read-only for now - would need dropdown widget)
    // TODO: Add enum dropdown when UI crate supports it
    let type_str = match body.body_type {
        RigidBodyType::Dynamic => "Dynamic",
        RigidBodyType::Static => "Static",
        RigidBodyType::Kinematic => "Kinematic",
    };
    // Display as read-only label
    inspector.header(&format!("  Type: {}", type_str));

    // Velocity
    if let EditResult::Changed(new_vel) = inspector.vec2("Velocity", body.velocity, -500.0, 500.0) {
        result.velocity_changed = true;
        result.new_velocity = new_vel;
    }

    // Angular velocity
    if let EditResult::Changed(new_ang_vel) = inspector.f32("Ang. Velocity", body.angular_velocity, -10.0, 10.0) {
        result.angular_velocity_changed = true;
        result.new_angular_velocity = new_ang_vel;
    }

    // Gravity scale (normalized 0-2 range, 1.0 is normal)
    if let EditResult::Changed(new_grav) = inspector.f32("Gravity Scale", body.gravity_scale, 0.0, 2.0) {
        result.gravity_scale_changed = true;
        result.new_gravity_scale = new_grav;
    }

    // Linear damping (0-1 range)
    if let EditResult::Changed(new_damp) = inspector.normalized_f32("Linear Damping", body.linear_damping) {
        result.linear_damping_changed = true;
        result.new_linear_damping = new_damp;
    }

    // Angular damping (0-1 range)
    if let EditResult::Changed(new_damp) = inspector.normalized_f32("Angular Damping", body.angular_damping) {
        result.angular_damping_changed = true;
        result.new_angular_damping = new_damp;
    }

    // Can rotate
    if let EditResult::Changed(new_can_rot) = inspector.bool("Can Rotate", body.can_rotate) {
        result.can_rotate_changed = true;
        result.new_can_rotate = new_can_rot;
    }

    // CCD enabled
    if let EditResult::Changed(new_ccd) = inspector.bool("CCD Enabled", body.ccd_enabled) {
        result.ccd_enabled_changed = true;
        result.new_ccd_enabled = new_ccd;
    }

    result
}

/// Inspector result for Collider component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ColliderEditResult {
    pub offset_changed: bool,
    pub is_sensor_changed: bool,
    pub friction_changed: bool,
    pub restitution_changed: bool,
    pub new_offset: Vec2,
    pub new_is_sensor: bool,
    pub new_friction: f32,
    pub new_restitution: f32,
}

/// Edit a Collider component.
pub fn edit_collider(
    inspector: &mut EditableInspector<'_>,
    collider: &Collider,
) -> ColliderEditResult {
    let mut result = ColliderEditResult {
        new_offset: collider.offset,
        new_is_sensor: collider.is_sensor,
        new_friction: collider.friction,
        new_restitution: collider.restitution,
        ..Default::default()
    };

    inspector.header("Collider");

    // Shape type (read-only for now - would need dropdown)
    let shape_str = match &collider.shape {
        ColliderShape::Box { .. } => "Box",
        ColliderShape::Circle { .. } => "Circle",
        ColliderShape::CapsuleY { .. } => "CapsuleY",
        ColliderShape::CapsuleX { .. } => "CapsuleX",
    };
    inspector.header(&format!("  Shape: {}", shape_str));

    // Offset
    if let EditResult::Changed(new_offset) = inspector.vec2("Offset", collider.offset, -100.0, 100.0) {
        result.offset_changed = true;
        result.new_offset = new_offset;
    }

    // Is sensor
    if let EditResult::Changed(new_sensor) = inspector.bool("Is Sensor", collider.is_sensor) {
        result.is_sensor_changed = true;
        result.new_is_sensor = new_sensor;
    }

    // Friction (0-1 range)
    if let EditResult::Changed(new_friction) = inspector.normalized_f32("Friction", collider.friction) {
        result.friction_changed = true;
        result.new_friction = new_friction;
    }

    // Restitution (0-1 range)
    if let EditResult::Changed(new_rest) = inspector.normalized_f32("Restitution", collider.restitution) {
        result.restitution_changed = true;
        result.new_restitution = new_rest;
    }

    // Collision groups/filter (read-only)
    inspector.u32("Groups", collider.collision_groups);
    inspector.u32("Filter", collider.collision_filter);

    result
}

/// Inspector result for AudioSource component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioSourceEditResult {
    pub volume_changed: bool,
    pub pitch_changed: bool,
    pub looping_changed: bool,
    pub play_on_spawn_changed: bool,
    pub spatial_changed: bool,
    pub max_distance_changed: bool,
    pub reference_distance_changed: bool,
    pub rolloff_factor_changed: bool,
    pub new_volume: f32,
    pub new_pitch: f32,
    pub new_looping: bool,
    pub new_play_on_spawn: bool,
    pub new_spatial: bool,
    pub new_max_distance: f32,
    pub new_reference_distance: f32,
    pub new_rolloff_factor: f32,
}

/// Edit an AudioSource component.
pub fn edit_audio_source(
    inspector: &mut EditableInspector<'_>,
    source: &AudioSource,
) -> AudioSourceEditResult {
    let mut result = AudioSourceEditResult {
        new_volume: source.volume,
        new_pitch: source.pitch,
        new_looping: source.looping,
        new_play_on_spawn: source.play_on_spawn,
        new_spatial: source.spatial,
        new_max_distance: source.max_distance,
        new_reference_distance: source.reference_distance,
        new_rolloff_factor: source.rolloff_factor,
        ..Default::default()
    };

    inspector.header("AudioSource");

    // Sound ID (read-only asset reference)
    inspector.u32("Sound ID", source.sound_id);

    // Volume (0-1 range)
    if let EditResult::Changed(new_vol) = inspector.normalized_f32("Volume", source.volume) {
        result.volume_changed = true;
        result.new_volume = new_vol;
    }

    // Pitch (0.1 to 3.0 range, allows slow-motion to chipmunk)
    if let EditResult::Changed(new_pitch) = inspector.f32("Pitch", source.pitch, 0.1, 3.0) {
        result.pitch_changed = true;
        result.new_pitch = new_pitch;
    }

    // Looping
    if let EditResult::Changed(new_loop) = inspector.bool("Looping", source.looping) {
        result.looping_changed = true;
        result.new_looping = new_loop;
    }

    // Play on spawn
    if let EditResult::Changed(new_play) = inspector.bool("Play on Spawn", source.play_on_spawn) {
        result.play_on_spawn_changed = true;
        result.new_play_on_spawn = new_play;
    }

    // Spatial audio enabled
    if let EditResult::Changed(new_spatial) = inspector.bool("Spatial", source.spatial) {
        result.spatial_changed = true;
        result.new_spatial = new_spatial;
    }

    // Spatial audio parameters (only relevant if spatial is true)
    if source.spatial {
        if let EditResult::Changed(new_dist) = inspector.f32("Max Distance", source.max_distance, 0.0, 5000.0) {
            result.max_distance_changed = true;
            result.new_max_distance = new_dist;
        }

        if let EditResult::Changed(new_ref) = inspector.f32("Ref Distance", source.reference_distance, 0.0, 1000.0) {
            result.reference_distance_changed = true;
            result.new_reference_distance = new_ref;
        }

        if let EditResult::Changed(new_roll) = inspector.f32("Rolloff", source.rolloff_factor, 0.0, 5.0) {
            result.rolloff_factor_changed = true;
            result.new_rolloff_factor = new_roll;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_edit_result_default() {
        let result = TransformEditResult::default();
        assert!(!result.position_changed);
        assert!(!result.rotation_changed);
        assert!(!result.scale_changed);
    }

    #[test]
    fn test_sprite_edit_result_default() {
        let result = SpriteEditResult::default();
        assert!(!result.offset_changed);
        assert!(!result.rotation_changed);
        assert!(!result.scale_changed);
        assert!(!result.color_changed);
        assert!(!result.depth_changed);
    }

    #[test]
    fn test_rigid_body_edit_result_default() {
        let result = RigidBodyEditResult::default();
        assert!(!result.velocity_changed);
        assert!(!result.angular_velocity_changed);
        assert!(!result.gravity_scale_changed);
    }

    #[test]
    fn test_collider_edit_result_default() {
        let result = ColliderEditResult::default();
        assert!(!result.offset_changed);
        assert!(!result.is_sensor_changed);
        assert!(!result.friction_changed);
        assert!(!result.restitution_changed);
    }

    #[test]
    fn test_audio_source_edit_result_default() {
        let result = AudioSourceEditResult::default();
        assert!(!result.volume_changed);
        assert!(!result.pitch_changed);
        assert!(!result.looping_changed);
    }

    #[test]
    fn test_transform_default_values() {
        let transform = Transform2D::default();
        assert_eq!(transform.position, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_sprite_default_values() {
        let sprite = Sprite::default();
        assert_eq!(sprite.offset, Vec2::ZERO);
        assert_eq!(sprite.rotation, 0.0);
        assert_eq!(sprite.scale, Vec2::ONE);
        assert_eq!(sprite.depth, 0.0);
    }

    #[test]
    fn test_rigid_body_default_values() {
        let body = RigidBody::default();
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.velocity, Vec2::ZERO);
        assert_eq!(body.gravity_scale, 1.0);
    }

    #[test]
    fn test_collider_default_values() {
        let collider = Collider::default();
        assert_eq!(collider.offset, Vec2::ZERO);
        assert!(!collider.is_sensor);
        assert_eq!(collider.friction, 0.5);
        assert_eq!(collider.restitution, 0.0);
    }

    #[test]
    fn test_audio_source_default_values() {
        let source = AudioSource::default();
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 1.0);
        assert!(!source.looping);
        assert!(!source.spatial);
    }
}
