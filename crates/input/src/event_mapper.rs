use winit::event::{Event, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event::ElementState;

use super::PlayerCommand;

/// Translate a raw winit event into your semantic command.
///
/// The new 0.30 API exposes `PhysicalKey::Code(..)` for layout‑independent keys
/// (Escape, A, D, etc.).  Logical‐layout keys live in `Key`, but for games the
/// *physical* codes are usually what you want. :contentReference[oaicite:0]{index=0}
pub fn command_from_event(event: &Event<()>) -> Option<PlayerCommand> {
    let Event::WindowEvent { event: WindowEvent::KeyboardInput { event: key_event, .. }, .. } = event else {
        return None;
    };

    if key_event.state != ElementState::Pressed {
        return None;
    }

    match key_event.physical_key {
        PhysicalKey::Code(KeyCode::Escape) => Some(PlayerCommand::TogglePause),
        PhysicalKey::Code(KeyCode::KeyA)    => Some(PlayerCommand::MoveLeft),
        PhysicalKey::Code(KeyCode::KeyD)    => Some(PlayerCommand::MoveRight),
        PhysicalKey::Code(KeyCode::Space)   => Some(PlayerCommand::Jump),
        PhysicalKey::Code(KeyCode::Enter)   => Some(PlayerCommand::FireWeapon),
        _ => None,
    }
}
