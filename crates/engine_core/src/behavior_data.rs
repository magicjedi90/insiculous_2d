//! Serialization mirror of `ecs::behavior::Behavior` for scene files.
//!
//! Part of the scene schema (`ComponentData::Behavior` wraps this). Kept in
//! its own module for file-size reasons; `scene_data.rs` re-exports it, so
//! `scene_data::BehaviorData` remains the canonical import path.

use serde::{Deserialize, Serialize};

/// Behavior data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorData {
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
        /// Tag to identify this entity for targeting
        #[serde(default = "default_player_tag")]
        tag: String,
    },
    /// Player-controlled top-down movement (WASD in all directions)
    PlayerTopDown {
        /// Movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
        /// Tag to identify this entity for targeting
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
    /// Smoothly move this entity toward the nearest entity with a tag
    /// (intended for camera entities)
    CameraFollow {
        /// Tag of the entity to follow
        #[serde(default = "default_player_tag")]
        target_tag: String,
        /// Fraction of the remaining distance covered per frame at 60 FPS (0.0–1.0)
        #[serde(default = "default_lerp_speed")]
        lerp_speed: f32,
        /// Fixed offset from the target position (x, y)
        #[serde(default)]
        offset: (f32, f32),
        /// Optional dead zone (full width, full height) centered on this entity
        #[serde(default)]
        dead_zone: Option<(f32, f32)>,
    },
}

/// Convert scene serialization data to the ECS component (load direction).
///
/// This pair of `From` impls is the single source of truth for
/// `Behavior` ↔ `BehaviorData` conversion — both the scene loader and the
/// scene serializer go through here. When adding a behavior variant, extend
/// both impls and nothing else.
impl From<&BehaviorData> for ecs::behavior::Behavior {
    fn from(data: &BehaviorData) -> Self {
        match data {
            BehaviorData::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
                Self::PlayerPlatformer { move_speed: *move_speed, jump_impulse: *jump_impulse, jump_cooldown: *jump_cooldown, tag: tag.clone() }
            }
            BehaviorData::PlayerTopDown { move_speed, tag } => {
                Self::PlayerTopDown { move_speed: *move_speed, tag: tag.clone() }
            }
            BehaviorData::FollowEntity { target_name, follow_distance, follow_speed } => {
                Self::FollowEntity { target_name: target_name.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            BehaviorData::FollowTagged { target_tag, follow_distance, follow_speed } => {
                Self::FollowTagged { target_tag: target_tag.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            BehaviorData::Patrol { point_a, point_b, speed, wait_time } => {
                Self::Patrol { point_a: *point_a, point_b: *point_b, speed: *speed, wait_time: *wait_time }
            }
            BehaviorData::Collectible { score_value, despawn_on_collect, collector_tag } => {
                Self::Collectible { score_value: *score_value, despawn_on_collect: *despawn_on_collect, collector_tag: collector_tag.clone() }
            }
            BehaviorData::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
                Self::ChaseTagged { target_tag: target_tag.clone(), detection_range: *detection_range, chase_speed: *chase_speed, lose_interest_range: *lose_interest_range }
            }
            BehaviorData::CameraFollow { target_tag, lerp_speed, offset, dead_zone } => {
                Self::CameraFollow { target_tag: target_tag.clone(), lerp_speed: *lerp_speed, offset: *offset, dead_zone: *dead_zone }
            }
        }
    }
}

/// Convert the ECS component to scene serialization data (save direction).
impl From<&ecs::behavior::Behavior> for BehaviorData {
    fn from(behavior: &ecs::behavior::Behavior) -> Self {
        use ecs::behavior::Behavior;
        match behavior {
            Behavior::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
                Self::PlayerPlatformer { move_speed: *move_speed, jump_impulse: *jump_impulse, jump_cooldown: *jump_cooldown, tag: tag.clone() }
            }
            Behavior::PlayerTopDown { move_speed, tag } => {
                Self::PlayerTopDown { move_speed: *move_speed, tag: tag.clone() }
            }
            Behavior::FollowEntity { target_name, follow_distance, follow_speed } => {
                Self::FollowEntity { target_name: target_name.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            Behavior::FollowTagged { target_tag, follow_distance, follow_speed } => {
                Self::FollowTagged { target_tag: target_tag.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            Behavior::Patrol { point_a, point_b, speed, wait_time } => {
                Self::Patrol { point_a: *point_a, point_b: *point_b, speed: *speed, wait_time: *wait_time }
            }
            Behavior::Collectible { score_value, despawn_on_collect, collector_tag } => {
                Self::Collectible { score_value: *score_value, despawn_on_collect: *despawn_on_collect, collector_tag: collector_tag.clone() }
            }
            Behavior::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
                Self::ChaseTagged { target_tag: target_tag.clone(), detection_range: *detection_range, chase_speed: *chase_speed, lose_interest_range: *lose_interest_range }
            }
            Behavior::CameraFollow { target_tag, lerp_speed, offset, dead_zone } => {
                Self::CameraFollow { target_tag: target_tag.clone(), lerp_speed: *lerp_speed, offset: *offset, dead_zone: *dead_zone }
            }
        }
    }
}

// Behavior default value functions
fn default_move_speed() -> f32 {
    120.0
}
fn default_jump_impulse() -> f32 {
    420.0
}
fn default_jump_cooldown() -> f32 {
    0.3
}
fn default_follow_distance() -> f32 {
    50.0
}
fn default_follow_speed() -> f32 {
    100.0
}
fn default_patrol_speed() -> f32 {
    80.0
}
fn default_wait_time() -> f32 {
    1.0
}
fn default_score() -> u32 {
    10
}
fn default_detection_range() -> f32 {
    200.0
}
fn default_chase_speed() -> f32 {
    80.0
}
fn default_lose_range() -> f32 {
    300.0
}
fn default_player_tag() -> String {
    "player".to_string()
}
fn default_lerp_speed() -> f32 {
    0.1
}
fn default_true() -> bool {
    true
}
