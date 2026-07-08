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
        /// Tag to identify this entity (for targeting by other behaviors)
        #[serde(default = "default_player_tag")]
        tag: String,
    },

    /// Player-controlled top-down movement (WASD in all directions)
    PlayerTopDown {
        /// Movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
        /// Tag to identify this entity (for targeting by other behaviors)
        #[serde(default = "default_player_tag")]
        tag: String,
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

    /// Follow the nearest entity with a specific tag
    FollowTagged {
        /// Tag of entities to follow
        #[serde(default = "default_player_tag")]
        target_tag: String,
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

    /// Collectible item that can be picked up by entities with a specific tag
    Collectible {
        /// Score value when collected
        #[serde(default = "default_score")]
        score_value: u32,
        /// Whether to despawn when collected
        #[serde(default = "default_true")]
        despawn_on_collect: bool,
        /// Tag of entities that can collect this item
        #[serde(default = "default_player_tag")]
        collector_tag: String,
    },

    /// AI that chases entities with a specific tag when in range
    ChaseTagged {
        /// Tag of entities to chase
        #[serde(default = "default_player_tag")]
        target_tag: String,
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
fn default_player_tag() -> String { "player".to_string() }

impl Default for Behavior {
    fn default() -> Self {
        Behavior::PlayerPlatformer {
            move_speed: default_move_speed(),
            jump_impulse: default_jump_impulse(),
            jump_cooldown: default_jump_cooldown(),
            tag: default_player_tag(),
        }
    }
}

impl Behavior {
    /// Variant names in declaration order (indices match `variant_index`).
    pub const VARIANT_NAMES: &'static [&'static str] = &[
        "PlayerPlatformer",
        "PlayerTopDown",
        "FollowEntity",
        "FollowTagged",
        "Patrol",
        "Collectible",
        "ChaseTagged",
    ];

    /// Display name of this behavior's variant
    pub fn variant_name(&self) -> &'static str {
        Self::VARIANT_NAMES[self.variant_index()]
    }

    /// Index of this behavior's variant within `VARIANT_NAMES`
    pub fn variant_index(&self) -> usize {
        match self {
            Behavior::PlayerPlatformer { .. } => 0,
            Behavior::PlayerTopDown { .. } => 1,
            Behavior::FollowEntity { .. } => 2,
            Behavior::FollowTagged { .. } => 3,
            Behavior::Patrol { .. } => 4,
            Behavior::Collectible { .. } => 5,
            Behavior::ChaseTagged { .. } => 6,
        }
    }

    /// Build a behavior of the given variant index with default field values.
    ///
    /// Indices wrap around, so `default_for_variant(i % VARIANT_NAMES.len())`
    /// callers may pass any index produced by cycling forward or backward.
    pub fn default_for_variant(index: usize) -> Behavior {
        match index % Self::VARIANT_NAMES.len() {
            0 => Behavior::default(),
            1 => Behavior::PlayerTopDown {
                move_speed: default_move_speed(),
                tag: default_player_tag(),
            },
            2 => Behavior::FollowEntity {
                target_name: String::new(),
                follow_distance: default_follow_distance(),
                follow_speed: default_follow_speed(),
            },
            3 => Behavior::FollowTagged {
                target_tag: default_player_tag(),
                follow_distance: default_follow_distance(),
                follow_speed: default_follow_speed(),
            },
            4 => Behavior::Patrol {
                point_a: (0.0, 0.0),
                point_b: (100.0, 0.0),
                speed: default_patrol_speed(),
                wait_time: default_wait_time(),
            },
            5 => Behavior::Collectible {
                score_value: default_score(),
                despawn_on_collect: default_true(),
                collector_tag: default_player_tag(),
            },
            _ => Behavior::ChaseTagged {
                target_tag: default_player_tag(),
                detection_range: default_detection_range(),
                chase_speed: default_chase_speed(),
                lose_interest_range: default_lose_range(),
            },
        }
    }
}

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

/// Tag component for entity identification.
///
/// Used by behaviors to identify and target entities dynamically.
/// For example, player behaviors add an EntityTag("player"), and
/// ChaseTagged behaviors can target any tag like "player", "enemy", "ally", etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityTag(pub String);

impl EntityTag {
    /// Create a new entity tag
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    /// Check if this tag matches a given string
    pub fn matches(&self, tag: &str) -> bool {
        self.0 == tag
    }
}

impl Default for EntityTag {
    fn default() -> Self {
        Self("player".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_serialization() {
        let behavior = Behavior::PlayerPlatformer {
            move_speed: 150.0,
            jump_impulse: 500.0,
            jump_cooldown: 0.25,
            tag: "hero".to_string(),
        };

        let serialized = ron::to_string(&behavior).expect("Failed to serialize");
        let deserialized: Behavior = ron::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            Behavior::PlayerPlatformer {
                move_speed,
                jump_impulse,
                jump_cooldown,
                tag,
            } => {
                assert_eq!(move_speed, 150.0);
                assert_eq!(jump_impulse, 500.0);
                assert_eq!(jump_cooldown, 0.25);
                assert_eq!(tag, "hero");
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
                tag,
            } => {
                assert_eq!(move_speed, 120.0);
                assert_eq!(jump_impulse, 420.0);
                assert_eq!(jump_cooldown, 0.3);
                assert_eq!(tag, "player"); // Default tag
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

    #[test]
    fn test_entity_tag() {
        let tag = EntityTag::new("enemy");
        assert!(tag.matches("enemy"));
        assert!(!tag.matches("player"));
        assert_eq!(tag.0, "enemy");
    }

    #[test]
    fn test_behavior_default_is_player_platformer_with_serde_defaults() {
        match Behavior::default() {
            Behavior::PlayerPlatformer {
                move_speed,
                jump_impulse,
                jump_cooldown,
                tag,
            } => {
                assert_eq!(move_speed, 120.0);
                assert_eq!(jump_impulse, 420.0);
                assert_eq!(jump_cooldown, 0.3);
                assert_eq!(tag, "player");
            }
            _ => panic!("Default must be PlayerPlatformer"),
        }
    }

    #[test]
    fn test_default_for_variant_round_trips_variant_index() {
        for index in 0..Behavior::VARIANT_NAMES.len() {
            let behavior = Behavior::default_for_variant(index);
            assert_eq!(behavior.variant_index(), index);
            assert_eq!(behavior.variant_name(), Behavior::VARIANT_NAMES[index]);
        }
    }

    #[test]
    fn test_default_for_variant_wraps_out_of_range_indices() {
        let count = Behavior::VARIANT_NAMES.len();
        assert_eq!(count, 7);
        assert_eq!(Behavior::default_for_variant(count).variant_index(), 0);
        assert_eq!(
            Behavior::default_for_variant(count + 2).variant_index(),
            2
        );
    }

    #[test]
    fn test_chase_tagged_serialization() {
        let behavior = Behavior::ChaseTagged {
            target_tag: "hero".to_string(),
            detection_range: 150.0,
            chase_speed: 100.0,
            lose_interest_range: 250.0,
        };

        let serialized = ron::to_string(&behavior).expect("Failed to serialize");
        let deserialized: Behavior = ron::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            Behavior::ChaseTagged { target_tag, .. } => {
                assert_eq!(target_tag, "hero");
            }
            _ => panic!("Wrong variant"),
        }
    }
}
