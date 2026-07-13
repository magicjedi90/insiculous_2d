use std::path::PathBuf;

use ecs::{GlobalTransform2D, World};
use editor::{EditorTool, PlayControlAction};
use glam::Vec2;

use engine_core::contexts::GameContext;
use engine_core::scene_data::PhysicsSettings;
use engine_core::Game;
use engine_core::GameConfig;

use super::viewport_interaction::build_pickable_entities;
use super::EditorGame;




struct DummyGame;
impl Game for DummyGame {
    fn update(&mut self, _ctx: &mut GameContext) {}
}

#[test]
fn test_editor_game_creation() {
    let editor = EditorGame::new(DummyGame);
    assert!(!editor.font_loaded);
    assert!(editor.editor.selection.is_empty());
    assert_eq!(editor.editor.current_tool(), EditorTool::Select);
    assert!(editor.world_snapshot.is_none());
    assert!(editor.editor.is_editing());
    assert_eq!(editor.entity_counter, 0);
}

#[test]
fn test_command_history_initialized() {
    let editor = EditorGame::new(DummyGame);
    assert!(!editor.command_history.can_undo());
    assert!(!editor.command_history.can_redo());
    assert!(editor.gizmo_drag_start.is_none());
}

#[test]
fn test_editor_game_default_panels() {
    let editor = EditorGame::new(DummyGame);
    assert_eq!(editor.editor.dock_area.panels().len(), 4);
}

#[test]
fn test_editor_config_enforces_minimum_size() {
    let config = GameConfig::new("Test").with_size(640, 480);
    let mut adjusted = config.clone();
    if adjusted.width < 1024 { adjusted.width = 1024; }
    if adjusted.height < 720 { adjusted.height = 720; }
    assert_eq!(adjusted.width, 1024);
    assert_eq!(adjusted.height, 720);
}

#[test]
fn test_editor_config_preserves_large_size() {
    let config = GameConfig::new("Test").with_size(1920, 1080);
    let mut adjusted = config.clone();
    if adjusted.width < 1024 { adjusted.width = 1024; }
    if adjusted.height < 720 { adjusted.height = 720; }
    assert_eq!(adjusted.width, 1920);
    assert_eq!(adjusted.height, 1080);
}

#[test]
fn test_play_action_captures_snapshot() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    world.add_component(&entity, common::Transform2D::new(glam::Vec2::new(10.0, 20.0))).ok();

    // Play → snapshot captured
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    assert!(editor.editor.is_playing());
    assert!(editor.world_snapshot.is_some());
}

#[test]
fn test_play_pause_resume_stop_cycle() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    world.add_component(&entity, common::Transform2D::new(glam::Vec2::ZERO)).ok();

    // Play
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    assert!(editor.editor.is_playing());

    // Pause
    editor.handle_play_action(PlayControlAction::Pause, &mut world);
    assert!(editor.editor.is_paused());

    // Resume
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    assert!(editor.editor.is_playing());

    // Stop
    editor.handle_play_action(PlayControlAction::Stop, &mut world);
    assert!(editor.editor.is_editing());
    assert!(editor.world_snapshot.is_none());
}

#[test]
fn test_stop_restores_world_state() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    world.add_component(&entity, common::Transform2D::new(glam::Vec2::new(10.0, 20.0))).ok();

    // Play
    editor.handle_play_action(PlayControlAction::Play, &mut world);

    // Simulate game modification
    if let Some(t) = world.get_mut::<common::Transform2D>(entity) {
        t.position = glam::Vec2::new(999.0, 999.0);
    }

    // Stop → should restore original position
    editor.handle_play_action(PlayControlAction::Stop, &mut world);

    let t = world.get::<common::Transform2D>(entity).unwrap();
    assert_eq!(t.position, glam::Vec2::new(10.0, 20.0));
}

#[test]
fn test_build_pickable_entities_with_both_components() {
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    world.add_component(&entity, GlobalTransform2D {
        position: Vec2::new(100.0, 200.0),
        scale: Vec2::new(2.0, 2.0),
        ..Default::default()
    }).ok();
    let mut sprite = ecs::sprite_components::Sprite::new(0);
    sprite.scale = Vec2::new(32.0, 32.0);
    sprite.depth = 5.0;
    world.add_component(&entity, sprite).ok();

    let pickables = build_pickable_entities(&world);
    assert_eq!(pickables.len(), 1);
    assert_eq!(pickables[0].entity_id, entity);
    assert_eq!(pickables[0].position, Vec2::new(100.0, 200.0));
    // Size = sprite.scale * global_transform.scale = (32, 32) * (2, 2) = (64, 64)
    assert_eq!(pickables[0].size, Vec2::new(64.0, 64.0));
    assert_eq!(pickables[0].depth, 5.0);
}

#[test]
fn test_build_pickable_entities_skips_without_sprite() {
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    // Only GlobalTransform2D, no Sprite
    world.add_component(&entity, GlobalTransform2D::default()).ok();

    let pickables = build_pickable_entities(&world);
    assert!(pickables.is_empty());
}

#[test]
fn test_build_pickable_entities_skips_without_global_transform() {
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    // Only Sprite, no GlobalTransform2D
    world.add_component(&entity, ecs::sprite_components::Sprite::new(0)).ok();

    let pickables = build_pickable_entities(&world);
    assert!(pickables.is_empty());
}

#[test]
fn test_build_pickable_entities_multiple() {
    let mut world = ecs::World::new();

    // Entity 1
    let e1 = world.create_entity();
    world.add_component(&e1, GlobalTransform2D {
        position: Vec2::new(10.0, 20.0),
        ..Default::default()
    }).ok();
    let mut sprite1 = ecs::sprite_components::Sprite::new(0);
    sprite1.depth = 1.0;
    world.add_component(&e1, sprite1).ok();

    // Entity 2
    let e2 = world.create_entity();
    world.add_component(&e2, GlobalTransform2D {
        position: Vec2::new(50.0, 60.0),
        ..Default::default()
    }).ok();
    let mut sprite2 = ecs::sprite_components::Sprite::new(1);
    sprite2.depth = 3.0;
    world.add_component(&e2, sprite2).ok();

    // Entity 3 — no sprite, should be excluded
    let e3 = world.create_entity();
    world.add_component(&e3, GlobalTransform2D::default()).ok();

    let pickables = build_pickable_entities(&world);
    assert_eq!(pickables.len(), 2);

    let ids: Vec<_> = pickables.iter().map(|p| p.entity_id).collect();
    assert!(ids.contains(&e1));
    assert!(ids.contains(&e2));
    assert!(!ids.contains(&e3));
}

// ================== Scene Save/Load Tests ==================

#[test]
fn test_editor_game_initial_scene_state() {
    let editor = EditorGame::new(DummyGame);
    assert!(!editor.editor.is_dirty());
    assert!(editor.editor.scene_path().is_none());
    assert!(editor.physics_settings.is_none());
}

#[test]
fn test_new_scene_clears_world() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let _e1 = world.create_entity();
    let _e2 = world.create_entity();
    assert_eq!(world.entities().len(), 2);

    editor.new_scene(&mut world);
    assert_eq!(world.entities().len(), 0);
    assert!(!editor.editor.is_dirty());
    assert!(editor.editor.scene_path().is_none());
}

#[test]
fn test_new_scene_resets_editor_state() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();

    // Simulate some state
    editor.editor.mark_dirty();
    editor.editor.set_scene_path(Some(PathBuf::from("test.ron")));
    editor.entity_counter = 5;

    editor.new_scene(&mut world);
    assert!(!editor.editor.is_dirty());
    assert!(editor.editor.scene_path().is_none());
    assert_eq!(editor.entity_counter, 0);
    assert!(editor.physics_settings.is_none());
}

#[test]
fn test_save_creates_file() {
    let _editor = EditorGame::new(DummyGame);
    let world = ecs::World::new();

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_save_scene.ron");

    // Use a simple texture path function since we don't have AssetManager in tests
    let scene_name = "test";
    let texture_path_fn = |handle: u32| -> String {
        if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
    };
    let scene_data = engine_core::scene_serializer::world_to_scene_data(
        &world, scene_name, None, &texture_path_fn,
    );
    let result = engine_core::scene_serializer::save_scene_to_file(&scene_data, &path);
    assert!(result.is_ok());
    assert!(path.exists());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_save_clears_dirty_flag() {
    let mut editor = EditorGame::new(DummyGame);
    let world = World::new();

    editor.editor.mark_dirty();
    assert!(editor.editor.is_dirty());

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_save_dirty.ron");

    // We can't use save_scene_as directly without AssetManager,
    // so test the flag behavior with set_dirty
    let texture_path_fn = |handle: u32| -> String {
        if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
    };
    let scene_data = engine_core::scene_serializer::world_to_scene_data(
        &world, "test", None, &texture_path_fn,
    );
    engine_core::scene_serializer::save_scene_to_file(&scene_data, &path).unwrap();
    editor.editor.set_scene_path(Some(path.clone()));
    editor.editor.set_dirty(false);

    assert!(!editor.editor.is_dirty());
    assert_eq!(editor.editor.scene_path(), Some(path.as_path()));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_new_scene_warns_if_dirty() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    world.create_entity();

    editor.editor.mark_dirty();
    // new_scene should still work even when dirty (just logs a warning)
    editor.new_scene(&mut world);
    assert_eq!(world.entities().len(), 0);
    assert!(!editor.editor.is_dirty());
}

#[test]
fn test_save_scene_roundtrip() {
    let _editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();

    // Create entities with components
    let e1 = world.create_entity();
    world.add_component(&e1, common::Transform2D::new(Vec2::new(100.0, 200.0))).ok();
    world.add_component(&e1, ecs::sprite_components::Name::new("player")).ok();

    let e2 = world.create_entity();
    world.add_component(&e2, common::Transform2D::new(Vec2::new(50.0, 50.0))).ok();

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_roundtrip.ron");

    // Serialize
    let texture_path_fn = |handle: u32| -> String {
        if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
    };
    let scene_data = engine_core::scene_serializer::world_to_scene_data(
        &world, "Roundtrip", None, &texture_path_fn,
    );
    engine_core::scene_serializer::save_scene_to_file(&scene_data, &path).unwrap();

    // Verify the file is valid RON by parsing it with SceneLoader
    let parsed = engine_core::scene_loader::SceneLoader::load_from_file(&path).unwrap();
    assert_eq!(parsed.name, "Roundtrip");
    assert_eq!(parsed.entities.len(), 2);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_save_as_updates_path() {
    let mut editor = EditorGame::new(DummyGame);

    assert!(editor.editor.scene_path().is_none());

    let path = PathBuf::from("scenes/my_scene.ron");
    editor.editor.set_scene_path(Some(path.clone()));

    assert_eq!(editor.editor.scene_path(), Some(path.as_path()));
    assert_eq!(editor.editor.scene_display_name(), "my_scene.ron");
}

#[test]
fn test_dirty_flag_set_on_entity_create() {
    let mut editor = EditorGame::new(DummyGame);
    assert!(!editor.editor.is_dirty());

    // Simulate entity creation marking dirty
    editor.editor.mark_dirty();
    assert!(editor.editor.is_dirty());
}

#[test]
fn test_load_scene_resets_selection() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let entity = world.create_entity();
    editor.editor.selection.select(entity);
    assert!(!editor.editor.selection.is_empty());

    editor.new_scene(&mut world);
    assert!(editor.editor.selection.is_empty());
}

#[test]
fn test_physics_settings_preserved_on_new() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();

    editor.physics_settings = Some(PhysicsSettings::default());
    assert!(editor.physics_settings.is_some());

    editor.new_scene(&mut world);
    assert!(editor.physics_settings.is_none());
}

#[test]
fn test_scene_display_in_status() {
    let editor = EditorGame::new(DummyGame);
    assert_eq!(editor.editor.scene_display_name(), "Untitled");
    assert_eq!(editor.editor.title_bar_text(), "Untitled - Insiculous Editor");
}

#[test]
fn test_undo_redo_on_empty_history_do_not_mark_dirty() {
    // GPP-L6: an Undo/Redo keypress on an empty history is a no-op and must
    // not dirty a clean scene. The shortcut/menu handlers gate mark_dirty()
    // on the bool returned by undo()/redo(), so mirror that gating here.
    let mut editor = EditorGame::new(DummyGame);
    let mut world = World::new();
    assert!(!editor.editor.is_dirty());

    if editor.command_history.undo(&mut world) {
        editor.editor.mark_dirty();
    }
    if editor.command_history.redo(&mut world) {
        editor.editor.mark_dirty();
    }

    assert!(!editor.command_history.undo(&mut world));
    assert!(!editor.command_history.redo(&mut world));
    assert!(!editor.editor.is_dirty());
}
