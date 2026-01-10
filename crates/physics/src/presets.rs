//! Physics presets for common game types
//!
//! These presets provide tested, ready-to-use physics configurations
//! so developers don't have to guess at values.

use glam::Vec2;
use crate::components::{Collider, RigidBody};
use crate::world::PhysicsConfig;

/// Movement configuration for character controllers
#[derive(Debug, Clone)]
pub struct MovementConfig {
    /// Horizontal movement speed in pixels/second
    pub move_speed: f32,
    /// Jump impulse strength
    pub jump_impulse: f32,
    /// Linear damping for the character body
    pub damping: f32,
}

impl MovementConfig {
    /// Platformer preset: precise, responsive controls
    /// - Move speed: 120 px/s (~2 pixels/frame at 60 FPS)
    /// - Jump impulse: 420 (satisfying jump height)
    /// - Damping: 5.0 (stops quickly when not moving)
    pub fn platformer() -> Self {
        Self {
            move_speed: 120.0,
            jump_impulse: 420.0,
            damping: 5.0,
        }
    }

    /// Fast platformer preset: quicker movement for action games
    /// - Move speed: 200 px/s
    /// - Jump impulse: 500
    /// - Damping: 4.0
    pub fn platformer_fast() -> Self {
        Self {
            move_speed: 200.0,
            jump_impulse: 500.0,
            damping: 4.0,
        }
    }

    /// Top-down game preset: equal movement in all directions
    /// - Move speed: 150 px/s
    /// - No jump (top-down perspective)
    /// - Damping: 8.0 (quick stops)
    pub fn top_down() -> Self {
        Self {
            move_speed: 150.0,
            jump_impulse: 0.0,
            damping: 8.0,
        }
    }

    /// Floaty/space preset: low friction, momentum-based
    /// - Move speed: 100 px/s
    /// - Jump impulse: 300
    /// - Damping: 1.0 (maintains momentum)
    pub fn floaty() -> Self {
        Self {
            move_speed: 100.0,
            jump_impulse: 300.0,
            damping: 1.0,
        }
    }
}

/// Preset rigid body configurations
impl RigidBody {
    /// Create a player body optimized for platformer games
    pub fn player_platformer() -> Self {
        Self::new_dynamic()
            .with_linear_damping(MovementConfig::platformer().damping)
            .with_rotation_locked(true)
            .with_ccd(true)
    }

    /// Create a player body optimized for top-down games
    pub fn player_top_down() -> Self {
        Self::new_dynamic()
            .with_linear_damping(MovementConfig::top_down().damping)
            .with_rotation_locked(true)
            .with_ccd(true)
    }

    /// Create a pushable object (crate, barrel, etc.)
    pub fn pushable() -> Self {
        Self::new_dynamic()
            .with_linear_damping(5.0)
            .with_ccd(true)
    }

    /// Create a physics prop that can tumble and roll
    pub fn physics_prop() -> Self {
        Self::new_dynamic()
            .with_linear_damping(2.0)
            .with_angular_damping(1.0)
            .with_ccd(true)
    }
}

/// Preset collider configurations
impl Collider {
    /// Create a player-sized box collider (matches default 80x80 sprite)
    pub fn player_box() -> Self {
        Self::box_collider(80.0, 80.0)
            .with_friction(0.8)
    }

    /// Create a small box collider (40x40)
    pub fn small_box() -> Self {
        Self::box_collider(40.0, 40.0)
            .with_friction(0.5)
    }

    /// Create a pushable object collider with low friction
    pub fn pushable_box(width: f32, height: f32) -> Self {
        Self::box_collider(width, height)
            .with_friction(0.3)
            .with_restitution(0.2)
    }

    /// Create a ground/platform collider
    pub fn platform(width: f32, height: f32) -> Self {
        Self::box_collider(width, height)
            .with_friction(0.8)
    }

    /// Create a bouncy collider (for trampolines, bumpers, etc.)
    pub fn bouncy(width: f32, height: f32) -> Self {
        Self::box_collider(width, height)
            .with_friction(0.3)
            .with_restitution(0.9)
    }

    /// Create a slippery collider (ice, oil, etc.)
    pub fn slippery(width: f32, height: f32) -> Self {
        Self::box_collider(width, height)
            .with_friction(0.05)
    }
}

/// Preset physics world configurations
impl PhysicsConfig {
    /// Standard platformer physics
    /// - Gravity: -980 (feels like ~10 m/s^2 with 100 px/m scale)
    /// - High solver iterations for stable stacking
    pub fn platformer() -> Self {
        Self::new(Vec2::new(0.0, -980.0))
            .with_iterations(16, 8)
            .with_scale(100.0)
    }

    /// Top-down game physics (no gravity)
    pub fn top_down() -> Self {
        Self::new(Vec2::ZERO)
            .with_iterations(8, 4)
            .with_scale(100.0)
    }

    /// Low gravity (moon-like, floaty jumps)
    pub fn low_gravity() -> Self {
        Self::new(Vec2::new(0.0, -300.0))
            .with_iterations(12, 6)
            .with_scale(100.0)
    }

    /// High gravity (heavy, impactful movement)
    pub fn high_gravity() -> Self {
        Self::new(Vec2::new(0.0, -1500.0))
            .with_iterations(16, 8)
            .with_scale(100.0)
    }

    /// Space physics (no gravity, low iterations)
    pub fn space() -> Self {
        Self::new(Vec2::ZERO)
            .with_iterations(4, 2)
            .with_scale(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_presets() {
        let platformer = MovementConfig::platformer();
        assert_eq!(platformer.move_speed, 120.0);
        assert_eq!(platformer.jump_impulse, 420.0);
        assert_eq!(platformer.damping, 5.0);

        let top_down = MovementConfig::top_down();
        assert_eq!(top_down.jump_impulse, 0.0);
    }

    #[test]
    fn test_rigid_body_presets() {
        let player = RigidBody::player_platformer();
        assert!(!player.can_rotate);
        assert!(player.ccd_enabled);
        assert_eq!(player.linear_damping, 5.0);
    }

    #[test]
    fn test_collider_presets() {
        let player = Collider::player_box();
        assert_eq!(player.friction, 0.8);

        let bouncy = Collider::bouncy(50.0, 50.0);
        assert_eq!(bouncy.restitution, 0.9);
    }

    #[test]
    fn test_physics_config_presets() {
        let platformer = PhysicsConfig::platformer();
        assert_eq!(platformer.gravity.y, -980.0);

        let top_down = PhysicsConfig::top_down();
        assert_eq!(top_down.gravity, Vec2::ZERO);
    }
}
