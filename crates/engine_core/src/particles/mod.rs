//! Particle system.
//!
//! `ParticleManager` is a flat pool of CPU-simulated particles. Games either
//! call [`ParticleManager::spawn_burst`] directly from update code (typical
//! for one-shot effects like explosions on collision) or attach a
//! [`ParticleEmitter`] component to an entity for continuous emission.
//!
//! Each frame [`ParticleSystem::update`] advances every alive particle and
//! drives any emitter components.
//!
//! Particles render via the existing sprite pipeline — they are appended to
//! the `SpriteBatcher` in `Game::render()` and pick up bloom for free when
//! their `emissive` value is nonzero.

mod emitter;
mod manager;
mod particle;
mod system;

pub use emitter::ParticleEmitter;
pub use manager::ParticleManager;
pub use particle::{Particle, ParticleConfig};
pub use system::ParticleSystem;
