pub mod event_mapper;

#[derive(Debug, Clone, Copy)]
pub enum PlayerCommand {
    MoveLeft,
    MoveRight,
    Jump,
    FireWeapon,
    TogglePause
}
