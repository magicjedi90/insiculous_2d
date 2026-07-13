//! Auto-despawn after a duration (PROJECT_ROADMAP Phase B, Gap 2).
//!
//! Attach [`Lifetime`] to bullets, effects, and debris; [`LifetimeSystem`]
//! counts every instance down and removes the entity the frame its time
//! crosses zero — no per-entity timer bookkeeping in game code.
//!
//! Removal goes through `world.remove_entity`, so hierarchy links detach
//! automatically and `PhysicsSystem` garbage-collects any rapier state on
//! its next update.

use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::query::Single;
use crate::system::System;
use crate::world::World;

/// Component: despawn the owning entity after `remaining` seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lifetime {
    /// Seconds until despawn.
    pub remaining: f32,
}

impl Lifetime {
    /// Despawn after `seconds`.
    pub fn new(seconds: f32) -> Self {
        Self { remaining: seconds }
    }
}

/// System: ticks every [`Lifetime`] down by the frame delta and removes
/// entities whose time expired. Add it to the world's system registry or
/// own an instance and call `update` from the game loop.
#[derive(Debug, Default)]
pub struct LifetimeSystem;

impl LifetimeSystem {
    /// Create a new lifetime system.
    pub fn new() -> Self {
        Self
    }
}

impl System for LifetimeSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // query_entities returns an owned Vec, so removal during the loop
        // can't invalidate the iteration.
        let carriers: Vec<EntityId> = world.query_entities::<Single<Lifetime>>();
        for entity in carriers {
            let expired = world
                .get_mut::<Lifetime>(entity)
                .map(|l| {
                    l.remaining -= delta_time;
                    l.remaining <= 0.0
                })
                .unwrap_or(false);
            if expired {
                world.remove_entity(&entity).ok();
            }
        }
    }

    fn name(&self) -> &str {
        "LifetimeSystem"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alive(world: &World, e: EntityId) -> bool {
        world.get::<Lifetime>(e).is_some()
    }

    #[test]
    fn test_entity_despawns_when_lifetime_crosses_zero() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(&e, Lifetime::new(0.5)).unwrap();

        let mut system = LifetimeSystem::new();
        // 25 frames at 16ms = 0.4s: still alive.
        for _ in 0..25 {
            system.update(&mut world, 0.016);
        }
        assert!(alive(&world, e), "entity must survive until its time is up (t=0.4)");

        // ~0.6s total: gone.
        for _ in 0..13 {
            system.update(&mut world, 0.016);
        }
        assert!(!alive(&world, e), "entity must be despawned after 0.5s (t~0.6)");
        assert!(!world.entities().contains(&e), "entity removed from the world entirely");
    }

    #[test]
    fn test_expiry_removes_exactly_once_and_further_updates_are_safe() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(&e, Lifetime::new(0.01)).unwrap();

        let mut system = LifetimeSystem::new();
        system.update(&mut world, 0.02); // expires
        assert!(!alive(&world, e));
        system.update(&mut world, 0.02); // must not panic or resurrect anything
        assert!(!world.entities().contains(&e));
    }

    #[test]
    fn test_lifetimes_tick_independently() {
        let mut world = World::new();
        let short = world.create_entity();
        world.add_component(&short, Lifetime::new(0.05)).unwrap();
        let long = world.create_entity();
        world.add_component(&long, Lifetime::new(10.0)).unwrap();

        let mut system = LifetimeSystem::new();
        system.update(&mut world, 0.1);
        assert!(!alive(&world, short), "short lifetime expired");
        assert!(alive(&world, long), "long lifetime unaffected");
        assert!((world.get::<Lifetime>(long).unwrap().remaining - 9.9).abs() < 1e-4);
    }

    #[test]
    fn test_entities_without_lifetime_are_untouched() {
        let mut world = World::new();
        let immortal = world.create_entity();

        let mut system = LifetimeSystem::new();
        for _ in 0..100 {
            system.update(&mut world, 1.0);
        }
        assert!(world.entities().contains(&immortal));
    }
}
