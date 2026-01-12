//! Common types and utilities for the Insiculous 2D game engine.
//!
//! This crate provides shared types used across multiple engine crates,
//! eliminating duplication and ensuring consistency.

pub mod color;
pub mod transform;
pub mod camera;
pub mod rect;
pub mod macros;

pub mod prelude {
    //! Prelude module for common types.
    //!
    //! Import with `use common::prelude::*;`

    pub use crate::color::Color;
    pub use crate::transform::Transform2D;
    pub use crate::camera::Camera2D;
    pub use crate::rect::Rect;
}

// Re-export at crate root for convenience
pub use color::Color;
pub use transform::Transform2D;
pub use camera::Camera2D;
pub use rect::Rect;
