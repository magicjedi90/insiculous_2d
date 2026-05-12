//! Spring-mass grid module.
//!
//! A grid of mass-points connected by springs, simulated on the CPU and
//! rendered as glowing lines. Apply impulses with [`GridMesh::apply_impulse`]
//! to deform the grid — paddle hits and explosions ripple outward.
//!
//! Render path: each frame, build the line vertex buffer with
//! [`GridMesh::build_line_vertices`] and hand it to
//! [`RenderManager::set_lines`](crate::render_manager::RenderManager::set_lines).
//! The line pipeline writes into the HDR target so the grid blooms.

mod grid_mesh;
mod impulse;

pub use grid_mesh::GridMesh;
pub use impulse::GridImpulse;
