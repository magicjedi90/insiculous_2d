//! Editor preferences for persisting editor state across sessions.
//!
//! Stores camera position, zoom level, last opened scene, and grid settings.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Persistent editor preferences saved between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPreferences {
    /// Last camera position (x, y)
    pub camera_position: (f32, f32),
    /// Last camera zoom level
    pub camera_zoom: f32,
    /// Path to the last opened scene file
    pub last_scene_path: Option<String>,
    /// Whether snap-to-grid was enabled
    pub snap_to_grid: bool,
    /// Grid cell size
    pub grid_size: f32,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            camera_position: (0.0, 0.0),
            camera_zoom: 1.0,
            last_scene_path: None,
            snap_to_grid: false,
            grid_size: 32.0,
        }
    }
}

impl EditorPreferences {
    /// Load preferences from a JSON file.
    ///
    /// Returns default preferences if the file doesn't exist or can't be parsed.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save preferences to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preferences: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write preferences file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_preferences_defaults() {
        let prefs = EditorPreferences::default();
        assert_eq!(prefs.camera_position, (0.0, 0.0));
        assert_eq!(prefs.camera_zoom, 1.0);
        assert!(prefs.last_scene_path.is_none());
        assert!(!prefs.snap_to_grid);
        assert_eq!(prefs.grid_size, 32.0);
    }

    #[test]
    fn test_editor_preferences_roundtrip() {
        let prefs = EditorPreferences {
            camera_position: (100.0, 200.0),
            camera_zoom: 2.5,
            last_scene_path: Some("scenes/test.ron".to_string()),
            snap_to_grid: true,
            grid_size: 64.0,
        };

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_editor_prefs.json");

        prefs.save(&path).expect("Failed to save");
        let loaded = EditorPreferences::load(&path);

        assert_eq!(loaded.camera_position, (100.0, 200.0));
        assert_eq!(loaded.camera_zoom, 2.5);
        assert_eq!(loaded.last_scene_path, Some("scenes/test.ron".to_string()));
        assert!(loaded.snap_to_grid);
        assert_eq!(loaded.grid_size, 64.0);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_editor_preferences_load_missing_file() {
        let prefs = EditorPreferences::load(Path::new("/nonexistent/path.json"));
        assert_eq!(prefs.camera_zoom, 1.0); // Should return defaults
    }
}
