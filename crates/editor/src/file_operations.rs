//! File operations for the editor (save, load, new scene).

use std::path::{Path, PathBuf};

use ecs::World;
use engine_core::assets::AssetManager;
use engine_core::scene_data::SceneLoadError;
use engine_core::scene_loader::{SceneInstance, SceneLoader};
use engine_core::scene_saver::{SceneSaveError, SceneSaver};

use crate::EditorContext;

/// Extended error type for file operations
#[derive(Debug, thiserror::Error)]
pub enum FileOperationError {
    #[error("No scene path set - use Save As")]
    NoPath,

    #[error("Save error: {0}")]
    SaveError(#[from] SceneSaveError),

    #[error("Load error: {0}")]
    LoadError(#[from] SceneLoadError),
}

/// Save the current scene to its existing path.
///
/// Returns an error if no path is set (use `save_scene_as` instead).
pub fn save_scene(
    editor: &EditorContext,
    world: &World,
    assets: Option<&AssetManager>,
) -> Result<PathBuf, FileOperationError> {
    let path = editor
        .current_scene_path()
        .ok_or(FileOperationError::NoPath)?
        .to_path_buf();

    SceneSaver::save_world_to_file(
        world,
        assets,
        editor.scene_name(),
        Some(editor.to_editor_settings()),
        &path,
    )?;

    Ok(path)
}

/// Save the current scene to a new path.
pub fn save_scene_as(
    editor: &mut EditorContext,
    world: &World,
    assets: Option<&AssetManager>,
    path: PathBuf,
) -> Result<(), FileOperationError> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    SceneSaver::save_world_to_file(
        world,
        assets,
        &name,
        Some(editor.to_editor_settings()),
        &path,
    )?;

    editor.set_current_scene(Some(path), name);
    Ok(())
}

/// Load a scene from a file.
///
/// Note: The caller is responsible for clearing the world before calling this
/// if a clean load is desired.
pub fn load_scene(
    editor: &mut EditorContext,
    world: &mut World,
    assets: &mut AssetManager,
    path: impl AsRef<Path>,
) -> Result<SceneInstance, FileOperationError> {
    let path = path.as_ref();
    let data = SceneLoader::load_from_file(path)?;

    // Apply editor settings if present
    if let Some(settings) = &data.editor {
        editor.apply_editor_settings(settings);
    }

    let name = data.name.clone();
    let instance = SceneLoader::instantiate(&data, world, assets)?;

    editor.set_current_scene(Some(path.to_path_buf()), name);

    // Clear selection since entities changed
    editor.selection.clear();

    Ok(instance)
}

/// Reset editor for a new empty scene.
///
/// Note: The caller is responsible for clearing the world.
pub fn new_scene(editor: &mut EditorContext) {
    editor.set_current_scene(None, "Untitled".to_string());
    editor.selection.clear();
    editor.reset_camera();
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_scene_no_path_returns_error() {
        let editor = EditorContext::new();
        let world = World::default();

        let result = save_scene(&editor, &world, None);

        assert!(matches!(result, Err(FileOperationError::NoPath)));
    }

    #[test]
    fn test_save_scene_as_updates_editor() {
        let mut editor = EditorContext::new();
        let world = World::default();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        save_scene_as(&mut editor, &world, None, path.clone()).unwrap();

        assert_eq!(editor.current_scene_path(), Some(path.as_path()));
    }

    #[test]
    fn test_new_scene_resets_editor() {
        let mut editor = EditorContext::new();

        // Set up some state
        editor.set_current_scene(
            Some(PathBuf::from("/test/scene.ron")),
            "Test".to_string(),
        );
        editor.set_camera_offset(Vec2::new(100.0, 100.0));
        editor.set_camera_zoom(2.0);

        new_scene(&mut editor);

        assert!(editor.current_scene_path().is_none());
        assert_eq!(editor.scene_name(), "Untitled");
        assert_eq!(editor.camera_offset(), Vec2::ZERO);
        assert_eq!(editor.camera_zoom(), 1.0);
    }
}
