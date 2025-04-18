//! # Insiculous2D Game Engine
//!
//! A unified interface to the Insiculous2D engine components.

// Re-export all the individual crates
pub use engine_core;
pub use renderer;
pub use components;
pub use input;
pub use audio;
pub use builtin_states;

// Create a comprehensive prelude that includes everything users typically need
pub mod prelude {
    // Re-export engine_core prelude
    pub use engine_core::prelude::*;

    // Re-export other commonly used items from each crate
    pub use components::{World, Entity, SpatialTransform, RenderableSprite};
    pub use input::*;  // Adjust as needed
    pub use audio::Audio;
    pub use builtin_states::*;

    // Additional re-exports as needed
}
