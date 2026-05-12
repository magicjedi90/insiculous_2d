//! System that drives [`ParticleEmitter`] components and steps the
//! [`ParticleManager`] pool each frame.

use common::Transform2D;
use ecs::World;

use super::emitter::ParticleEmitter;
use super::manager::ParticleManager;

/// Stateless façade for the per-frame particle update.
///
/// Call [`ParticleSystem::update`] once per frame from the engine loop,
/// after physics so emitter positions reflect the latest transforms.
pub struct ParticleSystem;

impl ParticleSystem {
    /// Drive every `ParticleEmitter` component, then advance the pool.
    ///
    /// Behavioral order:
    /// 1. For each entity with `ParticleEmitter + Transform2D`, accumulate
    ///    elapsed time and spawn particles into `manager`.
    /// 2. Step the manager so every alive particle ages by `dt`.
    pub fn update(world: &mut World, manager: &mut ParticleManager, dt: f32) {
        // Collect emit commands during iteration; spawn afterwards so the
        // mutable borrow on `world` doesn't overlap. Each command is the
        // entity's position plus the count of particles to spawn.
        //
        // `Vec` is allocated here on purpose — emitter counts are tiny
        // (typically zero or one entity in a Pong-scale game) so this is
        // <100 bytes total. For high-emitter scenes, swap this for a
        // reusable per-manager scratch buffer.
        let mut emit_queue: Vec<EmitCommand> = Vec::new();

        for entity in world.entities() {
            let position = match world.get::<Transform2D>(entity) {
                Some(t) => t.position,
                None => continue,
            };
            let Some(emitter) = world.get_mut::<ParticleEmitter>(entity) else { continue };
            if !emitter.active || emitter.emission_rate <= 0.0 {
                continue;
            }
            let Some(config) = emitter.config.as_ref().cloned() else { continue };

            emitter.accumulator += dt * emitter.emission_rate;
            let to_emit = emitter.accumulator.floor() as i32;
            if to_emit > 0 {
                emitter.accumulator -= to_emit as f32;
                emit_queue.push(EmitCommand { position, config, count: to_emit as usize });
            }
        }

        for cmd in emit_queue {
            // Each emitted "tick" produces config.count particles by design —
            // burst mode and continuous mode share one config knob. For
            // single-particle continuous trails, set count=1 in the config.
            for _ in 0..cmd.count {
                manager.spawn_burst(cmd.position, &cmd.config);
            }
        }

        manager.step(dt);
    }
}

struct EmitCommand {
    position: glam::Vec2,
    config: super::particle::ParticleConfig,
    count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particles::particle::ParticleConfig;
    use common::Transform2D;
    use ecs::World;
    use glam::Vec2;

    #[test]
    fn inactive_emitter_emits_nothing() {
        let mut world = World::new();
        let mut manager = ParticleManager::with_capacity(32);
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::default()).ok();
        let mut e = ParticleEmitter::new(100.0, ParticleConfig::burst(1));
        e.pause();
        world.add_component(&entity, e).ok();

        ParticleSystem::update(&mut world, &mut manager, 1.0);
        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn active_emitter_spawns_at_configured_rate() {
        let mut world = World::new();
        let mut manager = ParticleManager::with_capacity(64);
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::default()).ok();
        // 10 emits per second, each emit creates count=1 particle, infinite lifetime.
        let cfg = ParticleConfig::burst(1).with_lifetime(100.0, 100.0);
        world.add_component(&entity, ParticleEmitter::new(10.0, cfg)).ok();

        // Half a second should produce ~5 particles.
        ParticleSystem::update(&mut world, &mut manager, 0.5);
        assert_eq!(manager.alive_count(), 5);
    }

    #[test]
    fn emitter_without_transform_is_ignored() {
        let mut world = World::new();
        let mut manager = ParticleManager::with_capacity(8);
        let entity = world.create_entity();
        let cfg = ParticleConfig::burst(1).with_lifetime(100.0, 100.0);
        world.add_component(&entity, ParticleEmitter::new(100.0, cfg)).ok();

        ParticleSystem::update(&mut world, &mut manager, 0.1);
        assert_eq!(manager.alive_count(), 0);
    }

    #[test]
    fn particles_spawn_at_emitter_position() {
        let mut world = World::new();
        let mut manager = ParticleManager::with_capacity(8);
        let entity = world.create_entity();
        let t = Transform2D { position: Vec2::new(100.0, 50.0), ..Default::default() };
        world.add_component(&entity, t).ok();
        let cfg = ParticleConfig::burst(1)
            .with_speed(0.0, 0.0)
            .with_lifetime(100.0, 100.0);
        world.add_component(&entity, ParticleEmitter::new(100.0, cfg)).ok();

        ParticleSystem::update(&mut world, &mut manager, 0.01);
        // At least one particle should have spawned at the emitter's position.
        let p = manager.iter_alive().next().expect("emitter should produce a particle");
        // Particles spawn at origin and have zero velocity in this config,
        // so position should equal the emitter's transform position (within
        // one dt of drift, but speed = 0).
        assert!((p.position.x - 100.0).abs() < 1.0);
        assert!((p.position.y - 50.0).abs() < 1.0);
    }
}
