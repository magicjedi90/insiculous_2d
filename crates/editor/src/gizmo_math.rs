//! Pure math helpers for gizmo interactions.
//!
//! Kept separate from `gizmo.rs` so the coordinate conventions (screen Y grows
//! downward, world rotation is CCW-positive) are testable without a UI context.

use glam::Vec2;

/// World-space rotation delta for a mouse drag around `center`, in radians.
///
/// Screen Y grows downward while world rotation is CCW-positive, so the angle
/// is measured with a flipped Y. The result is wrapped to the shortest arc in
/// `(-PI, PI]` — without the wrap, a drag crossing the atan2 seam at ±PI would
/// produce a spurious ~2π jump.
pub fn world_rotation_delta(center: Vec2, last_mouse: Vec2, current_mouse: Vec2) -> f32 {
    let angle_of = |p: Vec2| (center.y - p.y).atan2(p.x - center.x);
    wrap_angle(angle_of(current_mouse) - angle_of(last_mouse))
}

/// Wrap an angle to the shortest arc in `(-PI, PI]`.
fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let wrapped = angle.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_drag_screen_up_on_right_side_is_ccw_positive() {
        // From (center + (r, 0)) to (center + (0, -r)): mouse moved up-left,
        // which is counter-clockwise in world terms -> +PI/2.
        let center = Vec2::new(100.0, 100.0);
        let delta = world_rotation_delta(
            center,
            center + Vec2::new(50.0, 0.0),
            center + Vec2::new(0.0, -50.0),
        );
        assert!((delta - PI / 2.0).abs() < EPS, "got {delta}");
    }

    #[test]
    fn test_drag_screen_down_on_right_side_is_cw_negative() {
        let center = Vec2::new(100.0, 100.0);
        let delta = world_rotation_delta(
            center,
            center + Vec2::new(50.0, 0.0),
            center + Vec2::new(0.0, 50.0),
        );
        assert!((delta + PI / 2.0).abs() < EPS, "got {delta}");
    }

    #[test]
    fn test_seam_crossing_returns_small_delta() {
        // Both points sit near the -X axis (world angle ~PI), on either side.
        // Naive subtraction would yield ~±2π; the wrap keeps it small.
        let center = Vec2::ZERO;
        let a = Vec2::new(-50.0, 1.0); // world angle just below +PI
        let b = Vec2::new(-50.0, -1.0); // world angle just above -PI
        let delta = world_rotation_delta(center, a, b);
        assert!(delta.abs() < 0.1, "expected small delta, got {delta}");
    }

    #[test]
    fn test_zero_movement_is_zero_delta() {
        let center = Vec2::new(10.0, 20.0);
        let p = Vec2::new(60.0, 20.0);
        assert_eq!(world_rotation_delta(center, p, p), 0.0);
    }

    #[test]
    fn test_wrap_angle_bounds() {
        // At the ±PI seam, float rounding may land on either sign — both
        // represent the same angle; only the magnitude matters.
        assert!((wrap_angle(3.0 * PI).abs() - PI).abs() < EPS);
        assert!((wrap_angle(-3.0 * PI).abs() - PI).abs() < EPS);
        assert!((wrap_angle(0.25) - 0.25).abs() < EPS);
        assert!((wrap_angle(-0.25) + 0.25).abs() < EPS);
    }
}
