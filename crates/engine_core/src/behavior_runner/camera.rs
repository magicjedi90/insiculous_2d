//! `Behavior::CameraFollow` — smoothed camera tracking with input-driven
//! look-ahead (the camera leans the way the player is pressing).

use glam::Vec2;

use ecs::behavior::BehaviorState;
use ecs::{EntityId, World};
use input::{GameAction, InputHandler};

use super::BehaviorRunner;

/// The `CameraFollow` behavior's scene-authored fields, bundled so the
/// handler keeps a short argument list.
pub(super) struct CameraFollowParams<'a> {
    /// Tag of the entity to follow.
    pub target_tag: &'a str,
    /// Fraction of the remaining distance covered per frame at 60 FPS.
    pub lerp_speed: f32,
    /// Fixed offset from the target position.
    pub offset: (f32, f32),
    /// Optional dead-zone box (full width, full height).
    pub dead_zone: Option<(f32, f32)>,
    /// Maximum look-ahead shift per axis while a direction is held.
    pub look_ahead: (f32, f32),
    /// Smoothing fraction per frame at 60 FPS for the look-ahead ramp.
    pub look_ahead_lerp: f32,
}

impl BehaviorRunner {
    /// `lerp_speed` is calibrated as fraction-per-frame at this frame rate;
    /// `update_camera_follow` dt-corrects so other frame rates converge at
    /// the same wall-clock speed.
    const CAMERA_LERP_REFERENCE_FPS: f32 = 60.0;

    /// `Behavior::CameraFollow` — exponentially lerp this entity toward the
    /// nearest tagged entity (plus `offset` and the smoothed look-ahead
    /// lead), optionally ignoring movement while the focus point stays
    /// inside a dead-zone box centered on the entity (the camera then moves
    /// just far enough to keep the focus on the box edge).
    ///
    /// The look-ahead direction is digital (−1/0/+1 per axis) and comes from
    /// this runner's own `InputMapping<GameAction>` — the same mapping
    /// `PlayerPlatformer` reads, so camera and player cannot disagree.
    /// Opposing directions cancel and the lead decays back to zero.
    ///
    /// Returns the camera's new position, or `None` when it should not move
    /// (no target, no transform, or already at rest).
    pub(super) fn update_camera_follow(
        &self,
        world: &World,
        entity: EntityId,
        input: &InputHandler,
        delta_time: f32,
        params: &CameraFollowParams<'_>,
        state: &mut BehaviorState,
    ) -> Option<Vec2> {
        let target_pos = Self::find_nearest_tagged_position(world, entity, params.target_tag)?;
        let pos = Self::get_position(world, entity)?;

        // Scene data is hand-editable: a negative or NaN look-ahead must
        // degrade to plain follow, never corrupt the transform.
        let sanitize_extent = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let look_ahead = Vec2::new(
            sanitize_extent(params.look_ahead.0),
            sanitize_extent(params.look_ahead.1),
        );
        // Finite check first: `f32::clamp` propagates NaN.
        let look_lerp = if params.look_ahead_lerp.is_finite() {
            params.look_ahead_lerp.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let dir_x = i8::from(self.actions.is_active(GameAction::MoveRight, input))
            - i8::from(self.actions.is_active(GameAction::MoveLeft, input));
        // +y is up, matching PlayerTopDown.
        let dir_y = i8::from(self.actions.is_active(GameAction::MoveUp, input))
            - i8::from(self.actions.is_active(GameAction::MoveDown, input));
        let target_look = Vec2::new(
            f32::from(dir_x) * look_ahead.x,
            f32::from(dir_y) * look_ahead.y,
        );
        let look_alpha = Self::dt_corrected_alpha(look_lerp, delta_time);
        state.look_offset += (target_look - state.look_offset) * look_alpha;

        let focus = target_pos + Vec2::new(params.offset.0, params.offset.1) + state.look_offset;
        let desired = match params.dead_zone {
            Some((w, h)) => {
                let delta = focus - pos;
                let excess = Vec2::new(
                    delta.x - delta.x.clamp(-w * 0.5, w * 0.5),
                    delta.y - delta.y.clamp(-h * 0.5, h * 0.5),
                );
                pos + excess
            }
            None => focus,
        };

        let alpha = Self::dt_corrected_alpha(params.lerp_speed.clamp(0.0, 1.0), delta_time);
        let new_pos = pos + (desired - pos) * alpha;
        (new_pos != pos).then_some(new_pos)
    }

    /// Convert a fraction-per-frame-at-60-FPS smoothing rate into this
    /// frame's blend factor.
    fn dt_corrected_alpha(fraction_per_frame: f32, delta_time: f32) -> f32 {
        1.0 - (1.0 - fraction_per_frame).powf(delta_time * Self::CAMERA_LERP_REFERENCE_FPS)
    }
}
