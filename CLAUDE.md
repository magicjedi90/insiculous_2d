@AGENTS.md
@training.md

# Agent Teams — Parallel Development System

When dispatching parallel work, use the Task tool to spawn subagents that work on
independent crates simultaneously. Each crate has its own `CLAUDE.md` with domain
expertise and Godot oracle references.

## How to Dispatch Work

Read `coordination/TODO.md` for available tasks. Dispatch subagents by crate:

```
Task(subagent_type="general-purpose", prompt="
  Read crates/ecs/CLAUDE.md for domain context, then work on TASK-XXX:
  [paste task spec from TODO.md]
  ...
  When done, run cargo test -p ecs && cargo test --workspace.
")
```

Launch independent tasks in parallel (single message, multiple Task calls).
Wait for all to complete, then verify with `cargo test --workspace`.

## Coordination Protocol

### Task Lifecycle
1. **Claim**: Before dispatching, check `coordination/current_tasks/` for locks
2. **Lock**: Create `coordination/current_tasks/TASK-XXX.lock` with agent description
3. **Work**: Subagent implements the task, writes tests, verifies
4. **Verify**: `cargo test --workspace` must pass (598+ tests, 0 failures)
5. **Log**: Append to `coordination/PROGRESS.md` with timestamp and summary
6. **Release**: Remove the lock file

### Parallel Safety Rules
- Dispatch agents to **different crates** to avoid merge conflicts
- Never have two agents editing the same file
- Cross-crate tasks (editor_integration touches editor + ecs) should be single-agent
- After all subagents finish, run `cargo test --workspace` from the coordinator

### Crate → Agent Mapping
| Crate | Domain Focus | Test Command |
|-------|-------------|--------------|
| `ecs` | Components, queries, hierarchy, world ops | `cargo test -p ecs` |
| `renderer` | WGPU pipeline, sprites, textures, shaders | `cargo test -p renderer` |
| `physics` | Rapier2d, colliders, presets, spatial | `cargo test -p physics` |
| `editor` | Panels, inspector, picking, gizmos | `cargo test -p editor` |
| `editor_integration` | Wiring editor to engine, play/pause | `cargo test -p editor_integration` |
| `engine_core` | Game API, managers, scene loading | `cargo test -p engine_core` |
| `ui` | Immediate-mode widgets, fonts | `cargo test -p ui` |
| `input` | Keyboard, mouse, gamepad, actions | `cargo test -p input` |
| `audio` | Rodio playback, spatial audio | `cargo test -p audio` |
| `common` | Math, shared types | `cargo test -p common` |

## Quality Review Role

After subagents push changes, review their work:
- Run `cargo clippy --workspace` — no new warnings
- Check that file sizes stay under 600 lines
- Verify test names describe behavior, not implementation
- No `unwrap()` outside tests, no `#[allow(dead_code)]` additions
- Cross-reference against Godot oracle if architectural decisions look questionable

## Godot Oracle — Global Reference

When any agent (or you as coordinator) is stuck on design decisions, consult Godot:
Use `WebFetch` on `https://github.com/godotengine/godot/blob/master/<path>`

**Quick lookup by feature area:**
- Editor architecture: `editor/editor_node.cpp`
- Entity CRUD: `editor/scene_tree_dock.cpp` — `_tool_selected`
- Inspector: `editor/editor_inspector.cpp` — `_property_changed`
- Viewport picking: `editor/plugins/canvas_item_editor_plugin.cpp` — `_gui_input_viewport`
- Undo/redo: `core/object/undo_redo.cpp`, `editor/editor_undo_redo_manager.cpp`
- Scene save/load: `scene/resources/packed_scene.cpp`
- 2D rendering: `servers/rendering/renderer_canvas_cull.cpp`
- 2D physics: `servers/physics_2d/godot_step_2d.cpp`
- Node hierarchy: `scene/main/node.cpp`

**Rule:** Study Godot's *design patterns*. Adapt to our Rust ECS architecture. Don't copy C++.

## When Stuck — Escalation
1. Agent consults its crate's `CLAUDE.md` Godot oracle table
2. Agent uses `WebFetch` to read relevant Godot source
3. If still stuck, agent writes findings to `coordination/BLOCKERS.md`
4. Coordinator reviews blockers, may reassign or break down the task

## Coordination Files
- `coordination/TODO.md` — Task queue (highest priority at top)
- `coordination/PROGRESS.md` — Completed work log
- `coordination/BLOCKERS.md` — Issues with what-was-tried documentation
- `coordination/current_tasks/` — Lock files for active work
