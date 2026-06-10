//! Spring-mass grid — a general-purpose deformable grid effect.
//!
//! A grid of mass-points connected by springs, simulated on the CPU and
//! rendered as glowing lines. Apply impulses with [`GridMesh::apply_impulse`]
//! to deform the grid — gameplay events (hits, explosions, pickups) ripple
//! outward Geometry-Wars style. This is an engine-level effect, usable by any
//! game as a reactive background, and a candidate backing for editor grid
//! visualization.
//!
//! The `Default` field values (stiffness, damping, color, glow) are starting
//! points, not requirements — configure per game via the public fields or
//! [`GridMesh::new`].
//!
//! Render path: each frame, build the line vertex buffer with
//! [`GridMesh::build_line_vertices`] and hand it to
//! [`RenderManager::set_lines`](crate::render_manager::RenderManager::set_lines).
//! The line pipeline writes into the HDR target so the grid blooms.

mod grid_mesh;
mod impulse;

pub use grid_mesh::GridMesh;
pub use impulse::GridImpulse;
