use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event::{ElementState, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum PlayerCommand {
    MoveLeft,
    MoveRight,
    Jump,
    FireWeapon,
}

pub fn interpret_keyboard_input(keyboard_input: &KeyEvent) -> Option<PlayerCommand> {
    if keyboard_input.state != ElementState::Pressed {
        return None;
    }
    match keyboard_input.physical_key {
        PhysicalKey::Code(KeyCode::KeyA)      => Some(PlayerCommand::MoveLeft),
        PhysicalKey::Code(KeyCode::KeyD)      => Some(PlayerCommand::MoveRight),
        PhysicalKey::Code(KeyCode::Space)     => Some(PlayerCommand::Jump),
        PhysicalKey::Code(KeyCode::Enter)     => Some(PlayerCommand::FireWeapon),
        _ => None,
    }
}
