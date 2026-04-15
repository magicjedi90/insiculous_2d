//! Achievement / trophy system.
//!
//! Engine-wide, game-agnostic: games register their own achievements at
//! startup and call `unlock(id)` when conditions are met. Unlocked state is
//! persisted to a JSON file so it survives restarts. A toast pops in via the
//! UI system when an achievement is unlocked for the first time.
//!
//! # Example
//! ```ignore
//! use engine_core::prelude::*;
//!
//! fn init(&mut self, ctx: &mut GameContext) {
//!     ctx.achievements.register(Achievement::new(
//!         "first_blood",
//!         "First Blood",
//!         "Defeat your first enemy",
//!     ));
//! }
//!
//! fn update(&mut self, ctx: &mut GameContext) {
//!     if enemy_defeated {
//!         ctx.achievements.unlock("first_blood");
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use ui::UIContext;
use common::{Color, Rect};

/// An achievement definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    /// Stable identifier (e.g. `"first_blood"`). Used for unlocking and persistence.
    pub id: String,
    /// Display name shown on toast and in menus.
    pub name: String,
    /// Longer description of how to earn it.
    pub description: String,
    /// If true, name/description stay hidden until unlocked (secret achievement).
    pub hidden: bool,
}

impl Achievement {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            hidden: false,
        }
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

/// Per-achievement unlock record (persisted).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnlockRecord {
    /// Unix timestamp in seconds when the achievement was unlocked.
    unlocked_at: u64,
}

/// On-disk save format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SaveFile {
    unlocks: HashMap<String, UnlockRecord>,
}

/// Active toast being displayed.
#[derive(Debug, Clone)]
struct Toast {
    achievement_id: String,
    name: String,
    description: String,
    remaining: f32,
}

/// Errors from achievement persistence.
#[derive(Debug, thiserror::Error)]
pub enum AchievementError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Default time (seconds) a toast stays visible before fading out.
pub const DEFAULT_TOAST_DURATION: f32 = 4.0;

/// Manages achievement registration, unlocking, persistence, and toasts.
pub struct AchievementManager {
    /// Registered achievement definitions, keyed by id.
    registered: HashMap<String, Achievement>,
    /// Unlock records loaded from disk / accumulated this session.
    unlocks: HashMap<String, UnlockRecord>,
    /// Toasts queued for display (FIFO).
    toasts: Vec<Toast>,
    /// Path to persist unlocks to. `None` disables persistence (useful for tests).
    save_path: Option<PathBuf>,
    /// How long each toast stays on screen.
    toast_duration: f32,
}

impl AchievementManager {
    /// Create a manager with no persistence (in-memory only).
    pub fn in_memory() -> Self {
        Self {
            registered: HashMap::new(),
            unlocks: HashMap::new(),
            toasts: Vec::new(),
            save_path: None,
            toast_duration: DEFAULT_TOAST_DURATION,
        }
    }

    /// Create a manager that persists unlocks to the given JSON file.
    ///
    /// If the file already exists, previously unlocked achievements are loaded.
    /// Missing file is treated as "nothing unlocked yet" (not an error).
    pub fn with_save_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut mgr = Self::in_memory();
        mgr.save_path = Some(path.clone());
        if path.exists() {
            if let Err(e) = mgr.load() {
                log::warn!("Failed to load achievements from {}: {}", path.display(), e);
            }
        }
        mgr
    }

    /// Override the toast duration (seconds).
    pub fn set_toast_duration(&mut self, seconds: f32) {
        self.toast_duration = seconds;
    }

    /// Register an achievement definition. Call once per achievement at startup.
    ///
    /// Registering the same id twice overwrites the previous definition.
    pub fn register(&mut self, achievement: Achievement) {
        self.registered.insert(achievement.id.clone(), achievement);
    }

    /// Returns the definition for an id, if registered.
    pub fn get(&self, id: &str) -> Option<&Achievement> {
        self.registered.get(id)
    }

    /// All registered achievements (order not guaranteed).
    pub fn all(&self) -> impl Iterator<Item = &Achievement> {
        self.registered.values()
    }

    /// Number of registered achievements.
    pub fn total(&self) -> usize {
        self.registered.len()
    }

    /// Number of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.unlocks.len()
    }

    /// True if the achievement with this id is unlocked.
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocks.contains_key(id)
    }

    /// Unlock an achievement by id. Returns true if this call actually unlocked
    /// it (i.e. it wasn't already unlocked). Idempotent — calling repeatedly is
    /// safe and only shows the toast once.
    ///
    /// If the id is not registered, this logs a warning and returns false.
    pub fn unlock(&mut self, id: &str) -> bool {
        if self.unlocks.contains_key(id) {
            return false;
        }
        let Some(def) = self.registered.get(id) else {
            log::warn!("unlock() called for unregistered achievement: {}", id);
            return false;
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.unlocks
            .insert(id.to_string(), UnlockRecord { unlocked_at: now });

        self.toasts.push(Toast {
            achievement_id: id.to_string(),
            name: def.name.clone(),
            description: def.description.clone(),
            remaining: self.toast_duration,
        });

        if let Some(path) = &self.save_path {
            let path = path.clone();
            if let Err(e) = self.save_to(&path) {
                log::warn!("Failed to save achievements: {}", e);
            }
        }

        log::info!("Achievement unlocked: {} ({})", def.name, id);
        true
    }

    /// Wipe all unlocked state (and persist the empty state if a save path is set).
    /// Typically used for dev/QA or a "reset progress" menu.
    pub fn reset(&mut self) {
        self.unlocks.clear();
        self.toasts.clear();
        if let Some(path) = &self.save_path {
            let path = path.clone();
            let _ = self.save_to(&path);
        }
    }

    /// Advance toast timers. Called once per frame.
    pub fn tick(&mut self, delta_time: f32) {
        for toast in &mut self.toasts {
            toast.remaining -= delta_time;
        }
        self.toasts.retain(|t| t.remaining > 0.0);
    }

    /// Draw any active toasts in the top-right corner of the window.
    ///
    /// Toasts fade out over their last second of life.
    pub fn draw_toasts(&self, ui: &mut UIContext, window_size: Vec2) {
        const WIDTH: f32 = 320.0;
        const HEIGHT: f32 = 72.0;
        const MARGIN: f32 = 16.0;
        const SPACING: f32 = 8.0;

        for (i, toast) in self.toasts.iter().enumerate() {
            let alpha = (toast.remaining / 1.0).clamp(0.0, 1.0);
            let x = window_size.x - WIDTH - MARGIN;
            let y = MARGIN + (HEIGHT + SPACING) * i as f32;

            let bg = Color::new(0.08, 0.08, 0.12, 0.92 * alpha);
            let border = Color::new(1.0, 0.82, 0.2, alpha);
            ui.panel_styled(Rect::new(x, y, WIDTH, HEIGHT), bg, border, 2.0);

            let title_color = Color::new(1.0, 0.82, 0.2, alpha);
            ui.label_styled(
                "Achievement Unlocked!",
                Vec2::new(x + 12.0, y + 10.0),
                title_color,
                14.0,
            );
            ui.label_styled(
                &toast.name,
                Vec2::new(x + 12.0, y + 30.0),
                Color::new(1.0, 1.0, 1.0, alpha),
                16.0,
            );
            ui.label_styled(
                &toast.description,
                Vec2::new(x + 12.0, y + 52.0),
                Color::new(0.8, 0.8, 0.85, alpha),
                12.0,
            );
            let _ = toast.achievement_id; // reserved for future icon lookup
        }
    }

    /// Persist current unlock state to the configured save path.
    /// Returns `Ok(false)` with no action if no save path is configured.
    pub fn save(&self) -> Result<bool, AchievementError> {
        let Some(path) = &self.save_path else { return Ok(false); };
        self.save_to(path)?;
        Ok(true)
    }

    fn save_to(&self, path: &Path) -> Result<(), AchievementError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let save = SaveFile { unlocks: self.unlocks.clone() };
        let json = serde_json::to_string_pretty(&save)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Reload unlock state from the configured save path, discarding any
    /// in-memory unlocks. Errors if no path is set.
    pub fn load(&mut self) -> Result<(), AchievementError> {
        let Some(path) = &self.save_path else {
            return Err(AchievementError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no save path configured",
            )));
        };
        let data = std::fs::read_to_string(path)?;
        let save: SaveFile = serde_json::from_str(&data)?;
        self.unlocks = save.unlocks;
        Ok(())
    }
}

impl Default for AchievementManager {
    fn default() -> Self {
        Self::in_memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample() -> Achievement {
        Achievement::new("test_id", "Test Achievement", "Do the test thing")
    }

    #[test]
    fn in_memory_manager_starts_empty() {
        let mgr = AchievementManager::in_memory();
        assert_eq!(mgr.total(), 0);
        assert_eq!(mgr.unlocked_count(), 0);
    }

    #[test]
    fn register_then_get() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        assert_eq!(mgr.total(), 1);
        assert_eq!(mgr.get("test_id").unwrap().name, "Test Achievement");
    }

    #[test]
    fn unlock_returns_true_first_time_false_after() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        assert!(mgr.unlock("test_id"));
        assert!(!mgr.unlock("test_id"));
        assert_eq!(mgr.unlocked_count(), 1);
    }

    #[test]
    fn unlock_unregistered_returns_false() {
        let mut mgr = AchievementManager::in_memory();
        assert!(!mgr.unlock("never_registered"));
        assert_eq!(mgr.unlocked_count(), 0);
    }

    #[test]
    fn is_unlocked_tracks_state() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        assert!(!mgr.is_unlocked("test_id"));
        mgr.unlock("test_id");
        assert!(mgr.is_unlocked("test_id"));
    }

    #[test]
    fn unlock_queues_one_toast() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        assert_eq!(mgr.toasts.len(), 0);
        mgr.unlock("test_id");
        assert_eq!(mgr.toasts.len(), 1);
        // Second unlock attempt does not queue another toast.
        mgr.unlock("test_id");
        assert_eq!(mgr.toasts.len(), 1);
    }

    #[test]
    fn tick_expires_toasts() {
        let mut mgr = AchievementManager::in_memory();
        mgr.set_toast_duration(2.0);
        mgr.register(sample());
        mgr.unlock("test_id");
        assert_eq!(mgr.toasts.len(), 1);
        mgr.tick(1.0);
        assert_eq!(mgr.toasts.len(), 1);
        mgr.tick(1.5);
        assert_eq!(mgr.toasts.len(), 0);
    }

    #[test]
    fn reset_clears_unlocks_and_toasts() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        mgr.unlock("test_id");
        mgr.reset();
        assert_eq!(mgr.unlocked_count(), 0);
        assert_eq!(mgr.toasts.len(), 0);
    }

    #[test]
    fn persistence_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ach.json");

        {
            let mut mgr = AchievementManager::with_save_path(&path);
            mgr.register(sample());
            mgr.register(Achievement::new("second", "Second", "Do it again"));
            mgr.unlock("test_id");
        }

        assert!(path.exists(), "save file should have been written");

        let mut restored = AchievementManager::with_save_path(&path);
        restored.register(sample());
        restored.register(Achievement::new("second", "Second", "Do it again"));
        assert!(restored.is_unlocked("test_id"));
        assert!(!restored.is_unlocked("second"));
        assert_eq!(restored.unlocked_count(), 1);
    }

    #[test]
    fn persistence_creates_parent_dir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/subdir/ach.json");
        let mut mgr = AchievementManager::with_save_path(&path);
        mgr.register(sample());
        mgr.unlock("test_id");
        assert!(path.exists());
    }

    #[test]
    fn missing_save_file_is_not_an_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist.json");
        let mgr = AchievementManager::with_save_path(&path);
        assert_eq!(mgr.unlocked_count(), 0);
    }

    #[test]
    fn hidden_achievement_flag_persists() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(Achievement::new("secret", "Secret", "Find it").hidden());
        assert!(mgr.get("secret").unwrap().hidden);
    }

    #[test]
    fn all_iterator_yields_registered() {
        let mut mgr = AchievementManager::in_memory();
        mgr.register(sample());
        mgr.register(Achievement::new("second", "Second", "Desc"));
        let ids: std::collections::HashSet<_> = mgr.all().map(|a| a.id.as_str()).collect();
        assert!(ids.contains("test_id"));
        assert!(ids.contains("second"));
    }
}
