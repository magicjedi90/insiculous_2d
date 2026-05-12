//! Impulse shapes that deform the grid.

use glam::Vec2;

/// An external force applied to grid nodes for one frame.
#[derive(Debug, Clone, Copy)]
pub enum GridImpulse {
    /// Apply a fixed force vector to every node within `radius` of `position`.
    /// Force falls off with a Gaussian curve so the effect is localized.
    Point {
        position: Vec2,
        force: Vec2,
        radius: f32,
    },
    /// Radial wave from `position`. When `attractive` is false, nodes are
    /// pushed outward; when true, pulled inward. `strength` is the peak
    /// force at the center; falloff is Gaussian over `radius`.
    Radial {
        position: Vec2,
        strength: f32,
        radius: f32,
        attractive: bool,
    },
}
