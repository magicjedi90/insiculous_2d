#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerCommand {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Jump,
    Action,
    TogglePause
}