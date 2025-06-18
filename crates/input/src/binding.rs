use winit::keyboard::PhysicalKey;
use winit::event::ElementState;

use crate::player_command::PlayerCommand;

/// One mapping from a low‐level key event to your high‐level command.
#[derive(Debug, Clone)]
pub struct InputBinding {
    /// The semantic command the game cares about.
    pub command: PlayerCommand,
    /// The physical key that triggers it.
    pub key: PhysicalKey,
    /// Press, release or hold?
    pub state: ElementState,
}

impl InputBinding {
    pub fn new(
        command: PlayerCommand,
        key: PhysicalKey,
        state: ElementState,
    ) -> Self {
        Self { command, key, state }
    }
}
