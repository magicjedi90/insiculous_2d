//! Shared menu-screen input: one frame's navigation signals plus wraparound
//! list navigation.
//!
//! Every arcade game's title/select screens read the same four signals
//! (up, down, confirm, back) and move a cursor through a wrapping list.
//! Signals come from the keyboard (W/↑, S/↓, Space/Enter, Escape) and from
//! **every connected gamepad** (dpad / left stick, A or Start, B) — menus
//! don't care which player navigates. The engine owns that mechanism; games
//! own what each screen and selection *means*.
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

use input::{AxisDirection, GamepadAxis, GamepadButton, GamepadState, InputHandler, AXIS_ACTIVATION_THRESHOLD};
use winit::keyboard::KeyCode;

/// One frame's worth of menu signals, read once per screen update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuInput {
    /// W/ArrowUp, or any pad's DPadUp / left-stick-up edge, was just pressed.
    pub up: bool,
    /// S/ArrowDown, or any pad's DPadDown / left-stick-down edge, was just pressed.
    pub down: bool,
    /// Space/Enter, or any pad's A or Start, was just pressed.
    pub confirm: bool,
    /// Escape, or any pad's B, was just pressed.
    pub back: bool,
}

impl MenuInput {
    /// Read this frame's menu signals from the input handler (keyboard plus
    /// every connected gamepad).
    pub fn read(input: &InputHandler) -> Self {
        let mut menu = Self {
            up: input.is_key_just_pressed(KeyCode::ArrowUp)
                || input.is_key_just_pressed(KeyCode::KeyW),
            down: input.is_key_just_pressed(KeyCode::ArrowDown)
                || input.is_key_just_pressed(KeyCode::KeyS),
            confirm: input.is_key_just_pressed(KeyCode::Space)
                || input.is_key_just_pressed(KeyCode::Enter),
            back: input.is_key_just_pressed(KeyCode::Escape),
        };
        for (_, pad) in input.gamepads().iter() {
            menu.up |= pad_up(pad);
            menu.down |= pad_down(pad);
            menu.confirm |= pad.is_button_just_pressed(GamepadButton::A)
                || pad.is_button_just_pressed(GamepadButton::Start);
            menu.back |= pad.is_button_just_pressed(GamepadButton::B);
        }
        menu
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

/// Pad "up" edge: DPadUp just pressed, or the left stick just crossed the
/// activation threshold upward (stick Y positive = up). Edge detection (not
/// level) so a held stick doesn't repeat-scroll every frame.
fn pad_up(pad: &GamepadState) -> bool {
    pad.is_button_just_pressed(GamepadButton::DPadUp)
        || pad.axis_just_activated(
            GamepadAxis::LeftStickY,
            AxisDirection::Positive,
            AXIS_ACTIVATION_THRESHOLD,
        )
}

/// Pad "down" edge: DPadDown just pressed, or the left stick just crossed
/// the activation threshold downward.
fn pad_down(pad: &GamepadState) -> bool {
    pad.is_button_just_pressed(GamepadButton::DPadDown)
        || pad.axis_just_activated(
            GamepadAxis::LeftStickY,
            AxisDirection::Negative,
            AXIS_ACTIVATION_THRESHOLD,
        )
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

    use input::InputEvent;

    fn frame(handler: &mut InputHandler, events: &[InputEvent]) {
        for event in events {
            handler.queue_event(event.clone());
        }
        handler.process_queued_events();
    }

    #[test]
    fn test_any_pads_dpad_and_buttons_navigate_menus() {
        let mut handler = InputHandler::new();
        // Pad 1 (not just pad 0) drives menus too
        frame(&mut handler, &[InputEvent::GamepadButtonPressed(1, GamepadButton::DPadDown)]);
        assert!(MenuInput::read(&handler).down);
        handler.end_frame();

        frame(&mut handler, &[
            InputEvent::GamepadButtonReleased(1, GamepadButton::DPadDown),
            InputEvent::GamepadButtonPressed(0, GamepadButton::A),
            InputEvent::GamepadButtonPressed(1, GamepadButton::B),
        ]);
        let input = MenuInput::read(&handler);
        assert!(input.confirm);
        assert!(input.back);
        assert!(!input.down, "released dpad no longer navigates");
    }

    #[test]
    fn test_held_stick_scrolls_once_not_every_frame() {
        let mut handler = InputHandler::new();

        // Frame 1: stick pushed up — edge fires
        frame(&mut handler, &[InputEvent::GamepadAxisUpdated(0, GamepadAxis::LeftStickY, 0.9)]);
        assert!(MenuInput::read(&handler).up);
        handler.end_frame();

        // Frame 2: stick still held — no repeat
        frame(&mut handler, &[InputEvent::GamepadAxisUpdated(0, GamepadAxis::LeftStickY, 0.9)]);
        assert!(!MenuInput::read(&handler).up);
        handler.end_frame();

        // Frame 3: stick released, pushed down — down edge fires
        frame(&mut handler, &[InputEvent::GamepadAxisUpdated(0, GamepadAxis::LeftStickY, -0.9)]);
        let input = MenuInput::read(&handler);
        assert!(input.down);
        assert!(!input.up);
    }

    #[test]
    fn test_keyboard_menu_behavior_unchanged_with_idle_pad_connected() {
        let mut handler = InputHandler::new();
        frame(&mut handler, &[
            InputEvent::GamepadConnected(0),
            InputEvent::KeyPressed(KeyCode::KeyW),
        ]);
        let input = MenuInput::read(&handler);
        assert!(input.up);
        assert!(!input.down && !input.confirm && !input.back);
    }
}
