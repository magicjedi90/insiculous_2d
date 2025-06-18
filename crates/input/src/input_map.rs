use winit::event::{Event, WindowEvent};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::{player_command::PlayerCommand, binding::InputBinding};

/// Holds *all* your game’s bindings and turns raw events into commands.
#[derive(Debug)]
pub struct InputMap {
    bindings: Vec<InputBinding>,
}

impl InputMap {
    /// Start with no bindings.
    pub fn new() -> Self {
        Self { bindings: Vec::new() }
    }

    /// Register one more binding.
    pub fn add_binding(&mut self, b: InputBinding) {
        self.bindings.push(b);
    }

    /// Remove *all* bindings for a command.
    pub fn remove_command(&mut self, cmd: PlayerCommand) {
        self.bindings.retain(|b| b.command != cmd);
    }

    /// Given a winit event, return the mapped command (if any).
    /// Call this from your engine’s event loop.
    pub fn map_event(&self, event: &Event<()>) -> Option<PlayerCommand> {
        // Only handle keyboard‐pressed events:
        let key_event = match event {
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event, .. }, .. } => event,
            _ => return None,
        };

        if key_event.state != ElementState::Pressed {
            return None;
        }

        // Look for the first binding that matches both key and state
        for b in &self.bindings {
            if b.key == key_event.physical_key && b.state == key_event.state {
                return Some(b.command);
            }
        }
        None
    }
}

impl Default for InputMap {
    fn default() -> Self {
        let mut input_map = InputMap::new();
        input_map.add_binding(InputBinding::new(PlayerCommand::TogglePause, PhysicalKey::Code(KeyCode::Escape), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::MoveDown, PhysicalKey::Code(KeyCode::KeyS), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::MoveUp, PhysicalKey::Code(KeyCode::KeyW), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::MoveLeft, PhysicalKey::Code(KeyCode::KeyA), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::MoveRight, PhysicalKey::Code(KeyCode::KeyD), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::Action, PhysicalKey::Code(KeyCode::Enter), ElementState::Pressed));
        input_map.add_binding(InputBinding::new(PlayerCommand::Jump, PhysicalKey::Code(KeyCode::Space), ElementState::Pressed));
        input_map
    }
}

