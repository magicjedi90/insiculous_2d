//! Shared menu-screen input: one frame's navigation keys plus wraparound
//! list navigation.
//!
//! Every arcade game's title/select screens read the same four signals
//! (W/↑ up, S/↓ down, Space/Enter confirm, Escape back) and move a cursor
//! through a wrapping list. The engine owns that mechanism; games own what
//! each screen and selection *means*.
//!
//! ```no_run
//! use engine_core::prelude::*;
//!
//! # fn update(ctx: &mut GameContext, selection: u8) {
//! let input = MenuInput::read(ctx.input);
//! let selection = input.navigate(selection, 3);
//! if input.confirm { /* enter the selected item */ }
//! if input.back { /* return to the previous screen */ }
//! # }
//! ```

use input::InputHandler;
use winit::keyboard::KeyCode;

/// One frame's worth of menu keys, read once per screen update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuInput {
    /// W or ArrowUp was just pressed.
    pub up: bool,
    /// S or ArrowDown was just pressed.
    pub down: bool,
    /// Space or Enter was just pressed.
    pub confirm: bool,
    /// Escape was just pressed.
    pub back: bool,
}

impl MenuInput {
    /// Read this frame's menu keys from the input handler.
    pub fn read(input: &InputHandler) -> Self {
        Self {
            up: input.is_key_just_pressed(KeyCode::ArrowUp)
                || input.is_key_just_pressed(KeyCode::KeyW),
            down: input.is_key_just_pressed(KeyCode::ArrowDown)
                || input.is_key_just_pressed(KeyCode::KeyS),
            confirm: input.is_key_just_pressed(KeyCode::Space)
                || input.is_key_just_pressed(KeyCode::Enter),
            back: input.is_key_just_pressed(KeyCode::Escape),
        }
    }

    /// Move `current` through a `count`-item list with wraparound.
    /// `up` takes precedence when both directions fire on the same frame.
    pub fn navigate(&self, current: u8, count: u8) -> u8 {
        if count == 0 {
            return 0;
        }
        if self.up {
            if current == 0 { count - 1 } else { current - 1 }
        } else if self.down {
            (current + 1) % count
        } else {
            current
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(up: bool, down: bool) -> MenuInput {
        MenuInput { up, down, confirm: false, back: false }
    }

    #[test]
    fn test_navigate_wraps_both_directions() {
        assert_eq!(keys(true, false).navigate(0, 3), 2);
        assert_eq!(keys(false, true).navigate(2, 3), 0);
    }

    #[test]
    fn test_navigate_holds_position_without_input() {
        assert_eq!(keys(false, false).navigate(1, 3), 1);
    }

    #[test]
    fn test_navigate_up_wins_over_down_on_same_frame() {
        assert_eq!(keys(true, true).navigate(1, 3), 0);
    }

    #[test]
    fn test_navigate_empty_list_stays_at_zero() {
        assert_eq!(keys(true, false).navigate(0, 0), 0);
    }

    #[test]
    fn test_read_on_idle_handler_reports_nothing() {
        let handler = InputHandler::new();
        let input = MenuInput::read(&handler);
        assert_eq!(input, MenuInput { up: false, down: false, confirm: false, back: false });
    }
}
