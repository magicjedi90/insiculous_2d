//! Behavior components for entity behaviors
//!
//! This module provides behavior components that define how entities respond
//! to input and game events. Behaviors are data-driven and can be defined
//! in scene files.

use serde::{Deserialize, Serialize};

/// Behavior component that defines how an entity responds to input and events.
///
/// Each variant represents a different type of behavior with its own configuration.
/// Behaviors are processed by the `BehaviorRunner` in engine_core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Behavior {
    /// Player-controlled platformer movement (WASD + Space for jump)
    PlayerPlatformer {
        /// Horizontal movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
        /// Upward impulse applied when jumping
        #[serde(default = "default_jump_impulse")]
        jump_impulse: f32,
        /// Cooldown between jumps in seconds
        #[serde(default = "default_jump_cooldown")]
        jump_cooldown: f32,
    },

    /// Player-controlled top-down movement (WASD in all directions)
    PlayerTopDown {
        /// Movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
    },

    /// Follow another entity by name
    FollowEntity {
        /// Name of the target entity to follow
        target_name: String,
        /// Minimum distance to maintain from target
        #[serde(default = "default_follow_distance")]
        follow_distance: f32,
        /// Movement speed when following
        #[serde(default = "default_follow_speed")]
        follow_speed: f32,
    },

    /// Patrol between two world positions
    Patrol {
        /// First patrol point (x, y)
        point_a: (f32, f32),
        /// Second patrol point (x, y)
        point_b: (f32, f32),
        /// Movement speed
        #[serde(default = "default_patrol_speed")]
        speed: f32,
        /// Time to wait at each point before moving
        #[serde(default = "default_wait_time")]
        wait_time: f32,
    },

    /// Collectible item that can be picked up
    Collectible {
        /// Score value when collected
        #[serde(default = "default_score")]
        score_value: u32,
        /// Whether to despawn when collected
        #[serde(default = "default_true")]
        despawn_on_collect: bool,
    },

    /// AI that chases the player when in range
    ChasePlayer {
        /// Distance at which the entity starts chasing
        #[serde(default = "default_detection_range")]
        detection_range: f32,
        /// Movement speed when chasing
        #[serde(default = "default_chase_speed")]
        chase_speed: f32,
        /// Distance at which the entity stops chasing
        #[serde(default = "default_lose_range")]
        lose_interest_range: f32,
    },
}

// Default value functions for serde
fn default_move_speed() -> f32 { 120.0 }
fn default_jump_impulse() -> f32 { 420.0 }
fn default_jump_cooldown() -> f32 { 0.3 }
fn default_follow_distance() -> f32 { 50.0 }
fn default_follow_speed() -> f32 { 100.0 }
fn default_patrol_speed() -> f32 { 80.0 }
fn default_wait_time() -> f32 { 1.0 }
fn default_score() -> u32 { 10 }
fn default_detection_range() -> f32 { 200.0 }
fn default_chase_speed() -> f32 { 80.0 }
fn default_lose_range() -> f32 { 300.0 }
fn default_true() -> bool { true }

// Note: Component trait is implemented via blanket impl in component.rs
// for all types that implement Any + Send + Sync

/// Runtime state for behaviors (not serialized in scene files).
///
/// This component stores transient state that behaviors need during execution,
/// such as timers and flags. It's automatically added by the BehaviorRunner.
#[derive(Debug, Clone, Default)]
pub struct BehaviorState {
    /// General-purpose timer (used for jump cooldown, wait time, etc.)
    pub timer: f32,
    /// Patrol direction (false = toward A, true = toward B)
    pub patrol_toward_b: bool,
    /// Whether currently chasing (for ChasePlayer behavior)
    pub is_chasing: bool,
    /// Whether waiting at a patrol point
    pub is_waiting: bool,
}

/// Marker component to identify player-controlled entities.
///
/// Used by AI behaviors (like ChasePlayer) to find the player.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerTag;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_serialization() {
        let behavior = Behavior::PlayerPlatformer {
            move_speed: 150.0,
            jump_impulse: 500.0,
            jump_cooldown: 0.25,
        };

        let serialized = ron::to_string(&behavior).expect("Failed to serialize");
        let deserialized: Behavior = ron::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            Behavior::PlayerPlatformer {
                move_speed,
                jump_impulse,
                jump_cooldown,
            } => {
                assert_eq!(move_speed, 150.0);
                assert_eq!(jump_impulse, 500.0);
                assert_eq!(jump_cooldown, 0.25);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_behavior_defaults() {
        let ron_str = "PlayerPlatformer()";
        let behavior: Behavior = ron::from_str(ron_str).expect("Failed to parse");

        match behavior {
            Behavior::PlayerPlatformer {
                move_speed,
                jump_impulse,
                jump_cooldown,
            } => {
                assert_eq!(move_speed, 120.0);
                assert_eq!(jump_impulse, 420.0);
                assert_eq!(jump_cooldown, 0.3);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_behavior_state_default() {
        let state = BehaviorState::default();
        assert_eq!(state.timer, 0.0);
        assert!(!state.patrol_toward_b);
        assert!(!state.is_chasing);
        assert!(!state.is_waiting);
    }
}
