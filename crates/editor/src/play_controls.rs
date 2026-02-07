//! Play / Pause / Stop controls for the editor toolbar.
//!
//! Renders context-sensitive buttons next to the tool toolbar and returns
//! the action the user clicked, if any.

use glam::Vec2;
use ui::{Color, Rect, UIContext};

use crate::play_state::EditorPlayState;

/// Action returned when the user clicks a play control button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayControlAction {
    /// Start or resume the game simulation.
    Play,
    /// Pause the running game simulation.
    Pause,
    /// Stop the game and restore the pre-play snapshot.
    Stop,
}

/// Play control widget rendered to the right of the tool toolbar.
#[derive(Debug, Clone)]
pub struct PlayControls {
    /// Position (set each frame based on toolbar bounds).
    pub position: Vec2,
    /// Button size (matches toolbar button size).
    pub button_size: f32,
    /// Spacing between buttons.
    pub spacing: f32,
}

impl Default for PlayControls {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayControls {
    /// Create new play controls with default sizing.
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            button_size: 40.0,
            spacing: 4.0,
        }
    }

    /// Render play controls and return the clicked action, if any.
    ///
    /// Button layout varies by state:
    /// - **Editing:** `[Play]`
    /// - **Playing:** `[Pause] [Stop]`
    /// - **Paused:**  `[Resume] [Stop]`
    pub fn render(&self, ui: &mut UIContext, state: EditorPlayState) -> Option<PlayControlAction> {
        let mut action = None;
        let x = self.position.x;
        let y = self.position.y;

        // Visual separator line between toolbar and play controls
        let sep_x = x - self.spacing * 2.0;
        ui.line(
            Vec2::new(sep_x, y + 4.0),
            Vec2::new(sep_x, y + self.button_size - 4.0),
            Color::new(0.4, 0.4, 0.4, 0.6),
            1.0,
        );

        match state {
            EditorPlayState::Editing => {
                let btn = Rect::new(x, y, self.button_size, self.button_size);
                // Green tint behind play button
                ui.rect_rounded(btn, Color::new(0.15, 0.35, 0.15, 1.0), 4.0);
                if ui.button("play_ctrl_play", "Play", btn) {
                    action = Some(PlayControlAction::Play);
                }
            }
            EditorPlayState::Playing => {
                let pause_btn = Rect::new(x, y, self.button_size, self.button_size);
                if ui.button("play_ctrl_pause", "Pause", pause_btn) {
                    action = Some(PlayControlAction::Pause);
                }

                let stop_x = x + self.button_size + self.spacing;
                let stop_btn = Rect::new(stop_x, y, self.button_size, self.button_size);
                ui.rect_rounded(stop_btn, Color::new(0.4, 0.15, 0.15, 1.0), 4.0);
                if ui.button("play_ctrl_stop", "Stop", stop_btn) {
                    action = Some(PlayControlAction::Stop);
                }
            }
            EditorPlayState::Paused => {
                let resume_btn = Rect::new(x, y, self.button_size + 10.0, self.button_size);
                ui.rect_rounded(resume_btn, Color::new(0.15, 0.35, 0.15, 1.0), 4.0);
                if ui.button("play_ctrl_resume", "Resume", resume_btn) {
                    action = Some(PlayControlAction::Play);
                }

                let stop_x = x + self.button_size + 10.0 + self.spacing;
                let stop_btn = Rect::new(stop_x, y, self.button_size, self.button_size);
                ui.rect_rounded(stop_btn, Color::new(0.4, 0.15, 0.15, 1.0), 4.0);
                if ui.button("play_ctrl_stop2", "Stop", stop_btn) {
                    action = Some(PlayControlAction::Stop);
                }
            }
        }

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_controls_default() {
        let controls = PlayControls::new();
        assert_eq!(controls.button_size, 40.0);
        assert_eq!(controls.spacing, 4.0);
    }

    #[test]
    fn test_play_control_action_eq() {
        assert_eq!(PlayControlAction::Play, PlayControlAction::Play);
        assert_ne!(PlayControlAction::Play, PlayControlAction::Pause);
        assert_ne!(PlayControlAction::Pause, PlayControlAction::Stop);
    }
}
