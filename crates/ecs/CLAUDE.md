# ECS Crate — Agent Context

You are working in the ECS (Entity Component System) crate. This is the data backbone of the engine.

## Architecture
```
ComponentRegistry (HashMap<TypeId, ComponentStore>)
└── ComponentStore (HashMap<EntityId, Box<dyn Component>>)

Query types: Single<T>, Pair<T, U>, Triple<T, U, V>
```

## Key Types
- `World` — owns entities + ComponentRegistry. All entity/component operations go through here.
- `EntityId` — newtype around u64
- `Component` trait — requires Debug + Any + Send + Sync + Serialize + Deserialize + Clone
- `ComponentMeta` trait — type_name(), field_names() for inspector/registry
- `WorldHierarchyExt` — set_parent(), get_children(), get_descendants(), get_root_entities()

## Built-in Components
- `Transform2D` — position (Vec2), rotation (f32), scale (Vec2)
- `GlobalTransform2D` — computed world-space transform
- `Sprite` — texture_handle, offset, rotation, scale, color, depth, tex_region
- `Camera` / `Camera2D` — viewport, zoom, main camera flag
- `Name` — entity display name
- `RigidBody`, `Collider` — physics (defined here, used by physics crate)
- `AudioSource`, `AudioListener` — audio components
- `SpriteAnimation` — frame-based animation

## File Map
- `world.rs` — World struct, entity/component CRUD
- `component.rs` — Component trait, ComponentStore
- `query.rs` — Type-safe query system (Single, Pair, Triple)
- `hierarchy_ext.rs` — Hierarchy operations (WorldHierarchyExt trait)
- `hierarchy_system.rs` — Transform propagation system
- `component_registry.rs` — Global component type registry
- `sprite_components.rs` — Built-in component definitions

## Critical Patterns
- **Adding components**: `world.add_component(&entity, Transform2D::new(pos)).ok()`
- **Queries**: `world.query_entities::<Pair<Transform2D, Sprite>>()`
- **Two-component access**: `world.get_two_mut::<(Transform2D, Sprite)>(entity)`
- **New components**: derive `DeriveComponentMeta`, register in global registry, add to `WorldSnapshot` known types

## Common Pitfalls
- `Box<dyn Component>` requires Clone via `dyn_clone` — don't forget CloneComponent impl
- TypeId is per-concrete-type — different generic params = different TypeIds
- Always check for circular references when reparenting in hierarchy
- serde_json for inspector, RON for scene files — both must work

## Testing
- 110 tests, run with `cargo test -p ecs`
- Integration tests in `tests/world.rs`, unit tests inline in source
- Naming: `test_<behavior_description>`

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

| Our Concept | Godot Equivalent | File |
|-------------|-----------------|------|
| World / hierarchy | Scene tree | `scene/main/node.cpp` — `add_child`, `remove_child`, `get_children` |
| Component + ComponentMeta | Object properties | `core/object/object.cpp` — `set`, `get`, `get_property_list` |
| Entity duplication | Node::duplicate | `scene/main/node.cpp` — search `duplicate` |
| Entity deletion | Node::queue_free | `scene/main/node.cpp` — search `queue_free`, `remove_child` |
| Transform propagation | Node2D transforms | `scene/2d/node_2d.cpp` — how transforms chain |

**Remember:** Godot uses scene tree + properties. Adapt *design patterns* to our Rust ECS, don't copy C++.
