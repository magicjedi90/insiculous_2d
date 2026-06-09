//! Debug-draw helpers. Currently only collider outlines.
//!
//! All helpers push [`LineVertex`] pairs into the buffer the game already
//! owns (`ctx.lines`), so the engine's line render pipeline picks them up
//! automatically with no extra plumbing.

use glam::{Vec2, Vec4};
use renderer::line_pipeline::LineVertex;

#[cfg(feature = "physics")]
use ecs::World;
#[cfg(feature = "physics")]
use common::Transform2D;
#[cfg(feature = "physics")]
use physics::{Collider, ColliderShape};

/// How many segments to use when approximating a circle / capsule cap with
/// straight line pieces. 24 keeps the silhouette smooth at gameplay scale
/// without flooding the line buffer.
const CIRCLE_SEGMENTS: u32 = 24;

/// Append a single line segment.
fn push_segment(lines: &mut Vec<LineVertex>, a: Vec2, b: Vec2, color: Vec4, emissive: f32) {
    lines.push(LineVertex::new(a, color, emissive));
    lines.push(LineVertex::new(b, color, emissive));
}

/// Draw an axis-aligned rectangle outline.
///
/// `half_extents` are the half-width and half-height. `center` is where the
/// rectangle is anchored.
pub fn push_box_outline(
    lines: &mut Vec<LineVertex>,
    center: Vec2,
    half_extents: Vec2,
    color: Vec4,
    emissive: f32,
) {
    let tl = center + Vec2::new(-half_extents.x,  half_extents.y);
    let tr = center + Vec2::new( half_extents.x,  half_extents.y);
    let br = center + Vec2::new( half_extents.x, -half_extents.y);
    let bl = center + Vec2::new(-half_extents.x, -half_extents.y);
    push_segment(lines, tl, tr, color, emissive);
    push_segment(lines, tr, br, color, emissive);
    push_segment(lines, br, bl, color, emissive);
    push_segment(lines, bl, tl, color, emissive);
}

/// Draw a circle outline approximated by [`CIRCLE_SEGMENTS`] line segments.
pub fn push_circle_outline(
    lines: &mut Vec<LineVertex>,
    center: Vec2,
    radius: f32,
    color: Vec4,
    emissive: f32,
) {
    push_arc(lines, center, radius, 0.0, std::f32::consts::TAU, CIRCLE_SEGMENTS, color, emissive);
}

/// Internal: walk an arc from `start_angle` to `start_angle + sweep`,
/// emitting `segments` line segments. Used by circle + capsule cap drawing.
fn push_arc(
    lines: &mut Vec<LineVertex>,
    center: Vec2,
    radius: f32,
    start_angle: f32,
    sweep: f32,
    segments: u32,
    color: Vec4,
    emissive: f32,
) {
    let segments = segments.max(1);
    let step = sweep / segments as f32;
    let mut prev = center + Vec2::new(start_angle.cos(), start_angle.sin()) * radius;
    for i in 1..=segments {
        let angle = start_angle + step * i as f32;
        let next = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        push_segment(lines, prev, next, color, emissive);
        prev = next;
    }
}

/// Draw a Y-axis capsule outline: two vertical sides + two semicircular caps.
///
/// `half_height` is the cylindrical middle's half-extent (the part between
/// the cap centers). `radius` is the cap radius. Total visual height is
/// `2 * (half_height + radius)`.
pub fn push_capsule_y_outline(
    lines: &mut Vec<LineVertex>,
    center: Vec2,
    half_height: f32,
    radius: f32,
    color: Vec4,
    emissive: f32,
) {
    let top_cap_center = center + Vec2::new(0.0, half_height);
    let bot_cap_center = center + Vec2::new(0.0, -half_height);
    // Two straight sides between the cap centers.
    push_segment(
        lines,
        top_cap_center + Vec2::new( radius, 0.0),
        bot_cap_center + Vec2::new( radius, 0.0),
        color, emissive,
    );
    push_segment(
        lines,
        top_cap_center + Vec2::new(-radius, 0.0),
        bot_cap_center + Vec2::new(-radius, 0.0),
        color, emissive,
    );
    // Top cap: arc from 0 (right side) sweeping +PI to the left side.
    push_arc(lines, top_cap_center, radius, 0.0, std::f32::consts::PI, CIRCLE_SEGMENTS / 2, color, emissive);
    // Bottom cap: arc from PI (left side) sweeping +PI back to the right side.
    push_arc(lines, bot_cap_center, radius, std::f32::consts::PI, std::f32::consts::PI, CIRCLE_SEGMENTS / 2, color, emissive);
}

/// Draw an X-axis capsule outline. Same as [`push_capsule_y_outline`] but
/// rotated 90°.
pub fn push_capsule_x_outline(
    lines: &mut Vec<LineVertex>,
    center: Vec2,
    half_width: f32,
    radius: f32,
    color: Vec4,
    emissive: f32,
) {
    let right_cap_center = center + Vec2::new(half_width, 0.0);
    let left_cap_center  = center + Vec2::new(-half_width, 0.0);
    push_segment(
        lines,
        right_cap_center + Vec2::new(0.0,  radius),
        left_cap_center  + Vec2::new(0.0,  radius),
        color, emissive,
    );
    push_segment(
        lines,
        right_cap_center + Vec2::new(0.0, -radius),
        left_cap_center  + Vec2::new(0.0, -radius),
        color, emissive,
    );
    push_arc(lines, right_cap_center, radius, -std::f32::consts::FRAC_PI_2, std::f32::consts::PI, CIRCLE_SEGMENTS / 2, color, emissive);
    push_arc(lines, left_cap_center,  radius,  std::f32::consts::FRAC_PI_2, std::f32::consts::PI, CIRCLE_SEGMENTS / 2, color, emissive);
}

/// Walk every entity with a [`Collider`] + `Transform2D` and push its outline
/// into `lines`. Sensors get the same outline shape — sensor-ness is a
/// behavior, not a different geometry.
///
/// `color` and `emissive` apply uniformly. Pick a high emissive value
/// (e.g. 2.0) if you want the outlines to bloom and read clearly over
/// game sprites.
#[cfg(feature = "physics")]
pub fn draw_colliders(
    world: &World,
    lines: &mut Vec<LineVertex>,
    color: Vec4,
    emissive: f32,
) {
    for entity in world.entities() {
        let Some(transform) = world.get::<Transform2D>(entity) else { continue };
        let Some(collider) = world.get::<Collider>(entity) else { continue };
        let center = transform.position + collider.offset;
        match collider.shape {
            ColliderShape::Box { half_extents } => {
                push_box_outline(lines, center, half_extents, color, emissive);
            }
            ColliderShape::Circle { radius } => {
                push_circle_outline(lines, center, radius, color, emissive);
            }
            ColliderShape::CapsuleY { half_height, radius } => {
                push_capsule_y_outline(lines, center, half_height, radius, color, emissive);
            }
            ColliderShape::CapsuleX { half_height, radius } => {
                // CapsuleX stores half_height for the cylindrical middle, but
                // that field is named oddly — it's actually the half-WIDTH of
                // the horizontal capsule. See physics/src/components.rs.
                push_capsule_x_outline(lines, center, half_height, radius, color, emissive);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_outline_emits_four_segments() {
        let mut lines = Vec::new();
        push_box_outline(&mut lines, Vec2::ZERO, Vec2::new(10.0, 5.0), Vec4::ONE, 0.0);
        // 4 segments * 2 vertices each = 8 vertices.
        assert_eq!(lines.len(), 8);
    }

    #[test]
    fn circle_outline_emits_one_segment_per_arc_step() {
        let mut lines = Vec::new();
        push_circle_outline(&mut lines, Vec2::ZERO, 10.0, Vec4::ONE, 0.0);
        assert_eq!(lines.len() as u32, CIRCLE_SEGMENTS * 2);
    }

    #[test]
    fn capsule_y_outline_includes_sides_and_two_caps() {
        let mut lines = Vec::new();
        push_capsule_y_outline(&mut lines, Vec2::ZERO, 50.0, 10.0, Vec4::ONE, 0.0);
        // 2 straight sides (2 segments) + 2 caps at CIRCLE_SEGMENTS/2 each.
        let expected = 2 + (CIRCLE_SEGMENTS / 2) * 2;
        assert_eq!(lines.len() as u32, expected * 2);
    }

    #[test]
    fn circle_vertices_stay_on_radius() {
        let mut lines = Vec::new();
        push_circle_outline(&mut lines, Vec2::new(100.0, 50.0), 25.0, Vec4::ONE, 0.0);
        for v in &lines {
            let pos = Vec2::from_array(v.position);
            let dist = pos.distance(Vec2::new(100.0, 50.0));
            assert!((dist - 25.0).abs() < 0.01, "vertex {:?} not on radius 25", pos);
        }
    }
}
