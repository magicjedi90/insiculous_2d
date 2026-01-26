//! Scene management for the engine.
//!
//! This module provides the `SceneManager` which manages the scene stack and lifecycle.
//! It handles pushing and popping scenes, and provides access to the active scene.

use crate::Scene;

/// Manages the scene stack and provides access to active scenes.
///
/// The SceneManager maintains a stack of scenes where the top scene is considered
/// the active scene. This allows for scene transitions, pause menus, and layered
/// scene management similar to other game engines.
pub struct SceneManager {
    /// Stack of scenes (0+ scenes)
    scenes: Vec<Scene>,
}

impl SceneManager {
    /// Create a new scene manager with no scenes.
    pub fn new() -> Self {
        Self {
            scenes: Vec::new(),
        }
    }

    /// Create a new scene manager with an initial scene.
    pub fn with_scene(scene: Scene) -> Self {
        Self {
            scenes: vec![scene],
        }
    }

    /// Push a scene onto the stack.
    pub fn push(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }

    /// Pop a scene from the stack.
    ///
    /// Returns the removed scene if the stack was not empty.
    pub fn pop(&mut self) -> Option<Scene> {
        self.scenes.pop()
    }

    /// Get a reference to the active scene (last scene in the stack).
    pub fn active(&self) -> Option<&Scene> {
        self.scenes.last()
    }

    /// Get a mutable reference to the active scene (last scene in the stack).
    pub fn active_mut(&mut self) -> Option<&mut Scene> {
        self.scenes.last_mut()
    }

    /// Get a reference to all scenes in the stack.
    pub fn scenes(&self) -> &[Scene] {
        &self.scenes
    }

    /// Get a mutable reference to all scenes in the stack.
    pub fn scenes_mut(&mut self) -> &mut [Scene] {
        &mut self.scenes
    }

    /// Check if the scene manager has any scenes.
    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }

    /// Get the number of scenes in the stack.
    pub fn len(&self) -> usize {
        self.scenes.len()
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_manager_creation() {
        let manager = SceneManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_scene_manager_with_scene() {
        let scene = Scene::new("test_scene");
        let manager = SceneManager::with_scene(scene);
        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);
        assert!(manager.active().is_some());
    }

    #[test]
    fn test_scene_manager_push_pop() {
        let mut manager = SceneManager::new();
        let scene1 = Scene::new("scene1");
        let scene2 = Scene::new("scene2");

        manager.push(scene1);
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.active().unwrap().name(), "scene1");

        manager.push(scene2);
        assert_eq!(manager.len(), 2);
        assert_eq!(manager.active().unwrap().name(), "scene2");

        let popped = manager.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().name(), "scene2");
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.active().unwrap().name(), "scene1");
    }

    #[test]
    fn test_scene_manager_active_mut() {
        let mut manager = SceneManager::new();
        let scene = Scene::new("test_scene");
        manager.push(scene);

        if let Some(active) = manager.active_mut() {
            // Test that we can access the scene mutably
            assert_eq!(active.name(), "test_scene");
        } else {
            panic!("Expected active scene");
        }
    }

    #[test]
    fn test_scene_manager_scenes_access() {
        let mut manager = SceneManager::new();
        let scene1 = Scene::new("scene1");
        let scene2 = Scene::new("scene2");

        manager.push(scene1);
        manager.push(scene2);

        let scenes = manager.scenes();
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].name(), "scene1");
        assert_eq!(scenes[1].name(), "scene2");

        let scenes_mut = manager.scenes_mut();
        assert_eq!(scenes_mut.len(), 2);
    }
}