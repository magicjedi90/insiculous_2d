//! Editor status bar displayed at the bottom of the window.
//!
//! Shows contextual status messages (left), runtime stats (center),
//! and version info (right). Status messages auto-clear after a timeout.

use glam::Vec2;
use ui::{Rect, UIContext};

use crate::theme::EditorTheme;

/// Height of the status bar in pixels.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;

/// Duration in seconds before a status message auto-clears.
const MESSAGE_TIMEOUT: f32 = 3.0;

/// Runtime statistics for the status bar center section.
#[derive(Debug, Clone, Default)]
pub struct StatusBarStats {
    /// Number of entities in the world.
    pub entity_count: usize,
    /// Smoothed frames-per-second.
    pub fps: f32,
}

/// The editor status bar widget.
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Current status message (left section).
    message: Option<String>,
    /// Time remaining before the message auto-clears (seconds).
    message_timer: f32,
    /// Whether the current message should persist (errors).
    message_persistent: bool,
    /// Version string (right section).
    version: String,
    /// Runtime stats updated each frame.
    stats: StatusBarStats,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new() -> Self {
        Self {
            message: None,
            message_timer: 0.0,
            message_persistent: false,
            version: String::from("v0.1.0"),
            stats: StatusBarStats::default(),
        }
    }

    /// Set the version string displayed on the right.
    pub fn set_version(&mut self, version: impl Into<String>) {
        self.version = version.into();
    }

    /// Show a temporary status message (auto-clears after 3 seconds).
    pub fn show_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
        self.message_timer = MESSAGE_TIMEOUT;
        self.message_persistent = false;
    }

    /// Show a persistent status message (stays until explicitly cleared).
    pub fn show_error(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
        self.message_timer = 0.0;
        self.message_persistent = true;
    }

    /// Clear the current status message.
    pub fn clear_message(&mut self) {
        self.message = None;
        self.message_timer = 0.0;
        self.message_persistent = false;
    }

    /// Get the current message, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Update runtime stats.
    pub fn update_stats(&mut self, entity_count: usize, fps: f32) {
        self.stats.entity_count = entity_count;
        self.stats.fps = fps;
    }

    /// Tick the message timer. Call once per frame with delta time.
    pub fn update(&mut self, delta_time: f32) {
        if !self.message_persistent && self.message.is_some() {
            self.message_timer -= delta_time;
            if self.message_timer <= 0.0 {
                self.message = None;
                self.message_timer = 0.0;
            }
        }
    }

    /// Render the status bar. Returns the bar bounds for layout purposes.
    ///
    /// The status bar is positioned at the bottom of the given `window_size`.
    pub fn render(
        &self,
        ui: &mut UIContext,
        window_size: Vec2,
        theme: &EditorTheme,
    ) -> Rect {
        let bar = Rect::new(0.0, window_size.y - STATUS_BAR_HEIGHT, window_size.x, STATUS_BAR_HEIGHT);

        // Background
        ui.rect(bar, theme.status_bar_bg);

        // Top border line
        ui.line(
            Vec2::new(bar.x, bar.y),
            Vec2::new(bar.x + bar.width, bar.y),
            theme.border_subtle,
            1.0,
        );

        let text_y = bar.y + 4.0;
        let padding = 8.0;

        // Left section: status message
        let status_text = self.message.as_deref().unwrap_or("Ready");
        let status_color = if self.message_persistent {
            theme.error_red
        } else {
            theme.text_secondary
        };
        ui.label_styled(status_text, Vec2::new(bar.x + padding, text_y), status_color, 11.0);

        // Center section: runtime stats
        let stats_text = format!(
            "Objects: {} | FPS: {:.0}",
            self.stats.entity_count,
            self.stats.fps,
        );
        let center_x = bar.x + bar.width / 2.0 - 80.0;
        ui.label_styled(&stats_text, Vec2::new(center_x, text_y), theme.text_muted, 11.0);

        // Right section: version
        let version_x = bar.x + bar.width - padding - (self.version.len() as f32 * 7.0);
        ui.label_styled(&self.version, Vec2::new(version_x, text_y), theme.accent_cyan, 11.0);

        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_default() {
        let bar = StatusBar::new();
        assert!(bar.message().is_none());
        assert_eq!(bar.stats.entity_count, 0);
        assert_eq!(bar.stats.fps, 0.0);
    }

    #[test]
    fn test_show_message() {
        let mut bar = StatusBar::new();
        bar.show_message("Entity created");
        assert_eq!(bar.message(), Some("Entity created"));
    }

    #[test]
    fn test_message_auto_clears() {
        let mut bar = StatusBar::new();
        bar.show_message("Saved");

        // Tick partially — message still visible
        bar.update(1.0);
        assert!(bar.message().is_some());

        // Tick past timeout — message cleared
        bar.update(3.0);
        assert!(bar.message().is_none());
    }

    #[test]
    fn test_persistent_error_message() {
        let mut bar = StatusBar::new();
        bar.show_error("Failed to save");

        // Even after a long time, error persists
        bar.update(100.0);
        assert_eq!(bar.message(), Some("Failed to save"));

        // Explicit clear removes it
        bar.clear_message();
        assert!(bar.message().is_none());
    }

    #[test]
    fn test_update_stats() {
        let mut bar = StatusBar::new();
        bar.update_stats(42, 60.0);
        assert_eq!(bar.stats.entity_count, 42);
        assert_eq!(bar.stats.fps, 60.0);
    }

    #[test]
    fn test_set_version() {
        let mut bar = StatusBar::new();
        bar.set_version("v2.0.1 - Stable");
        assert_eq!(bar.version, "v2.0.1 - Stable");
    }

    #[test]
    fn test_show_message_resets_timer() {
        let mut bar = StatusBar::new();
        bar.show_message("First");
        bar.update(2.0); // 1 second left

        bar.show_message("Second");
        // Timer should be reset to full duration
        bar.update(2.0); // Should still be visible
        assert_eq!(bar.message(), Some("Second"));

        bar.update(2.0); // Now it should be gone
        assert!(bar.message().is_none());
    }

    #[test]
    fn test_clear_message_stops_timer() {
        let mut bar = StatusBar::new();
        bar.show_message("Temp");
        bar.clear_message();
        assert!(bar.message().is_none());
        assert!(!bar.message_persistent);
    }
}
