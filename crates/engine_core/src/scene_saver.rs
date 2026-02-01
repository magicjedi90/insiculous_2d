//! Scene saving for serializing World entities to RON files.

use std::collections::HashMap;

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
        _world: &World,
        _assets: Option<&AssetManager>,
        _entity: EntityId,
    ) -> Vec<ComponentData> {
        // TODO: Implement component extraction in next task
        Vec::new()
    }

    fn extract_name(world: &World, entity: EntityId) -> Option<String> {
        world.get::<Name>(entity).map(|n| n.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_empty_world() {
        let world = World::default();
        let scene = SceneSaver::extract_from_world(&world, None, "Empty");

        assert_eq!(scene.name, "Empty");
        assert!(scene.entities.is_empty());
        assert!(scene.prefabs.is_empty());
        assert!(scene.editor.is_none());
    }
}
