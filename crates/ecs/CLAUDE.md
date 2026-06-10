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
- `Component` trait — blanket impl over `Any + Send + Sync` (component.rs:22). Any such type is a Component automatically; the trait only adds `type_name()` / `as_any()` / `as_any_mut()` for downcasting. It does NOT require Debug, Serialize, Deserialize, or Clone (those are needed separately for scene serialization, inspector, and snapshots)
- `ComponentMeta` trait — type_name(), field_names() for inspector/registry
- `WorldHierarchyExt` — set_parent(), get_children(), get_descendants(), get_root_entities()

## Built-in Components
- `Transform2D` — position (Vec2), rotation (f32), scale (Vec2)
- `GlobalTransform2D` — computed world-space transform
- `Sprite` — texture_handle, offset, rotation, scale, color, depth, tex_region
- `Camera` / `Camera2D` — viewport, zoom, main camera flag
- `Name` — entity display name
- `AudioSource`, `AudioListener` — audio components
- `SpriteAnimation` — frame-based animation

Note: `RigidBody` and `Collider` are NOT defined in this crate — they live in
`crates/physics/src/components.rs`. They are stored in the ecs `World` as
components like any other type, but the physics crate owns their definitions.

## File Map
- `world.rs` — World struct, entity/component CRUD
- `component.rs` — Component trait, ComponentStore
- `query.rs` — Type-safe query system (Single, Pair, Triple)
- `hierarchy_extension.rs` — Hierarchy operations (WorldHierarchyExt trait)
- `hierarchy_system.rs` — Transform propagation system
- `component_registry.rs` — Global component type registry
- `sprite_components.rs` — Built-in component definitions

## Critical Patterns
- **Adding components**: `world.add_component(&entity, Transform2D::new(pos)).ok()`
- **Queries**: `world.query_entities::<Pair<Transform2D, Sprite>>()`
- **Typed access**: `world.get::<Transform2D>(entity)` / `world.get_mut::<Sprite>(entity)` — take `EntityId` by value, return `Option`. There is no `get_two_mut`; to touch two components on one entity, read what you need from the first (`get`), then `get_mut` the second sequentially:
  ```rust
  let offset = world.get::<Sprite>(entity).map(|s| s.offset);
  if let (Some(offset), Some(transform)) = (offset, world.get_mut::<Transform2D>(entity)) {
      transform.position += offset;
  }
  ```
- **New components**: derive `DeriveComponentMeta`, register in global registry, add to `WorldSnapshot` known types

## Documented Conventions
- Typed accessors `get`/`get_mut` take `EntityId` by value; CRUD methods (`add_component`, `remove_component`, `has_component`, `get_component`) take `&EntityId`. Prefer by-value for new APIs.
- `Children` uses a `Vec<EntityId>` deliberately — child order is load-bearing for the editor hierarchy panel and scene serialization. Do not swap to `HashSet`.

## Common Pitfalls
- `Box<dyn Component>` is NOT clonable — there is no `dyn_clone`/`CloneComponent` machinery. Storage is plain `HashMap<EntityId, Box<dyn Component>>`; anything that needs to copy components (e.g. `WorldSnapshot`, entity duplication) downcasts to each known concrete type and calls its own `Clone`
- When downcasting a `Box<dyn Component>`, call `.as_ref().as_any()` (or `.as_mut().as_any_mut()`) — calling `.as_any()` directly on the Box hits the blanket impl on the Box itself, not the concrete type (see component.rs comments)
- TypeId is per-concrete-type — different generic params = different TypeIds
- Always check for circular references when reparenting in hierarchy
- serde_json for inspector, RON for scene files — both must work

## Testing
- 164 passing, 10 ignored — `cargo test -p ecs`
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
