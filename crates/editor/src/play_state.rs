//! Editor play state for controlling game simulation.
//!
//! Defines the three-state lifecycle for running a game inside the editor:
//! `Editing` (default), `Playing` (game logic runs), and `Paused` (frozen).

use ui::Color;

/// The current play state of the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorPlayState {
    /// Normal editing mode — game logic does not run.
    #[default]
    Editing,
    /// Game is running inside the editor.
    Playing,
    /// Game is paused — simulation frozen, state preserved.
    Paused,
}

impl EditorPlayState {
    /// Whether the editor is in normal editing mode.
    pub fn is_editing(self) -> bool {
        self == EditorPlayState::Editing
    }

    /// Whether the game simulation is actively running.
    pub fn is_playing(self) -> bool {
        self == EditorPlayState::Playing
    }

    /// Whether the game is paused.
    pub fn is_paused(self) -> bool {
        self == EditorPlayState::Paused
    }

    /// Whether a play session is active (Playing or Paused).
    ///
    /// Returns `true` when a snapshot exists and the world has been modified
    /// by game logic (even if currently paused).
    pub fn in_play_session(self) -> bool {
        matches!(self, EditorPlayState::Playing | EditorPlayState::Paused)
    }

    /// Human-readable label for the current state.
    pub fn label(self) -> &'static str {
        match self {
            EditorPlayState::Editing => "Editing",
            EditorPlayState::Playing => "Playing",
            EditorPlayState::Paused => "Paused",
        }
    }

    /// Border color for the viewport tint.
    ///
    /// Blue while editing, green while playing, yellow while paused.
    pub fn border_color(self) -> Color {
        match self {
            EditorPlayState::Editing => Color::new(0.3, 0.4, 0.7, 0.5),
            EditorPlayState::Playing => Color::new(0.2, 0.8, 0.2, 0.8),
            EditorPlayState::Paused => Color::new(0.9, 0.8, 0.1, 0.8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_editing() {
        let state = EditorPlayState::default();
        assert_eq!(state, EditorPlayState::Editing);
        assert!(state.is_editing());
        assert!(!state.is_playing());
        assert!(!state.is_paused());
    }

    #[test]
    fn test_playing_state() {
        let state = EditorPlayState::Playing;
        assert!(!state.is_editing());
        assert!(state.is_playing());
        assert!(!state.is_paused());
    }

    #[test]
    fn test_paused_state() {
        let state = EditorPlayState::Paused;
        assert!(!state.is_editing());
        assert!(!state.is_playing());
        assert!(state.is_paused());
    }

    #[test]
    fn test_in_play_session() {
        assert!(!EditorPlayState::Editing.in_play_session());
        assert!(EditorPlayState::Playing.in_play_session());
        assert!(EditorPlayState::Paused.in_play_session());
    }

    #[test]
    fn test_labels() {
        assert_eq!(EditorPlayState::Editing.label(), "Editing");
        assert_eq!(EditorPlayState::Playing.label(), "Playing");
        assert_eq!(EditorPlayState::Paused.label(), "Paused");
    }

    #[test]
    fn test_border_colors_are_distinct() {
        let editing = EditorPlayState::Editing.border_color();
        let playing = EditorPlayState::Playing.border_color();
        let paused = EditorPlayState::Paused.border_color();
        // All three states produce different colors
        assert_ne!(editing.r, playing.r);
        assert_ne!(playing.r, paused.r);
        assert_ne!(editing.r, paused.r);
    }
}
