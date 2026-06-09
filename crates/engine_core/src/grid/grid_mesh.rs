//! The grid mesh itself: nodes, springs, and the Verlet integrator.
//!
//! Each node has a rest position; springs pull every node toward its
//! neighbors with the rest length set to the configured spacing. When an
//! impulse displaces a node, the connected springs propagate the
//! displacement and a Geometry-Wars-style ripple emerges naturally.
//!
//! Border nodes are pinned (inv_mass = 0) so the grid doesn't drift.

use glam::{Vec2, Vec4};
use renderer::line_pipeline::LineVertex;

use super::impulse::GridImpulse;

/// A single mass point in the grid.
#[derive(Debug, Clone, Copy)]
struct GridNode {
    /// Initial position. Nodes return here when impulses fade.
    rest: Vec2,
    /// Current position. Diverges from `rest` while the grid is excited.
    position: Vec2,
    velocity: Vec2,
    /// 0.0 = pinned (won't move). 1.0 = free node. Allows different stiffnesses.
    inv_mass: f32,
}

/// A spring connecting two nodes.
#[derive(Debug, Clone, Copy)]
struct Spring {
    a: u32,
    b: u32,
    rest_length: f32,
}

/// The spring-mass grid.
///
/// Build with [`GridMesh::new`]. Each frame:
/// 1. Apply impulses with [`apply_impulse`](Self::apply_impulse).
/// 2. Step the simulation with [`step`](Self::step).
/// 3. Build line vertices with [`build_line_vertices`](Self::build_line_vertices).
pub struct GridMesh {
    pub cols: u32,
    pub rows: u32,
    pub spacing: f32,
    pub origin: Vec2,
    nodes: Vec<GridNode>,
    springs: Vec<Spring>,
    /// Pre-allocated line vertex scratch reused every frame.
    line_scratch: Vec<LineVertex>,
    /// Pre-allocated spring-force scratch reused every substep.
    force_scratch: Vec<Vec2>,
    /// Visual params. The alpha component of `color` controls grid
    /// translucency — fade it to 0 to hide the grid while keeping the
    /// simulation running.
    pub color: Vec4,
    pub emissive: f32,
    /// When `false`, [`build_line_vertices`](Self::build_line_vertices)
    /// returns an empty slice and no lines render. The simulation still
    /// steps (so re-enabling later resumes a settled grid).
    pub visible: bool,
    /// Spring stiffness (force per unit of stretch). Higher = faster ripples,
    /// lower = wobblier.
    pub stiffness: f32,
    /// Per-frame velocity decay. 0.0 = no damping (energy preserved),
    /// 1.0 = critical damping (no movement). Typical 0.05–0.15.
    pub damping: f32,
    /// Pull-to-rest force coefficient. Without this the grid never returns
    /// to its rest position once excited. Typical 0.5–2.0.
    pub rest_pull: f32,
    /// Number of physics substeps per frame. Higher = more stable at large
    /// dt or high stiffness.
    pub substeps: u32,
}

impl GridMesh {
    /// Build a `cols × rows` grid with `spacing` world units between nodes,
    /// centered at `origin`.
    pub fn new(cols: u32, rows: u32, spacing: f32, origin: Vec2) -> Self {
        assert!(cols >= 2 && rows >= 2, "grid must be at least 2x2");
        let (nodes, springs) = build_topology(cols, rows, spacing, origin);
        let force_scratch = vec![Vec2::ZERO; nodes.len()];
        Self {
            cols,
            rows,
            spacing,
            origin,
            nodes,
            springs,
            line_scratch: Vec::new(),
            force_scratch,
            color: Vec4::new(0.2, 0.5, 1.0, 0.8),
            emissive: 0.6,
            visible: true,
            stiffness: 24.0,
            damping: 0.08,
            rest_pull: 1.0,
            substeps: 4,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self { self.color = color; self }
    pub fn with_emissive(mut self, emissive: f32) -> Self { self.emissive = emissive; self }
    pub fn with_stiffness(mut self, stiffness: f32) -> Self { self.stiffness = stiffness; self }
    pub fn with_damping(mut self, damping: f32) -> Self { self.damping = damping; self }

    /// Replace just the alpha of the grid's color. Useful for fading the
    /// grid in/out without touching the RGB tint.
    pub fn with_alpha(mut self, alpha: f32) -> Self { self.color.w = alpha.clamp(0.0, 1.0); self }

    /// Same as [`with_alpha`](Self::with_alpha) but on `&mut self` for
    /// imperative fading from gameplay code.
    pub fn set_alpha(&mut self, alpha: f32) { self.color.w = alpha.clamp(0.0, 1.0); }

    /// Builder variant of the [`visible`](Self::visible) field. Set to false
    /// to start the grid hidden — useful for games that toggle it on certain
    /// events (boss fights, menus, etc.).
    pub fn with_visible(mut self, visible: bool) -> Self { self.visible = visible; self }

    /// Number of mass points.
    pub fn node_count(&self) -> usize { self.nodes.len() }
    /// Number of spring connections.
    pub fn spring_count(&self) -> usize { self.springs.len() }

    /// Apply an impulse to the grid for one frame.
    pub fn apply_impulse(&mut self, impulse: &GridImpulse) {
        match *impulse {
            GridImpulse::Point { position, force, radius } => {
                let r2 = (radius * radius).max(1e-3);
                for node in &mut self.nodes {
                    if node.inv_mass == 0.0 { continue; }
                    let d2 = node.rest.distance_squared(position);
                    if d2 > r2 * 9.0 { continue; }
                    // Gaussian falloff: exp(-d^2 / (2 * sigma^2)) with sigma = radius/2.
                    let falloff = (-d2 / r2).exp();
                    node.velocity += force * falloff * node.inv_mass;
                }
            }
            GridImpulse::Radial { position, strength, radius, attractive } => {
                let r2 = (radius * radius).max(1e-3);
                let sign = if attractive { -1.0 } else { 1.0 };
                for node in &mut self.nodes {
                    if node.inv_mass == 0.0 { continue; }
                    let diff = node.rest - position;
                    let d2 = diff.length_squared();
                    if d2 > r2 * 9.0 || d2 < 1e-6 { continue; }
                    let dist = d2.sqrt();
                    let dir = diff / dist;
                    let falloff = (-d2 / r2).exp();
                    node.velocity += dir * (strength * falloff * sign) * node.inv_mass;
                }
            }
        }
    }

    /// Advance the simulation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 { return; }
        let substeps = self.substeps.max(1);
        let h = dt / substeps as f32;
        // Per-substep damping factor so total damping doesn't depend on dt.
        let damp = (1.0 - self.damping).clamp(0.0, 1.0).powf(h * 60.0);

        for _ in 0..substeps {
            self.accumulate_forces();
            for (node, force) in self.nodes.iter_mut().zip(self.force_scratch.iter()) {
                if node.inv_mass == 0.0 { continue; }
                node.velocity += *force * h * node.inv_mass;
                node.velocity *= damp;
                node.position += node.velocity * h;
            }
        }
    }

    /// Sum spring forces and rest-position pulls into `force_scratch`.
    fn accumulate_forces(&mut self) {
        for f in &mut self.force_scratch { *f = Vec2::ZERO; }

        // Spring forces.
        for s in &self.springs {
            let a = self.nodes[s.a as usize].position;
            let b = self.nodes[s.b as usize].position;
            let delta = b - a;
            let len = delta.length();
            if len < 1e-6 { continue; }
            let stretch = len - s.rest_length;
            let dir = delta / len;
            let f = dir * (stretch * self.stiffness);
            self.force_scratch[s.a as usize] += f;
            self.force_scratch[s.b as usize] -= f;
        }

        // Pull every node back toward its rest position so excitement decays.
        for (i, node) in self.nodes.iter().enumerate() {
            let pull = (node.rest - node.position) * self.rest_pull;
            self.force_scratch[i] += pull;
        }
    }

    /// Emit one [`LineVertex`] pair per spring into the scratch buffer and
    /// return the slice. Returned slice is invalidated by the next call.
    ///
    /// Uses the grid's `color` and `emissive` for every vertex. For a
    /// per-edge color (e.g. based on stretch), call manually.
    ///
    /// Returns an empty slice when [`visible`](Self::visible) is `false` or
    /// the color is fully transparent — skipping the per-spring loop
    /// entirely so a hidden grid costs nothing to "render".
    pub fn build_line_vertices(&mut self) -> &[LineVertex] {
        self.line_scratch.clear();
        if !self.visible || self.color.w <= 0.0 {
            return &self.line_scratch;
        }
        let color = self.color.to_array();
        let emissive = self.emissive;
        for s in &self.springs {
            let a = self.nodes[s.a as usize].position;
            let b = self.nodes[s.b as usize].position;
            self.line_scratch.push(LineVertex {
                position: a.to_array(),
                color,
                emissive,
            });
            self.line_scratch.push(LineVertex {
                position: b.to_array(),
                color,
                emissive,
            });
        }
        &self.line_scratch
    }

    /// Total kinetic energy in the grid — handy for tests verifying that
    /// undamped impulses keep the grid bounded and damped impulses decay.
    pub fn total_energy(&self) -> f32 {
        self.nodes.iter().map(|n| 0.5 * n.velocity.length_squared()).sum()
    }
}

/// Build a grid topology: rows*cols nodes with springs to right and down
/// neighbors. Border nodes are pinned.
fn build_topology(cols: u32, rows: u32, spacing: f32, origin: Vec2) -> (Vec<GridNode>, Vec<Spring>) {
    let half_w = (cols - 1) as f32 * spacing * 0.5;
    let half_h = (rows - 1) as f32 * spacing * 0.5;
    let mut nodes = Vec::with_capacity((cols * rows) as usize);
    for y in 0..rows {
        for x in 0..cols {
            let pos = origin + Vec2::new(x as f32 * spacing - half_w, y as f32 * spacing - half_h);
            let pinned = x == 0 || y == 0 || x == cols - 1 || y == rows - 1;
            nodes.push(GridNode {
                rest: pos,
                position: pos,
                velocity: Vec2::ZERO,
                inv_mass: if pinned { 0.0 } else { 1.0 },
            });
        }
    }

    let idx = |x: u32, y: u32| -> u32 { y * cols + x };
    let mut springs = Vec::with_capacity(((cols - 1) * rows + cols * (rows - 1)) as usize);
    for y in 0..rows {
        for x in 0..cols {
            if x + 1 < cols {
                springs.push(Spring {
                    a: idx(x, y),
                    b: idx(x + 1, y),
                    rest_length: spacing,
                });
            }
            if y + 1 < rows {
                springs.push(Spring {
                    a: idx(x, y),
                    b: idx(x, y + 1),
                    rest_length: spacing,
                });
            }
        }
    }
    (nodes, springs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_construction_sizes() {
        let g = GridMesh::new(5, 4, 10.0, Vec2::ZERO);
        assert_eq!(g.node_count(), 20);
        // Springs: (cols-1)*rows + cols*(rows-1) = 4*4 + 5*3 = 16+15 = 31.
        assert_eq!(g.spring_count(), 31);
    }

    #[test]
    fn border_nodes_are_pinned() {
        let g = GridMesh::new(3, 3, 1.0, Vec2::ZERO);
        // First node is corner, must be pinned.
        assert_eq!(g.nodes[0].inv_mass, 0.0);
        // Middle node (index 4 in 3x3) must be free.
        assert_eq!(g.nodes[4].inv_mass, 1.0);
    }

    #[test]
    fn impulse_moves_interior_node() {
        let mut g = GridMesh::new(5, 5, 10.0, Vec2::ZERO);
        let initial = g.nodes[12].position; // center
        g.apply_impulse(&GridImpulse::Point {
            position: initial,
            force: Vec2::new(0.0, 100.0),
            radius: 5.0,
        });
        g.step(0.05);
        assert!(g.nodes[12].position.y > initial.y, "node should have moved up");
    }

    #[test]
    fn pinned_corner_never_moves() {
        let mut g = GridMesh::new(5, 5, 10.0, Vec2::ZERO);
        let corner_rest = g.nodes[0].rest;
        g.apply_impulse(&GridImpulse::Radial {
            position: corner_rest,
            strength: 1000.0,
            radius: 100.0,
            attractive: false,
        });
        for _ in 0..30 {
            g.step(0.016);
        }
        assert!((g.nodes[0].position - corner_rest).length() < 1e-3);
    }

    #[test]
    fn energy_decays_with_damping() {
        let mut g = GridMesh::new(7, 7, 10.0, Vec2::ZERO).with_damping(0.2);
        g.apply_impulse(&GridImpulse::Radial {
            position: Vec2::ZERO,
            strength: 500.0,
            radius: 20.0,
            attractive: false,
        });
        g.step(0.016);
        let e_start = g.total_energy();
        for _ in 0..240 { // ~4 seconds
            g.step(0.016);
        }
        let e_end = g.total_energy();
        assert!(e_end < e_start * 0.1, "energy {} -> {} should decay heavily", e_start, e_end);
    }

    #[test]
    fn grid_returns_to_rest() {
        let mut g = GridMesh::new(5, 5, 10.0, Vec2::ZERO).with_damping(0.2);
        let center_idx = 12;
        let rest = g.nodes[center_idx].rest;
        g.apply_impulse(&GridImpulse::Point {
            position: rest,
            force: Vec2::new(50.0, 50.0),
            radius: 8.0,
        });
        for _ in 0..600 { // ~10 seconds — plenty of time to settle
            g.step(0.016);
        }
        let final_offset = (g.nodes[center_idx].position - rest).length();
        assert!(final_offset < 0.1, "node should settle near rest, offset = {}", final_offset);
    }

    #[test]
    fn build_line_vertices_produces_two_per_spring() {
        let mut g = GridMesh::new(4, 3, 1.0, Vec2::ZERO);
        let verts = g.build_line_vertices();
        assert_eq!(verts.len(), g.spring_count() * 2);
    }

    #[test]
    fn invisible_grid_produces_no_vertices() {
        let mut g = GridMesh::new(4, 3, 1.0, Vec2::ZERO);
        g.visible = false;
        assert_eq!(g.build_line_vertices().len(), 0);
    }

    #[test]
    fn transparent_grid_produces_no_vertices() {
        let mut g = GridMesh::new(4, 3, 1.0, Vec2::ZERO).with_alpha(0.0);
        assert_eq!(g.build_line_vertices().len(), 0);
    }

    #[test]
    fn alpha_clamped_to_unit_range() {
        let g = GridMesh::new(3, 3, 1.0, Vec2::ZERO).with_alpha(2.5);
        assert_eq!(g.color.w, 1.0);
        let g = GridMesh::new(3, 3, 1.0, Vec2::ZERO).with_alpha(-0.4);
        assert_eq!(g.color.w, 0.0);
    }

    #[test]
    fn hidden_grid_still_simulates() {
        // Re-enabling a hidden grid should resume from the same physics
        // state, so invisible grids must keep stepping normally.
        let mut g = GridMesh::new(5, 5, 10.0, Vec2::ZERO);
        g.visible = false;
        let center_idx = 12;
        g.apply_impulse(&GridImpulse::Point {
            position: g.nodes[center_idx].rest,
            force: Vec2::new(0.0, 100.0),
            radius: 5.0,
        });
        g.step(0.05);
        assert!(g.nodes[center_idx].position.y > g.nodes[center_idx].rest.y);
    }
}
