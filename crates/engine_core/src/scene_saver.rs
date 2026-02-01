//! Scene saving for serializing World entities to RON files.

use std::collections::HashMap;

use ecs::sprite_components::{Sprite, Transform2D};
use ecs::{EntityId, Name, World, WorldHierarchyExt};

use crate::assets::AssetManager;
use crate::scene_data::{ComponentData, EntityData, SceneData};

/// Scene saver for extracting world state to RON files.
pub struct SceneSaver;

impl SceneSaver {
    /// Extract all entities from world into SceneData.
    ///
    /// If `assets` is None, texture paths will default to "#white".
    pub fn extract_from_world(
        world: &World,
        assets: Option<&AssetManager>,
        scene_name: &str,
    ) -> SceneData {
        let mut entities = Vec::new();

        // Get root entities (those without parents)
        for root_id in world.get_root_entities() {
            if let Some(entity_data) = Self::extract_entity_recursive(world, assets, root_id) {
                entities.push(entity_data);
            }
        }

        SceneData {
            name: scene_name.to_string(),
            physics: None,
            editor: None,
            prefabs: HashMap::new(),
            entities,
        }
    }

    fn extract_entity_recursive(
        world: &World,
        assets: Option<&AssetManager>,
        entity: EntityId,
    ) -> Option<EntityData> {
        let components = Self::extract_components(world, assets, entity);

        // Recursively extract children
        let children: Vec<EntityData> = world
            .get_children(entity)
            .map(|child_ids| {
                child_ids
                    .iter()
                    .filter_map(|&child| Self::extract_entity_recursive(world, assets, child))
                    .collect()
            })
            .unwrap_or_default();

        Some(EntityData {
            name: Self::extract_name(world, entity),
            prefab: None,
            parent: None,
            overrides: Vec::new(),
            components,
            children,
        })
    }

    fn extract_components(
        world: &World,
        assets: Option<&AssetManager>,
        entity: EntityId,
    ) -> Vec<ComponentData> {
        let mut components = Vec::new();

        // Transform2D
        if let Some(t) = world.get::<Transform2D>(entity) {
            components.push(ComponentData::Transform2D {
                position: (t.position.x, t.position.y),
                rotation: t.rotation,
                scale: (t.scale.x, t.scale.y),
            });
        }

        // Sprite
        if let Some(s) = world.get::<Sprite>(entity) {
            let texture = assets
                .and_then(|a| a.get_texture_path(s.texture_handle))
                .unwrap_or("#white")
                .to_string();

            components.push(ComponentData::Sprite {
                texture,
                offset: (s.offset.x, s.offset.y),
                rotation: s.rotation,
                scale: (s.scale.x, s.scale.y),
                color: (s.color.x, s.color.y, s.color.z, s.color.w),
                depth: s.depth,
            });
        }

        components
    }

    fn extract_name(world: &World, entity: EntityId) -> Option<String> {
        world.get::<Name>(entity).map(|n| n.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::sprite_components::{Sprite, Transform2D};
    use glam::{Vec2, Vec4};

    #[test]
    fn test_extract_empty_world() {
        let world = World::default();
        let scene = SceneSaver::extract_from_world(&world, None, "Empty");

        assert_eq!(scene.name, "Empty");
        assert!(scene.entities.is_empty());
        assert!(scene.prefabs.is_empty());
        assert!(scene.editor.is_none());
    }

    #[test]
    fn test_extract_entity_with_transform() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                Transform2D {
                    position: Vec2::new(100.0, 200.0),
                    rotation: 1.5,
                    scale: Vec2::new(2.0, 3.0),
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Transform2D {
                position,
                rotation,
                scale,
            } => {
                assert_eq!(*position, (100.0, 200.0));
                assert_eq!(*rotation, 1.5);
                assert_eq!(*scale, (2.0, 3.0));
            }
            _ => panic!("Expected Transform2D"),
        }
    }

    #[test]
    fn test_extract_entity_with_sprite() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                Sprite {
                    texture_handle: 0,
                    offset: Vec2::new(5.0, 10.0),
                    rotation: 0.5,
                    scale: Vec2::new(1.0, 1.0),
                    color: Vec4::new(1.0, 0.5, 0.25, 1.0),
                    depth: 5.0,
                    tex_region: [0.0, 0.0, 1.0, 1.0],
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Sprite {
                texture,
                offset,
                rotation,
                scale: _,
                color,
                depth,
            } => {
                assert_eq!(texture, "#white");
                assert_eq!(*offset, (5.0, 10.0));
                assert_eq!(*rotation, 0.5);
                assert_eq!(*color, (1.0, 0.5, 0.25, 1.0));
                assert_eq!(*depth, 5.0);
            }
            _ => panic!("Expected Sprite"),
        }
    }
}
