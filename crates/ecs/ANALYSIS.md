# ECS (Entity Component System) Analysis

> **Audit: 2026-04-15**
> Pruned stale items: the "archetype-based ECS" description (crate now uses HashMap per-type
> storage — no archetype code remains), fixed items (PlayerTag alias, `#[allow(dead_code)]`
> suppressions, mixed module visibility, incomplete test assertions, stale test counts), and
> the outdated `EngineApplication`/scene-stack integration diagram (EngineApplication was
> deleted). Kept architectural sections, tradeoffs, and risks that still inform future work.

## Current State (April 2026)

The ECS crate is the data backbone of the engine. It provides entity management, HashMap-based
per-type component storage, a type-safe query system, scene-graph hierarchy, and lifecycle-aware
system execution. The crate is stable and used by `engine_core`, `physics`, `editor`,
and `editor_integration`.

**Test count:** 156 passing, 10 ignored (doctest markers), 0 failing.

## Architecture

### Storage Model — HashMap<TypeId, ComponentStore>

```
World
├── entities: HashMap<EntityId, Entity>
├── entity_generations: HashMap<EntityId, EntityGeneration>
├── components: ComponentRegistry
│   └── storages: HashMap<TypeId, ComponentStore>
│       └── components: HashMap<EntityId, Box<dyn Component>>
├── systems:   SystemRegistry
├── resources: ResourceStorage            (typed singletons)
├── events:    EventBus                   (typed per-frame messaging)
└── config:    WorldConfig
```

**Tradeoff vs. archetype storage:** We chose HashMap per-type storage for implementation
simplicity and cheap editor workflows (arbitrary add/remove component without bucket shuffling).
The cost is poor cache locality and O(entities × types) queries — see "Known Tradeoffs" below.

### Query System (`query.rs`)

Type-safe queries via the `QueryTypes` trait with three concrete types:
- `Single<T>` — entities with component `T`
- `Pair<T, U>` — entities with both `T` and `U`
- `Triple<T, U, V>` — entities with all three

Implementation walks every entity and checks each required `TypeId` against the registry.
Adequate for hundreds of entities; would need rethinking at 10k+ (see tradeoffs).

### Hierarchy (Scene Graph)

- Components: `Parent`, `Children`, `GlobalTransform2D` in `hierarchy.rs`
- Extension trait: `WorldHierarchyExt` in `hierarchy_extension.rs` — `set_parent()`,
  `remove_parent()`, `get_parent()`, `get_children()`, `get_ancestors()`, `get_descendants()`,
  `is_ancestor_of()`, `is_descendant_of()`, `get_root_entities()`, recursive removal
- Transform propagation: `TransformHierarchySystem` in `hierarchy_system.rs`
- Cycle prevention on `set_parent()`
- Full RON serialization (parent field + inline children) via `engine_core::scene_loader`

### Component Registry & Reflection

Two registries with distinct roles — do not confuse them:

- `component::ComponentRegistry` — runtime *storage*, owned by `World`. HashMap keyed by `TypeId`.
- `component_registry::ComponentRegistry` — global *metadata* registry for editor/scene loading.
  Maps type name strings → TypeId and factory closures for JSON deserialization. Lazily
  initialized via `OnceLock` in `global_registry()`. Built-ins registered at first access:
  `Transform2D`, `Sprite`, `SpriteAnimation`, `Camera`.

The `ComponentMeta` trait provides `type_name()` and `field_names()` for editor inspection.
The `DeriveComponentMeta` proc-macro (re-exported from the `ecs_macros` sibling crate)
auto-implements it. The older `define_component!` declarative macro is still available for
convenience but rarely needed now.

### Resources & Events (added in `d67152e`)

- `ResourceStorage` — typed singleton state on the World (`insert_resource`, `resource`, `resource_mut`, `remove_resource`, `has_resource`)
- `EventBus` — typed per-frame messaging (`emit_event`, `read_events`, `has_events`, `flush_events`)
- `state_machine` module — `StateMachine` and `HierarchicalStateMachine` types

## Built-in Components

| Component | Module | Notes |
|-----------|--------|-------|
| `Transform2D` | re-exported from `common` | position, rotation, scale |
| `GlobalTransform2D` | `hierarchy` | world-space transform, populated by `TransformHierarchySystem` |
| `Sprite` | `sprite_components` | texture_handle, offset, rotation, scale, color, depth, tex_region, visible |
| `Camera` | re-exported from `common` | renamed from `Camera2D` in `947e359` |
| `Name` | `sprite_components` | used by editor hierarchy panel |
| `Parent`, `Children` | `hierarchy` | scene graph |
| `SpriteAnimation` | `sprite_components` | frame-based; animation frame tex_region applied in renderer (fixed in `7c98289`) |
| `AudioSource`, `AudioListener` | `audio_components` | consumed by audio crate |
| `RigidBody`, `Collider` | defined here, consumed by `physics` crate |

## Known Tradeoffs / Future Directions

### 1. HashMap-per-type storage, not archetypes

**Current:** `HashMap<TypeId, HashMap<EntityId, Box<dyn Component>>>`.

**Consequences:**
- Add/remove component is O(1) and doesn't shuffle entities between buckets — great for editor UX.
- Cache locality is poor (every component access is a pointer chase through two hashes and a Box).
- Queries walk all entities and test each required `TypeId` separately.

At current scene sizes (hundreds of entities, handful of components each), this is fine.
If we start pushing thousands of sprites or need a particle system, revisit archetype or SoA
storage. Don't rush it — the current storage has let the editor move fast.

### 2. `renderer` is a direct dependency of `ecs`

`sprite_components.rs` and `sprite_system.rs` import `renderer::{Sprite, Camera, TextureHandle}`.
This means `ecs` cannot be built headless-of-renderer. Options if this becomes a problem
(e.g., server-side simulation, or want to break a cycle):
- Move sprite/camera components into `renderer` or a new `render_components` crate
- Feature-gate rendering components behind a `render` feature on `ecs`
- Introduce a trait-level abstraction so `ecs` defines the data, `renderer` defines the GPU side

Not urgent — everything downstream of ecs also depends on renderer today.

### 3. Query walks all entities

`query_entities` calls `self.entities()` (which allocates a `Vec<EntityId>`) and filters.
Two issues:
- Allocation on every query call
- No index from component type → entities, so we re-scan every call

Likely fine for gameplay code that caches the result. For hot paths (rendering, physics)
consider iterating `self.components.storages[&type_id]` directly for the smallest required
type, then testing the others. Returning an iterator instead of a `Vec` would also help.

### 4. `World::update()` uses `mem::swap` to avoid borrow conflicts

`update()` swaps `self.systems` out, runs them against `&mut self`, swaps back. It works,
and the system registry uses `catch_unwind` so a panicking system won't lose the registry in
practice. But the pattern is fragile. A cleaner design would split World into "data" and
"systems" structs so systems can borrow data mutably without the swap dance.

### 5. Scene snapshot needs explicit type list

`WorldSnapshot` (in `editor_integration`) must know every concrete component type to
clone/restore. New components must be added there. Not a crate-level issue, but worth
remembering when adding components to this crate.

## Cross-Crate Interactions

- **engine_core** — owns the `World`, drives `update()`, loads scenes via `scene_loader`
- **physics** — reads/writes `RigidBody`, `Collider`, `Transform2D`; types defined here, semantics live in physics
- **renderer** — consumed by `sprite_components`/`sprite_system` for GPU-facing types
- **editor** — reads components via `ComponentMeta` for the inspector; uses hierarchy for the tree view
- **editor_integration** — snapshots/restores the World for play/pause; needs every concrete
  component type hand-listed in `WorldSnapshot`
- **ecs_macros** — sibling crate providing `#[derive(ComponentMeta)]`

## File Map (quick reference)

```
src/
├── lib.rs                   public API, visibility strategy, EcsError
├── world.rs                 World struct (entity/component/system CRUD, resources, events)
├── entity.rs                EntityId, Entity
├── entity_builder.rs        Fluent spawn builder: world.spawn().with(...).id()
├── component.rs             Component trait, ComponentStore, ComponentRegistry (storage)
├── component_registry.rs    Global metadata registry, ComponentMeta trait, define_component! macro
├── query.rs                 Single, Pair, Triple, QueryTypes trait
├── generation.rs            EntityGeneration, EntityReference, GenerationError
├── hierarchy.rs             Parent, Children, GlobalTransform2D
├── hierarchy_extension.rs   WorldHierarchyExt trait
├── hierarchy_system.rs      TransformHierarchySystem
├── sprite_components.rs     Sprite, Name, re-exports Transform2D/Camera from common
├── sprite_system.rs         SpriteRenderSystem, SpriteAnimationSystem
├── audio_components.rs      AudioSource, AudioListener, PlaySoundEffect
├── behavior.rs              Behavior trait, EntityTag
├── event.rs                 EventBus (typed per-frame messaging)
├── resource.rs              ResourceStorage (typed singletons)
├── state_machine.rs         StateMachine, HierarchicalStateMachine
├── system.rs                System trait, SystemRegistry, lifecycle
└── prelude.rs               Common re-exports
```
