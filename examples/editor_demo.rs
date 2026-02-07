//! Editor demo - demonstrates the editor UI framework.
//!
//! Run with: cargo run --example editor_demo --features editor

use engine_core::prelude::*;
use editor_integration::run_game_with_editor;
use glam::Vec2;

struct EditorDemoGame;

impl Game for EditorDemoGame {
    fn init(&mut self, ctx: &mut GameContext) {
        use ecs::{GlobalTransform2D, Name, WorldHierarchyExt};

        // Root entity: "Player" with child hierarchy
        let player = ctx.world.create_entity();
        ctx.world.add_component(&player, Name::new("Player")).ok();
        ctx.world.add_component(&player, ecs::sprite_components::Transform2D::new(Vec2::new(-100.0, 0.0))).ok();
        ctx.world.add_component(&player, GlobalTransform2D::default()).ok();

        let weapon = ctx.world.create_entity();
        ctx.world.add_component(&weapon, Name::new("Weapon")).ok();
        ctx.world.add_component(&weapon, ecs::sprite_components::Transform2D::new(Vec2::new(20.0, 0.0))).ok();
        ctx.world.add_component(&weapon, GlobalTransform2D::default()).ok();
        ctx.world.set_parent(weapon, player).ok();

        let muzzle = ctx.world.create_entity();
        ctx.world.add_component(&muzzle, Name::new("Muzzle Flash")).ok();
        ctx.world.add_component(&muzzle, ecs::sprite_components::Transform2D::new(Vec2::new(10.0, 0.0))).ok();
        ctx.world.add_component(&muzzle, GlobalTransform2D::default()).ok();
        ctx.world.set_parent(muzzle, weapon).ok();

        // Sprite entity (no Name — tests fallback display)
        let sprite_entity = ctx.world.create_entity();
        ctx.world.add_component(&sprite_entity, ecs::sprite_components::Transform2D::new(Vec2::new(100.0, 50.0))).ok();
        ctx.world.add_component(&sprite_entity, ecs::Sprite::new(0)).ok();
        ctx.world.add_component(&sprite_entity, GlobalTransform2D::default()).ok();

        let child_entity = ctx.world.create_entity();
        ctx.world.add_component(&child_entity, ecs::sprite_components::Transform2D::new(Vec2::new(0.0, -30.0))).ok();
        ctx.world.add_component(&child_entity, GlobalTransform2D::default()).ok();
        ctx.world.set_parent(child_entity, sprite_entity).ok();

        // Standalone root entity
        let standalone = ctx.world.create_entity();
        ctx.world.add_component(&standalone, ecs::sprite_components::Transform2D::new(Vec2::new(0.0, -100.0))).ok();
        ctx.world.add_component(&standalone, GlobalTransform2D::default()).ok();

        log::info!("Editor demo initialized with 6 entities in hierarchy");
    }

    fn update(&mut self, _ctx: &mut GameContext) {
        // Editor handles all chrome — game logic goes here
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = GameConfig::new("Insiculous 2D - Editor Demo")
        .with_size(1280, 720);

    if let Err(e) = run_game_with_editor(EditorDemoGame, config) {
        log::error!("Editor error: {}", e);
    }
}
