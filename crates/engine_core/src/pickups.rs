//! Generic pickup / collectible tracking shared by games.
//!
//! Games spawn pickup entities however they like (floating sensors in Pong,
//! falling capsules in Breakout), `track` them here with a game-defined kind,
//! and each frame call [`Pickups::collect`] with the frame's drained
//! collision events (`physics.take_collision_events()`, taken once and
//! shared) and the set of collector entities. The engine owns the mechanism — collection
//! detection, once-per-pickup semantics, despawn bookkeeping — while games
//! own the meaning: what kinds exist, what effects they grant, how pickups
//! move and look. (Same split as `InputMapping<A>` and `ChaosMode`.)
//!
//! [`EffectTimer`] is the companion countdown for timed pickup effects.

use ecs::{EntityId, World};
use physics::{CollisionData, PhysicsSystem};

/// A live pickup entity with its game-defined kind.
#[derive(Debug, Clone, Copy)]
pub struct Pickup<K> {
    /// The pickup's ECS entity (sprite + sensor collider, spawned by the game)
    pub entity: EntityId,
    /// Game-defined kind (what catching it grants)
    pub kind: K,
}

/// Tracks live pickups and resolves collection against collision events.
#[derive(Debug)]
pub struct Pickups<K> {
    items: Vec<Pickup<K>>,
}

// Manual impl: `#[derive(Default)]` would needlessly require `K: Default`.
impl<K> Default for Pickups<K> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<K: Copy> Pickups<K> {
    /// Create an empty pickup set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a spawned pickup entity.
    pub fn track(&mut self, entity: EntityId, kind: K) {
        self.items.push(Pickup { entity, kind });
    }

    /// Number of live pickups.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether no pickups are live.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Entities of all live pickups (for visibility toggles etc.).
    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.items.iter().map(|p| p.entity)
    }

    /// Resolve collection: any pickup with a `started` collision against one
    /// of `collectors` is collected — removed from tracking and destroyed via
    /// [`PhysicsSystem::destroy_entity`]. Each pickup is collected **at most
    /// once per call** (the first matching collector wins), even if several
    /// collectors touch it in the same frame.
    ///
    /// Returns the collected `(kind, collector)` pairs.
    pub fn collect(
        &mut self,
        collisions: &[CollisionData],
        collectors: &[EntityId],
        physics: &mut PhysicsSystem,
        world: &mut World,
    ) -> Vec<(K, EntityId)> {
        let mut collected: Vec<(usize, K, EntityId)> = Vec::new();
        for (i, pickup) in self.items.iter().enumerate() {
            if collected.iter().any(|&(j, _, _)| j == i) {
                continue;
            }
            let hit = collisions.iter().find_map(|c| {
                if !c.event.started {
                    return None;
                }
                collectors
                    .iter()
                    .copied()
                    .find(|&col| c.event.involves(col, pickup.entity))
            });
            if let Some(collector) = hit {
                collected.push((i, pickup.kind, collector));
            }
        }

        // Remove in descending index order so earlier indices stay valid.
        for &(i, _, _) in collected.iter().rev() {
            let pickup = self.items.remove(i);
            physics.destroy_entity(world, pickup.entity);
        }

        collected.into_iter().map(|(_, kind, by)| (kind, by)).collect()
    }

    /// Remove and destroy every pickup matching `pred` (missed / expired).
    pub fn remove_where(
        &mut self,
        physics: &mut PhysicsSystem,
        world: &mut World,
        mut pred: impl FnMut(&Pickup<K>) -> bool,
    ) {
        let mut i = 0;
        while i < self.items.len() {
            if pred(&self.items[i]) {
                let pickup = self.items.remove(i);
                physics.destroy_entity(world, pickup.entity);
            } else {
                i += 1;
            }
        }
    }

    /// Remove and destroy all live pickups (match end / reset).
    pub fn clear(&mut self, physics: &mut PhysicsSystem, world: &mut World) {
        for pickup in self.items.drain(..) {
            physics.destroy_entity(world, pickup.entity);
        }
    }
}

/// Countdown for a timed pickup effect (speed boost, wrecking ball, ...).
///
/// `start` sets (or refreshes) the duration; `tick` counts down and returns
/// `true` exactly on the frame the timer crosses zero, so callers can revert
/// the effect's visuals once.
#[derive(Debug, Clone, Copy, Default)]
pub struct EffectTimer {
    remaining: f32,
}

impl EffectTimer {
    /// Start the effect, or refresh it to the full duration if already active.
    pub fn start(&mut self, duration: f32) {
        self.remaining = duration.max(0.0);
    }

    /// Deactivate immediately. `tick` will NOT report expiry after this —
    /// use it when the caller reverts effect state itself (match reset).
    pub fn stop(&mut self) {
        self.remaining = 0.0;
    }

    /// Whether the effect is currently active.
    pub fn active(&self) -> bool {
        self.remaining > 0.0
    }

    /// Seconds left (0 when inactive).
    pub fn remaining(&self) -> f32 {
        self.remaining
    }

    /// Count down by `dt`. Returns `true` exactly once: on the tick where the
    /// timer crosses from active to expired.
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.remaining <= 0.0 {
            return false;
        }
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::sprite_components::Transform2D;
    use ecs::System;
    use glam::Vec2;
    use physics::{Collider, CollisionEvent, PhysicsConfig, RigidBody};

    fn started_event(a: EntityId, b: EntityId) -> CollisionData {
        CollisionData {
            event: CollisionEvent { entity_a: a, entity_b: b, started: true, stopped: false },
            contacts: vec![],
        }
    }

    fn stopped_event(a: EntityId, b: EntityId) -> CollisionData {
        CollisionData {
            event: CollisionEvent { entity_a: a, entity_b: b, started: false, stopped: true },
            contacts: vec![],
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Boost,
        Multi,
    }

    #[test]
    fn test_collect_ignores_non_started_events() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let pickup = world.create_entity();
        let collector = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(pickup, Kind::Boost);

        let events = vec![stopped_event(collector, pickup)];
        let got = pickups.collect(&events, &[collector], &mut physics, &mut world);
        assert!(got.is_empty());
        assert_eq!(pickups.len(), 1);
    }

    #[test]
    fn test_collect_returns_kind_and_collector_and_untracks() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let pickup = world.create_entity();
        let collector = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(pickup, Kind::Multi);

        let events = vec![started_event(pickup, collector)];
        let got = pickups.collect(&events, &[collector], &mut physics, &mut world);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, Kind::Multi);
        assert_eq!(got[0].1, collector);
        assert!(pickups.is_empty());
        // The pickup entity is destroyed from the world too
        assert!(!world.entities().contains(&pickup));
    }

    #[test]
    fn test_pickup_collected_once_even_with_two_collectors() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let pickup = world.create_entity();
        let ball_a = world.create_entity();
        let ball_b = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(pickup, Kind::Boost);

        // Both collectors hit the same pickup in one frame (the Pong
        // double-apply bug this module exists to prevent).
        let events = vec![started_event(ball_a, pickup), started_event(ball_b, pickup)];
        let got = pickups.collect(&events, &[ball_a, ball_b], &mut physics, &mut world);
        assert_eq!(got.len(), 1, "one pickup must grant exactly one effect");
        assert!(pickups.is_empty());
    }

    #[test]
    fn test_multiple_pickups_collected_in_one_frame() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let p1 = world.create_entity();
        let p2 = world.create_entity();
        let p3 = world.create_entity();
        let collector = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(p1, Kind::Boost);
        pickups.track(p2, Kind::Multi);
        pickups.track(p3, Kind::Boost);

        // p1 and p3 hit — non-adjacent indices (0 and 2), p2 survives.
        let events = vec![started_event(collector, p3), started_event(collector, p1)];
        let got = pickups.collect(&events, &[collector], &mut physics, &mut world);
        assert_eq!(got.len(), 2);
        assert_eq!(pickups.len(), 1);
        assert_eq!(pickups.entities().next(), Some(p2));
    }

    #[test]
    fn test_remove_where_destroys_matching() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let keep = world.create_entity();
        let drop = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(keep, Kind::Boost);
        pickups.track(drop, Kind::Multi);

        pickups.remove_where(&mut physics, &mut world, |p| p.kind == Kind::Multi);
        assert_eq!(pickups.len(), 1);
        assert!(!world.entities().contains(&drop));
        assert!(world.entities().contains(&keep));
    }

    #[test]
    fn test_clear_empties_and_destroys() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::new();
        let p1 = world.create_entity();
        let p2 = world.create_entity();

        let mut pickups = Pickups::new();
        pickups.track(p1, Kind::Boost);
        pickups.track(p2, Kind::Multi);

        pickups.clear(&mut physics, &mut world);
        assert!(pickups.is_empty());
        assert!(!world.entities().contains(&p1));
        assert!(!world.entities().contains(&p2));
    }

    #[test]
    fn test_effect_timer_lifecycle() {
        let mut t = EffectTimer::default();
        assert!(!t.active());
        assert_eq!(t.remaining(), 0.0);
        assert!(!t.tick(1.0), "inactive timer never fires expiry");

        t.start(1.0);
        assert!(t.active());
        assert!(!t.tick(0.4));
        assert!((t.remaining() - 0.6).abs() < 1e-6);
        assert!(t.tick(0.6), "must fire exactly when crossing zero");
        assert!(!t.active());
        assert!(!t.tick(0.1), "must not fire again after expiry");
    }

    #[test]
    fn test_effect_timer_start_refreshes_while_active() {
        let mut t = EffectTimer::default();
        t.start(1.0);
        t.tick(0.9);
        t.start(1.0); // caught a second pickup — refresh, no stacking
        assert!((t.remaining() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_effect_timer_stop_deactivates_without_expiry_event() {
        let mut t = EffectTimer::default();
        t.start(5.0);
        t.stop();
        assert!(!t.active());
        assert!(!t.tick(0.1), "stop() means the caller reverted state itself");
    }

    /// End-to-end catch mechanics: a falling dynamic sensor pickup crossing a
    /// kinematic collector emits a started event that `collect` resolves.
    /// This is the DYNAMIC↔KINEMATIC sensor-event proof the games rely on.
    #[test]
    fn test_falling_sensor_pickup_collected_by_kinematic_body() {
        let mut world = World::new();
        let mut physics = PhysicsSystem::with_config(PhysicsConfig::top_down());

        let paddle = world
            .spawn()
            .with(Transform2D::new(Vec2::new(0.0, 0.0)))
            .with(RigidBody::new_kinematic())
            .with(Collider::box_collider(110.0, 16.0).with_friction(0.0))
            .id();

        let pickup_entity = world
            .spawn()
            .with(Transform2D::new(Vec2::new(0.0, 60.0)))
            .with(
                RigidBody::new_dynamic()
                    .with_gravity_scale(0.0)
                    .with_rotation_locked(true),
            )
            .with(Collider::box_collider(18.0, 18.0).as_sensor())
            .id();
        physics.set_velocity(pickup_entity, Vec2::new(0.0, -180.0), 0.0);

        let mut pickups = Pickups::new();
        pickups.track(pickup_entity, Kind::Multi);

        let mut got = Vec::new();
        for _ in 0..60 {
            physics.update(&mut world, 1.0 / 60.0);
            let events = physics.take_collision_events();
            got.extend(pickups.collect(&events, &[paddle], &mut physics, &mut world));
            if !got.is_empty() {
                break;
            }
        }

        assert_eq!(got.len(), 1, "falling sensor pickup must be caught by the kinematic body");
        assert_eq!(got[0].0, Kind::Multi);
        assert_eq!(got[0].1, paddle);
        assert!(pickups.is_empty());
    }
}
