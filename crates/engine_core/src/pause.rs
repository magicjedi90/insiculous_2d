//! Shared pause mechanism: state, toggle, and the standard pause menu.
//!
//! The engine owns the mechanism — Menu (Escape / any pad's Start) toggles,
//! [`MenuInput`] navigates, and the menu offers Resume / Restart / Quit to
//! Title. Games own the meaning: they decide which states are pausable,
//! call [`PauseMenu::update`] there, skip their entire gameplay update while
//! paused (no physics step, no timers), and map [`PauseAction::Restart`] /
//! [`PauseAction::QuitToTitle`] onto their own `start_game` / `reset_to_title`.
//!
//! ```
//! use engine_core::{PauseAction, PauseMenu};
//! use input::{InputHandler, InputSettings};
//!
//! let mut pause = PauseMenu::new();
//! let settings = InputSettings::default_two_player();
//! let input = InputHandler::new();
//!
//! // Each frame, from a pausable gameplay state:
//! match pause.update(&settings, &input) {
//!     PauseAction::Restart => { /* self.start_game(...) */ }
//!     PauseAction::QuitToTitle => { /* self.reset_to_title(...) */ }
//!     PauseAction::Resumed => { /* skip this frame; unfreeze next */ }
//!     PauseAction::Idle => {}
//! }
//! // ctx.time_scale = pause.time_scale();   // freezes engine particles
//! if pause.is_active() { /* skip gameplay; draw the overlay in the UI pass */ }
//! ```

use glam::Vec2;
use input::{GameAction, InputHandler, InputSettings};
use ui::UIContext;

use crate::menu_input::MenuInput;
use crate::menu_panel::{MenuPanel, MenuStyle};

/// What the pause menu decided this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    /// Nothing changed: either not paused, or paused and still browsing.
    Idle,
    /// The player unpaused (toggle or the Resume item). Games should skip
    /// the rest of this frame's update so the resuming keypress can't leak
    /// into gameplay; the world unfreezes next frame.
    Resumed,
    /// The player picked Restart — restart the current match. Unpauses.
    Restart,
    /// The player picked Quit to Title. Unpauses.
    QuitToTitle,
}

/// The pause menu's item labels, in selection order.
const ITEMS: [&str; 3] = ["Resume", "Restart", "Quit to Title"];

/// Pause state + menu. Embed one per game and drive it from the game's
/// pausable states (see the module docs for the frame pattern).
#[derive(Debug, Default)]
pub struct PauseMenu {
    active: bool,
    selection: u8,
}

impl PauseMenu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the game is currently paused.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// `0.0` while paused, `1.0` otherwise — assign to `ctx.time_scale`
    /// every frame so engine-side particles freeze with the game.
    pub fn time_scale(&self) -> f32 {
        if self.active { 0.0 } else { 1.0 }
    }

    /// Advance the pause state machine one frame.
    ///
    /// Not paused: a Menu edge (Escape / any pad's Start, either player)
    /// pauses. Paused: Menu or back (Escape/Start/B) always resumes —
    /// the same button that pauses unpauses, predictably — and
    /// confirm (Space/Enter/A) executes the highlighted item.
    pub fn update(&mut self, players: &InputSettings, input: &InputHandler) -> PauseAction {
        let menu_pressed = players.just_activated_any(GameAction::Menu, input);

        if !self.active {
            if menu_pressed {
                self.active = true;
                self.selection = 0;
            }
            return PauseAction::Idle;
        }

        let nav = MenuInput::read(input);
        // Toggle/back wins over confirm so pad Start (bound to both) always
        // resumes rather than executing whatever happens to be highlighted.
        if menu_pressed || nav.back {
            self.active = false;
            return PauseAction::Resumed;
        }

        self.selection = nav.navigate(self.selection, ITEMS.len() as u8);
        if nav.confirm {
            self.active = false;
            return match self.selection {
                0 => PauseAction::Resumed,
                1 => PauseAction::Restart,
                _ => PauseAction::QuitToTitle,
            };
        }
        PauseAction::Idle
    }

    /// Draw the standard pause overlay (input-blocking, dimmed backdrop,
    /// menu-panel chrome). Call from the game's UI pass while
    /// [`is_active`](Self::is_active); the frozen world stays visible
    /// beneath it.
    pub fn draw(&self, ui: &mut UIContext, window_size: Vec2, style: &MenuStyle) {
        let panel = MenuPanel::new("PAUSED", window_size / 2.0, 300.0, ITEMS.len());
        panel.draw_as_overlay(ui, window_size, style, |panel, ui, mut y| {
            for (i, item) in ITEMS.iter().enumerate() {
                y = panel.item(ui, y, item, i as u8 == self.selection, style);
            }
            panel.hint(ui, "ESC resumes - SPACE confirms", style);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::{GamepadButton, InputEvent};
    use winit::keyboard::KeyCode;

    fn frame(input: &mut InputHandler, events: &[InputEvent]) {
        input.end_frame();
        for event in events {
            input.queue_event(event.clone());
        }
        input.process_queued_events();
    }

    fn setup() -> (PauseMenu, InputSettings, InputHandler) {
        (PauseMenu::new(), InputSettings::default_two_player(), InputHandler::new())
    }

    #[test]
    fn menu_press_pauses_and_same_button_resumes() {
        let (mut pause, settings, mut input) = setup();

        // Escape edge pauses
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Idle);
        assert!(pause.is_active());

        // Held Escape is not an edge — stays paused, no toggle-flapping
        frame(&mut input, &[]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Idle);
        assert!(pause.is_active());

        // Release, press again: resumes
        frame(&mut input, &[InputEvent::KeyReleased(KeyCode::Escape)]);
        pause.update(&settings, &input);
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Resumed);
        assert!(!pause.is_active());
    }

    #[test]
    fn pad_start_pauses_and_resumes_from_either_player() {
        let (mut pause, settings, mut input) = setup();

        // Player 2's pad (id 1) Start pauses...
        frame(&mut input, &[InputEvent::GamepadButtonPressed(1, GamepadButton::Start)]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Idle);
        assert!(pause.is_active());

        // ...and player 1's pad Start resumes (any player controls pause)
        frame(&mut input, &[
            InputEvent::GamepadButtonReleased(1, GamepadButton::Start),
        ]);
        pause.update(&settings, &input);
        frame(&mut input, &[InputEvent::GamepadButtonPressed(0, GamepadButton::Start)]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Resumed);
    }

    #[test]
    fn back_button_resumes() {
        let (mut pause, settings, mut input) = setup();
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        pause.update(&settings, &input);
        assert!(pause.is_active());

        frame(&mut input, &[
            InputEvent::KeyReleased(KeyCode::Escape),
            InputEvent::GamepadButtonPressed(0, GamepadButton::B),
        ]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Resumed);
    }

    #[test]
    fn confirm_executes_highlighted_item() {
        for (downs, expected) in [
            (0, PauseAction::Resumed),
            (1, PauseAction::Restart),
            (2, PauseAction::QuitToTitle),
        ] {
            let (mut pause, settings, mut input) = setup();
            frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
            pause.update(&settings, &input);
            frame(&mut input, &[InputEvent::KeyReleased(KeyCode::Escape)]);
            pause.update(&settings, &input);

            for _ in 0..downs {
                frame(&mut input, &[InputEvent::KeyPressed(KeyCode::KeyS)]);
                assert_eq!(pause.update(&settings, &input), PauseAction::Idle);
                frame(&mut input, &[InputEvent::KeyReleased(KeyCode::KeyS)]);
                pause.update(&settings, &input);
            }

            frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Space)]);
            assert_eq!(pause.update(&settings, &input), expected, "{downs} downs");
            assert!(!pause.is_active(), "every confirm unpauses");
        }
    }

    #[test]
    fn selection_wraps_and_resets_on_reopen() {
        let (mut pause, settings, mut input) = setup();
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        pause.update(&settings, &input);
        frame(&mut input, &[InputEvent::KeyReleased(KeyCode::Escape)]);
        pause.update(&settings, &input);

        // Up from the top wraps to the last item (Quit) — confirm proves it
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::KeyW)]);
        pause.update(&settings, &input);
        frame(&mut input, &[
            InputEvent::KeyReleased(KeyCode::KeyW),
            InputEvent::KeyPressed(KeyCode::Space),
        ]);
        assert_eq!(pause.update(&settings, &input), PauseAction::QuitToTitle);

        // Reopening starts back at Resume
        frame(&mut input, &[
            InputEvent::KeyReleased(KeyCode::Space),
            InputEvent::KeyPressed(KeyCode::Escape),
        ]);
        pause.update(&settings, &input);
        frame(&mut input, &[
            InputEvent::KeyReleased(KeyCode::Escape),
            InputEvent::KeyPressed(KeyCode::Space),
        ]);
        assert_eq!(pause.update(&settings, &input), PauseAction::Resumed);
    }

    #[test]
    fn time_scale_is_zero_only_while_paused() {
        let (mut pause, settings, mut input) = setup();
        assert_eq!(pause.time_scale(), 1.0);

        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        pause.update(&settings, &input);
        assert_eq!(pause.time_scale(), 0.0);

        frame(&mut input, &[InputEvent::KeyReleased(KeyCode::Escape)]);
        pause.update(&settings, &input);
        frame(&mut input, &[InputEvent::KeyPressed(KeyCode::Escape)]);
        pause.update(&settings, &input);
        assert_eq!(pause.time_scale(), 1.0);
    }
}
