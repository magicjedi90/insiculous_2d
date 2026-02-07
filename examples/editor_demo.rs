//! Editor demo - wraps the Hello World platformer in the full editor UI.
//!
//! Run with: cargo run --example editor_demo --features editor
//!
//! This loads the same scene and game logic as hello_world.rs, but wrapped
//! inside the editor. Use Play/Pause/Stop (Ctrl+P / Ctrl+Shift+P) to run
//! the game simulation, inspect entities in the hierarchy & inspector,
//! and watch the world restore on Stop.
//!
//! Controls (while Playing):
//!   WASD to move player, SPACE to jump, R to reset
//!
//! Editor shortcuts:
//!   Ctrl+P       Play / Pause toggle
//!   Ctrl+Shift+P Stop (restore scene)
//!   F5           Play / Resume
//!   Q/W/E/R      Select / Move / Rotate / Scale tool
//!   G            Toggle grid

use engine_core::prelude::*;
use editor_integration::run_game_with_editor;
use ecs::hierarchy_system::TransformHierarchySystem;
use ecs::WorldHierarchyExt;
use std::path::Path;

/// Platformer game — same logic as hello_world.rs.
struct PlatformerGame {
    physics: Option<PhysicsSystem>,
    behaviors: BehaviorRunner,
    scene_instance: Option<SceneInstance>,
    transform_hierarchy: TransformHierarchySystem,
    jump_sound: Option<SoundHandle>,
    music_playing: bool,
    volume: f32,
    show_ui: bool,
}

impl PlatformerGame {
    fn new() -> Self {
        Self {
            physics: None,
            behaviors: BehaviorRunner::new(),
            scene_instance: None,
            transform_hierarchy: TransformHierarchySystem::new(),
            jump_sound: None,
            music_playing: false,
            volume: 1.0,
            show_ui: true,
        }
    }

    fn reset_player(&mut self, ctx: &mut GameContext) {
        let player = self.scene_instance.as_ref()
            .and_then(|scene| scene.get_entity("player"));

        if let Some(player) = player {
            if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
                transform.position = glam::Vec2::new(-200.0, 100.0);
            }
            if let Some(body) = ctx.world.get_mut::<RigidBody>(player) {
                body.velocity = glam::Vec2::ZERO;
            }
            if let Some(physics) = &mut self.physics {
                physics.physics_world_mut().set_body_transform(
                    player, glam::Vec2::new(-200.0, 100.0), 0.0,
                );
                physics.physics_world_mut().set_body_velocity(player, glam::Vec2::ZERO, 0.0);
            }
        }
    }
}

impl Game for PlatformerGame {
    fn init(&mut self, ctx: &mut GameContext) {
        ctx.assets.set_base_path("examples");

        let scene_path = Path::new("examples/assets/scenes/hello_world.scene.ron");

        match SceneLoader::load_and_instantiate(scene_path, &mut ctx.world, ctx.assets) {
            Ok(instance) => {
                log::info!("Loaded scene '{}' with {} entities", instance.name, instance.entity_count);
                self.behaviors.set_named_entities(instance.named_entities.clone());

                let physics_config = if let Some(settings) = &instance.physics {
                    PhysicsConfig::new(glam::Vec2::new(settings.gravity.0, settings.gravity.1))
                        .with_scale(settings.pixels_per_meter)
                } else {
                    PhysicsConfig::platformer()
                };

                self.physics = Some(PhysicsSystem::with_config(physics_config));
                self.scene_instance = Some(instance);
            }
            Err(e) => {
                log::warn!("Failed to load scene: {}", e);
                log::info!("Creating entities programmatically as fallback...");

                use ecs::behavior::Behavior;
                let player = ctx.world.create_entity();
                ctx.world.add_component(&player, Transform2D::new(glam::Vec2::new(-200.0, 100.0))).ok();
                ctx.world.add_component(&player, Sprite::new(0).with_color(glam::Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
                ctx.world.add_component(&player, RigidBody::player_platformer()).ok();
                ctx.world.add_component(&player, Collider::player_box()).ok();
                ctx.world.add_component(&player, Behavior::PlayerPlatformer {
                    move_speed: 120.0,
                    jump_impulse: 420.0,
                    jump_cooldown: 0.3,
                    tag: "player".to_string(),
                }).ok();

                let ground = ctx.world.create_entity();
                ctx.world.add_component(&ground,
                    Transform2D::new(glam::Vec2::new(0.0, -250.0))
                        .with_scale(glam::Vec2::new(10.0, 0.5))
                ).ok();
                ctx.world.add_component(&ground,
                    Sprite::new(0).with_color(glam::Vec4::new(0.3, 0.3, 0.3, 1.0))
                ).ok();
                ctx.world.add_component(&ground, RigidBody::new_static()).ok();
                ctx.world.add_component(&ground, Collider::platform(800.0, 40.0)).ok();

                self.physics = Some(PhysicsSystem::with_config(PhysicsConfig::platformer()));
            }
        }

        // Initialise physics
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.initialize(&mut ctx.world).ok();
        }

        // Initialise transform hierarchy
        {
            use ecs::System;
            self.transform_hierarchy.initialize(&mut ctx.world).ok();
        }

        // Add Name components for editor hierarchy display
        self.add_editor_names(ctx);

        // Try to load sound effects
        match ctx.audio.load_sound("examples/assets/sounds/snd_jump.wav") {
            Ok(handle) => {
                self.jump_sound = Some(handle);
                log::info!("Loaded jump sound effect");
            }
            Err(e) => {
                log::info!("No jump sound loaded ({})", e);
            }
        }

        // Try to load background music
        match ctx.audio.play_music("examples/assets/sounds/music.ogg") {
            Ok(()) => { self.music_playing = true; }
            Err(_) => {}
        }

        let total = ctx.world.entity_count();
        let roots = ctx.world.get_root_entities().len();
        log::info!("Game initialised: {} entities ({} roots, {} children)",
                   total, roots, total - roots);
    }

    fn update(&mut self, ctx: &mut GameContext) {
        // Jump sound
        if ctx.input.is_key_just_pressed(KeyCode::Space) {
            if let Some(jump_sound) = &self.jump_sound {
                let settings = SoundSettings::new().with_volume(0.8).with_speed(1.0);
                ctx.audio.play_with_settings(jump_sound, settings).ok();
            }
        }

        // Music toggle
        if ctx.input.is_key_just_pressed(KeyCode::KeyM) {
            if self.music_playing {
                ctx.audio.pause_music();
                self.music_playing = false;
            } else {
                ctx.audio.resume_music();
                self.music_playing = true;
            }
        }

        // Behaviours (player movement)
        self.behaviors.update(
            &mut ctx.world,
            ctx.input,
            ctx.delta_time,
            self.physics.as_mut(),
        );

        // Reset
        if ctx.input.is_key_pressed(KeyCode::KeyR) {
            self.reset_player(ctx);
        }

        // Physics
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.update(&mut ctx.world, ctx.delta_time);
        }

        // Hierarchy propagation
        {
            use ecs::System;
            self.transform_hierarchy.update(&mut ctx.world, ctx.delta_time);
        }

        // In-game UI (only when the editor delegates update to us)
        if ctx.input.is_key_just_pressed(KeyCode::KeyH) {
            self.show_ui = !self.show_ui;
        }

        if self.show_ui {
            let panel_rect = UIRect::new(10.0, 10.0, 220.0, 140.0);
            ctx.ui.panel(panel_rect);
            ctx.ui.label("Game Controls", glam::Vec2::new(20.0, 25.0));

            ctx.ui.label("Volume:", glam::Vec2::new(20.0, 55.0));
            let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
            let new_volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
            if new_volume != self.volume {
                self.volume = new_volume;
                ctx.audio.set_master_volume(self.volume);
            }

            let reset_btn_rect = UIRect::new(20.0, 100.0, 90.0, 30.0);
            if ctx.ui.button("reset_btn", "Reset", reset_btn_rect) {
                self.reset_player(ctx);
            }
        }
    }
}

impl PlatformerGame {
    /// Add Name + GlobalTransform2D to entities that lack them, so the editor
    /// hierarchy and inspector work nicely.
    fn add_editor_names(&self, ctx: &mut GameContext) {
        use ecs::{GlobalTransform2D, Name};

        if let Some(instance) = &self.scene_instance {
            for (name, &entity_id) in &instance.named_entities {
                if ctx.world.get::<Name>(entity_id).is_none() {
                    ctx.world.add_component(&entity_id, Name::new(name)).ok();
                }
                if ctx.world.get::<GlobalTransform2D>(entity_id).is_none() {
                    ctx.world.add_component(&entity_id, GlobalTransform2D::default()).ok();
                }
            }
        }

        // Ensure all entities have a GlobalTransform2D for the editor gizmo
        for entity_id in ctx.world.entities() {
            if ctx.world.get::<Transform2D>(entity_id).is_some()
                && ctx.world.get::<GlobalTransform2D>(entity_id).is_none()
            {
                ctx.world.add_component(&entity_id, GlobalTransform2D::default()).ok();
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = GameConfig::new("Insiculous 2D - Editor Demo")
        .with_size(1280, 720)
        .with_clear_color(0.1, 0.1, 0.15, 1.0);

    if let Err(e) = run_game_with_editor(PlatformerGame::new(), config) {
        log::error!("Editor error: {}", e);
    }
}
