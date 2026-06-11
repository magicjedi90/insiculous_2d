# ECS Macros Crate — Agent Context

Procedural macros for the ECS crate.

## Provides
- `#[derive(DeriveComponentMeta)]` — auto-implements `ComponentMeta` trait (type_name, field_names)
- `define_component!` macro — component struct definition with Default impl

## Testing
- 4 tests (3 integration + 1 doc), run with `cargo test -p ecs_macros`
