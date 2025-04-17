//! Small “prelude” so engine users can `use engine_core::prelude::*;`
//! without pulling the whole crate into scope.  Keep this list minimal;
//! Add new items *only* when they appear in almost every game file.

pub use log::{debug, error, info, trace, warn};
pub use glam::{Vec2, Vec3};                                // common math types

// High‑level engine symbols
pub use crate::engine::{launch, GameState};

// Event bus types (decoupled Observer pattern)
pub use crate::events::{EngineEvent, EventBus};

// Time helpers
pub use crate::time::ApplicationClock;
