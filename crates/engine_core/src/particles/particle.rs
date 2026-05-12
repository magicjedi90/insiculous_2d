//! Particle POD struct + [`ParticleConfig`] builder.

use glam::{Vec2, Vec4};

/// A single live particle. Plain old data — no allocations, fully Copy.
///
/// The `alive` flag marks pool slots; dead particles aren't iterated by the
/// renderer or the simulation step.
#[derive(Clone, Copy, Debug, Default)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    /// Acceleration applied every step. Use it for gravity or buoyancy.
    pub acceleration: Vec2,
    /// Color at age 0.
    pub color_start: Vec4,
    /// Color at age = lifetime.
    pub color_end: Vec4,
    pub scale_start: f32,
    pub scale_end: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    /// Multiplicative drag per second (e.g. 0.5 means velocity halves each
    /// second, exponentially). 0.0 = no drag.
    pub drag: f32,
    pub age: f32,
    pub lifetime: f32,
    /// Bloom hook — values > 0 push the sprite into HDR and glow.
    pub emissive: f32,
    pub texture: u32,
    pub alive: bool,
}

impl Particle {
    /// Interpolation parameter `t` in [0, 1] across the particle's lifetime.
    pub fn t(&self) -> f32 {
        if self.lifetime <= 0.0 { 1.0 } else { (self.age / self.lifetime).clamp(0.0, 1.0) }
    }
}

/// Configuration for one burst of particles or for a continuous emitter.
///
/// Build with [`ParticleConfig::burst`] then chain `with_*` methods.
#[derive(Clone, Debug)]
pub struct ParticleConfig {
    /// Number of particles to spawn per burst (or per `emission_rate` second
    /// of continuous emission, where one tick = one particle).
    pub count: usize,
    /// Particle lifetime range in seconds. Each particle picks a random value
    /// uniformly in this range.
    pub lifetime_range: (f32, f32),
    /// Speed range in world units per second.
    pub speed_range: (f32, f32),
    /// Mean direction. Normalized internally.
    pub direction: Vec2,
    /// Half-cone angle around `direction` in radians. PI = full omnidirectional.
    pub spread_radians: f32,
    pub color_start: Vec4,
    pub color_end: Vec4,
    pub scale_start: f32,
    pub scale_end: f32,
    pub angular_velocity_range: (f32, f32),
    pub gravity: Vec2,
    /// Exponential drag — see [`Particle::drag`].
    pub drag: f32,
    pub emissive: f32,
    pub texture: u32,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            count: 16,
            lifetime_range: (0.5, 1.0),
            speed_range: (100.0, 200.0),
            direction: Vec2::Y,
            spread_radians: std::f32::consts::PI,
            color_start: Vec4::new(1.0, 1.0, 1.0, 1.0),
            color_end: Vec4::new(1.0, 1.0, 1.0, 0.0),
            scale_start: 8.0,
            scale_end: 0.0,
            angular_velocity_range: (-3.0, 3.0),
            gravity: Vec2::ZERO,
            drag: 0.0,
            emissive: 1.5,
            texture: 0,
        }
    }
}

impl ParticleConfig {
    /// Start with a sensible default omnidirectional burst of `count` particles.
    pub fn burst(count: usize) -> Self {
        Self { count, ..Default::default() }
    }

    pub fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime_range = (min.min(max), min.max(max));
        self
    }

    pub fn with_speed(mut self, min: f32, max: f32) -> Self {
        self.speed_range = (min.min(max), min.max(max));
        self
    }

    /// `dir` should be roughly unit length; the constructor will normalize.
    /// `spread_radians = 0` shoots a single line; `PI` is omnidirectional.
    pub fn with_direction(mut self, dir: Vec2, spread_radians: f32) -> Self {
        self.direction = if dir.length_squared() > 0.0 { dir.normalize() } else { Vec2::Y };
        self.spread_radians = spread_radians;
        self
    }

    pub fn with_color(mut self, start: Vec4, end: Vec4) -> Self {
        self.color_start = start;
        self.color_end = end;
        self
    }

    pub fn with_scale(mut self, start: f32, end: f32) -> Self {
        self.scale_start = start;
        self.scale_end = end;
        self
    }

    pub fn with_angular_velocity(mut self, min: f32, max: f32) -> Self {
        self.angular_velocity_range = (min.min(max), min.max(max));
        self
    }

    pub fn with_gravity(mut self, gravity: Vec2) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    pub fn with_emissive(mut self, emissive: f32) -> Self {
        self.emissive = emissive;
        self
    }

    pub fn with_texture(mut self, texture: u32) -> Self {
        self.texture = texture;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_chains() {
        let cfg = ParticleConfig::burst(32)
            .with_lifetime(0.2, 0.8)
            .with_speed(50.0, 300.0)
            .with_direction(Vec2::new(1.0, 0.0), 0.3)
            .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_emissive(3.0);
        assert_eq!(cfg.count, 32);
        assert!((cfg.lifetime_range.0 - 0.2).abs() < 1e-5);
        assert!((cfg.lifetime_range.1 - 0.8).abs() < 1e-5);
        assert!((cfg.speed_range.1 - 300.0).abs() < 1e-3);
        assert!((cfg.direction.length() - 1.0).abs() < 1e-5);
        assert!((cfg.emissive - 3.0).abs() < 1e-5);
    }

    #[test]
    fn with_lifetime_swaps_min_max() {
        // Passing args backwards still produces a valid (min, max) range.
        let cfg = ParticleConfig::burst(1).with_lifetime(0.9, 0.1);
        assert!(cfg.lifetime_range.0 <= cfg.lifetime_range.1);
    }

    #[test]
    fn particle_t_clamped() {
        let mut p = Particle { lifetime: 1.0, age: 0.5, ..Default::default() };
        assert!((p.t() - 0.5).abs() < 1e-5);
        p.age = 2.0; // past end of life
        assert!((p.t() - 1.0).abs() < 1e-5);
        p.lifetime = 0.0;
        assert_eq!(p.t(), 1.0);
    }
}
