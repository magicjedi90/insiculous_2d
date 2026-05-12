//! Flat-pool particle manager.
//!
//! Particles live in a fixed-capacity `Vec<Particle>`. The `alive` flag on
//! each slot marks whether the slot is occupied. Spawn walks the pool from a
//! cursor (round-robin); when the pool is full it overwrites the oldest slot,
//! which is the canonical "ring-buffer pool" pattern used by production
//! particle systems.
//!
//! Zero allocations per frame after construction.

use glam::{Vec2, Vec4};

use super::particle::{Particle, ParticleConfig};

/// Default pool capacity. Pong-scale games rarely need more than this.
pub const DEFAULT_CAPACITY: usize = 16_384;

/// Owns the live particle pool.
///
/// The pool is fixed-size; [`spawn_burst`](Self::spawn_burst) overwrites the
/// oldest slots when more particles are emitted than the capacity holds.
/// Use [`with_capacity`](Self::with_capacity) to size the pool to your needs.
pub struct ParticleManager {
    pool: Vec<Particle>,
    /// Cursor into the pool — the next slot considered for spawning.
    cursor: usize,
    /// Cheap LCG seed for direction / lifetime / speed jitter. Avoids pulling
    /// in `rand` as a workspace dep.
    rng_state: u64,
}

impl Default for ParticleManager {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl ParticleManager {
    /// Build a manager with a custom pool capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            pool: vec![Particle::default(); capacity],
            cursor: 0,
            // Seeded with a non-zero constant so behavior is reproducible
            // across runs unless [`reseed`](Self::reseed) is called.
            rng_state: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Override the deterministic seed — useful for tests.
    pub fn reseed(&mut self, seed: u64) {
        self.rng_state = seed.max(1);
    }

    /// Total slot count, alive + dead.
    pub fn capacity(&self) -> usize {
        self.pool.len()
    }

    /// Number of currently alive particles.
    pub fn alive_count(&self) -> usize {
        self.pool.iter().filter(|p| p.alive).count()
    }

    /// Mark every slot dead.
    pub fn clear(&mut self) {
        for slot in &mut self.pool {
            slot.alive = false;
        }
        self.cursor = 0;
    }

    /// Spawn a burst at `origin` using the parameters in `config`.
    ///
    /// If `config.count` exceeds the number of free slots in the pool, the
    /// oldest slots are overwritten. This is a deliberate trade — better to
    /// drop a few frames of stale dust than to allocate or stutter.
    pub fn spawn_burst(&mut self, origin: Vec2, config: &ParticleConfig) {
        let half_spread = config.spread_radians * 0.5;
        let base_angle = config.direction.y.atan2(config.direction.x);

        for _ in 0..config.count {
            let slot = self.next_slot();
            let lifetime = self.uniform_range(config.lifetime_range.0, config.lifetime_range.1).max(0.001);
            let speed = self.uniform_range(config.speed_range.0, config.speed_range.1);
            let jitter = self.uniform_range(-half_spread, half_spread);
            let angle = base_angle + jitter;
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            let angular_velocity =
                self.uniform_range(config.angular_velocity_range.0, config.angular_velocity_range.1);

            self.pool[slot] = Particle {
                position: origin,
                velocity,
                acceleration: config.gravity,
                color_start: config.color_start,
                color_end: config.color_end,
                scale_start: config.scale_start,
                scale_end: config.scale_end,
                rotation: 0.0,
                angular_velocity,
                drag: config.drag,
                age: 0.0,
                lifetime,
                emissive: config.emissive,
                texture: config.texture,
                alive: true,
            };
        }
    }

    /// Advance every alive particle by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        for p in &mut self.pool {
            if !p.alive {
                continue;
            }
            p.age += dt;
            if p.age >= p.lifetime {
                p.alive = false;
                continue;
            }
            // Symplectic Euler: integrate velocity then position. Stable
            // enough for cosmetic particles even at large dt.
            p.velocity += p.acceleration * dt;
            if p.drag > 0.0 {
                // Exponential damping: v *= exp(-drag * dt). Use a cheap
                // 2-term approximation since dt is small.
                let decay = (1.0 - p.drag * dt).max(0.0);
                p.velocity *= decay;
            }
            p.position += p.velocity * dt;
            p.rotation += p.angular_velocity * dt;
        }
    }

    /// Iterate alive particles in pool order.
    pub fn iter_alive(&self) -> impl Iterator<Item = &Particle> {
        self.pool.iter().filter(|p| p.alive)
    }

    /// Interpolated color of an in-flight particle. Snapshot — does not
    /// mutate. Returned color is what the renderer should tint with.
    pub fn current_color(p: &Particle) -> Vec4 {
        let t = p.t();
        p.color_start.lerp(p.color_end, t)
    }

    /// Interpolated scale of an in-flight particle.
    pub fn current_scale(p: &Particle) -> f32 {
        let t = p.t();
        p.scale_start + (p.scale_end - p.scale_start) * t
    }

    /// Walk the pool finding a dead slot. Wraps; if the entire pool is alive
    /// the cursor's slot is reused (overwriting the oldest).
    fn next_slot(&mut self) -> usize {
        let cap = self.pool.len();
        for _ in 0..cap {
            let idx = self.cursor;
            self.cursor = (self.cursor + 1) % cap;
            if !self.pool[idx].alive {
                return idx;
            }
        }
        // Pool is fully alive — overwrite at the cursor position.
        let idx = self.cursor;
        self.cursor = (self.cursor + 1) % cap;
        idx
    }

    /// xorshift64* — fast, deterministic, fine for cosmetic randomness.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn uniform_range(&mut self, min: f32, max: f32) -> f32 {
        if (max - min).abs() < f32::EPSILON {
            return min;
        }
        // 24 bits of mantissa is plenty for direction/speed jitter.
        let bits = (self.next_u64() >> 40) as u32;
        let unit = bits as f32 / (1u32 << 24) as f32;
        min + unit * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn new_pool_has_no_alive_particles() {
        let m = ParticleManager::default();
        assert_eq!(m.alive_count(), 0);
        assert_eq!(m.capacity(), DEFAULT_CAPACITY);
    }

    #[test]
    fn with_capacity_clamps_to_at_least_one() {
        let m = ParticleManager::with_capacity(0);
        assert_eq!(m.capacity(), 1);
    }

    #[test]
    fn spawn_burst_creates_count_particles() {
        let mut m = ParticleManager::with_capacity(64);
        let cfg = ParticleConfig::burst(10);
        m.spawn_burst(Vec2::ZERO, &cfg);
        assert_eq!(m.alive_count(), 10);
    }

    #[test]
    fn step_decays_particles_to_death() {
        let mut m = ParticleManager::with_capacity(64);
        let cfg = ParticleConfig::burst(5).with_lifetime(0.1, 0.1);
        m.spawn_burst(Vec2::ZERO, &cfg);
        assert_eq!(m.alive_count(), 5);
        m.step(0.05);
        assert_eq!(m.alive_count(), 5, "still alive at half-life");
        m.step(0.1);
        assert_eq!(m.alive_count(), 0, "all expired after full lifetime");
    }

    #[test]
    fn spawn_reuses_dead_slots() {
        let mut m = ParticleManager::with_capacity(8);
        let cfg = ParticleConfig::burst(5).with_lifetime(0.1, 0.1);
        m.spawn_burst(Vec2::ZERO, &cfg);
        m.step(0.2); // all dead
        assert_eq!(m.alive_count(), 0);
        m.spawn_burst(Vec2::ZERO, &cfg);
        assert_eq!(m.alive_count(), 5);
    }

    #[test]
    fn overfull_pool_overwrites_oldest() {
        let mut m = ParticleManager::with_capacity(4);
        let cfg = ParticleConfig::burst(10).with_lifetime(10.0, 10.0);
        m.spawn_burst(Vec2::ZERO, &cfg);
        // Pool can only hold 4, so 4 alive (the most recent 4).
        assert_eq!(m.alive_count(), 4);
    }

    #[test]
    fn clear_kills_all_particles() {
        let mut m = ParticleManager::with_capacity(16);
        let cfg = ParticleConfig::burst(8);
        m.spawn_burst(Vec2::ZERO, &cfg);
        assert_eq!(m.alive_count(), 8);
        m.clear();
        assert_eq!(m.alive_count(), 0);
    }

    #[test]
    fn velocity_integrates_position_over_time() {
        let mut m = ParticleManager::with_capacity(2);
        let cfg = ParticleConfig::burst(1)
            .with_lifetime(10.0, 10.0)
            .with_speed(100.0, 100.0)
            .with_direction(Vec2::X, 0.0);
        m.spawn_burst(Vec2::ZERO, &cfg);
        m.step(0.5);
        let p = m.iter_alive().next().unwrap();
        // 100 units/s for 0.5s = ~50 units along X.
        assert!((p.position.x - 50.0).abs() < 0.5, "x: {}", p.position.x);
        assert!(p.position.y.abs() < 1e-3);
    }

    #[test]
    fn direction_spread_stays_within_cone() {
        let mut m = ParticleManager::with_capacity(256);
        m.reseed(42);
        let cfg = ParticleConfig::burst(200)
            .with_speed(100.0, 100.0)
            .with_direction(Vec2::Y, std::f32::consts::FRAC_PI_2); // 90° half-cone
        m.spawn_burst(Vec2::ZERO, &cfg);
        // With direction = +Y and ±45° cone, every particle should have positive Y velocity.
        for p in m.iter_alive() {
            assert!(p.velocity.y > 0.0, "expected upward y, got {:?}", p.velocity);
        }
    }

    #[test]
    fn gravity_pulls_particles_down() {
        let mut m = ParticleManager::with_capacity(2);
        let cfg = ParticleConfig::burst(1)
            .with_lifetime(10.0, 10.0)
            .with_speed(0.0, 0.0)
            .with_gravity(Vec2::new(0.0, -100.0));
        m.spawn_burst(Vec2::ZERO, &cfg);
        m.step(0.5);
        let p = m.iter_alive().next().unwrap();
        assert!(p.position.y < 0.0, "expected y < 0 from gravity, got {}", p.position.y);
    }

    #[test]
    fn current_color_interpolates() {
        let p = Particle {
            lifetime: 1.0,
            age: 0.5,
            color_start: Vec4::new(1.0, 0.0, 0.0, 1.0),
            color_end: Vec4::new(0.0, 0.0, 1.0, 1.0),
            ..Default::default()
        };
        let c = ParticleManager::current_color(&p);
        // Halfway: equal mix of red and blue.
        assert!((c.x - 0.5).abs() < 1e-3);
        assert!((c.z - 0.5).abs() < 1e-3);
    }
}
