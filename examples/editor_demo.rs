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
use input::{InputMapping, InputSource};
use std::path::Path;

/// Anchor all asset paths to the repository so the example runs from any
/// working directory.
const EXAMPLES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");

/// Where the player respawns on reset (matches the scene file's spawn point).
const PLAYER_SPAWN: Vec2 = Vec2::new(-200.0, 100.0);

// --- Actions: game-defined enum evaluated through the engine's InputMapping ---

/// Debug actions for the wrapped platformer (movement/jump come from the
/// scene's `PlayerPlatformer` behavior; the jump SOUND reads `ctx.players`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoAction {
    ToggleMusic,
    ToggleUi,
    ResetPlayer,
}

fn demo_actions() -> InputMapping<DemoAction> {
    let mut actions = InputMapping::new();
    actions.bind(DemoAction::ToggleMusic, InputSource::Keyboard(KeyCode::KeyM));
    actions.bind(DemoAction::ToggleUi, InputSource::Keyboard(KeyCode::KeyH));
    actions.bind(DemoAction::ResetPlayer, InputSource::Keyboard(KeyCode::KeyR));
    actions
}

// --- Resources ---
#[derive(Debug, Clone, Default)]
struct GameState {
    score: u32,
    coins_collected: u32,
}

// --- State Machine ---
#[derive(Debug, Clone, PartialEq)]
enum PlayerState { Idle, Running, Jumping, Falling }

#[derive(Debug, Clone, PartialEq)]
enum PlayerGroup { OnGround, InAir }

fn player_group(state: &PlayerState) -> PlayerGroup {
    match state {
        PlayerState::Idle | PlayerState::Running => PlayerGroup::OnGround,
        PlayerState::Jumping | PlayerState::Falling => PlayerGroup::InAir,
    }
}

/// Platformer game — same logic as hello_world.rs.
struct PlatformerGame {
    physics: Option<PhysicsSystem>,
    behaviors: BehaviorRunner,
    scene_instance: Option<SceneInstance>,
    transform_hierarchy: TransformHierarchySystem,
    actions: InputMapping<DemoAction>,
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
            actions: demo_actions(),
            jump_sound: None,
            music_playing: false,
            volume: 1.0,
            show_ui: true,
        }
    }

    fn player_entity(&self) -> Option<EntityId> {
        self.scene_instance
            .as_ref()
            .and_then(|scene| scene.get_entity("player"))
    }

    /// Move the player back to spawn and zero its velocity. Live physics
    /// bodies are owned by rapier, so the reset goes through the physics API
    /// instead of writing `Transform2D` directly (which would be overwritten).
    fn reset_player(&mut self, ctx: &mut GameContext) {
        let Some(player) = self.player_entity() else { return };

        if let Some(physics) = &mut self.physics {
            physics.reset_body(player, PLAYER_SPAWN);
        } else if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
            transform.position = PLAYER_SPAWN;
        }
    }

    fn toggle_music(&mut self, ctx: &mut GameContext) {
        if self.music_playing {
            ctx.audio.pause_music();
            self.music_playing = false;
        } else {
            ctx.audio.resume_music();
            self.music_playing = true;
        }
    }
}

impl Game for PlatformerGame {
    fn init(&mut self, ctx: &mut GameContext) {
        let scene_path = Path::new(EXAMPLES_DIR).join("assets/scenes/hello_world.scene.ron");

        match SceneLoader::load_and_instantiate(&scene_path, ctx.world, ctx.assets) {
            Ok(instance) => {
                log::info!("Loaded scene '{}' with {} entities", instance.name, instance.entity_count);
                self.behaviors.set_named_entities(instance.named_entities.clone());

                let physics_config = if let Some(settings) = &instance.physics {
                    PhysicsConfig::new(Vec2::new(settings.gravity.0, settings.gravity.1))
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

                let player = ctx.world.create_entity();
                ctx.world.add_component(&player, Transform2D::new(PLAYER_SPAWN)).ok();
                ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
                ctx.world.add_component(&player, RigidBody::player_platformer()).ok();
                ctx.world.add_component(&player, Collider::player_box(80.0, 80.0)).ok();
                ctx.world.add_component(&player, Behavior::PlayerPlatformer {
                    move_speed: 120.0,
                    jump_impulse: 420.0,
                    jump_cooldown: 0.3,
                    tag: "player".to_string(),
                }).ok();

                let ground = ctx.world.create_entity();
                ctx.world.add_component(&ground,
                    Transform2D::new(Vec2::new(0.0, -250.0))
                        .with_scale(Vec2::new(10.0, 0.5))
                ).ok();
                ctx.world.add_component(&ground,
                    Sprite::new(0).with_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
                ).ok();
                ctx.world.add_component(&ground, RigidBody::new_static()).ok();
                ctx.world.add_component(&ground, Collider::platform(800.0, 40.0)).ok();

                self.physics = Some(PhysicsSystem::with_config(PhysicsConfig::platformer()));
            }
        }

        // Initialise physics
        if let Some(physics) = &mut self.physics {
            physics.initialize(ctx.world).ok();
        }

        // Initialise transform hierarchy
        self.transform_hierarchy.initialize(ctx.world).ok();

        // Add Name components for editor hierarchy display
        self.add_editor_names(ctx);

        // --- Resources & State Machine ---
        ctx.world.insert_resource(GameState::default());
        if let Some(player) = self.player_entity() {
            ctx.world.add_component(
                &player,
                HierarchicalStateMachine::new(PlayerState::Idle, player_group),
            ).ok();
        }

        // Try to load sound effects
        match ctx.audio.load_sound(Path::new(EXAMPLES_DIR).join("assets/sounds/snd_jump.wav")) {
            Ok(handle) => {
                self.jump_sound = Some(handle);
                log::info!("Loaded jump sound effect");
            }
            Err(e) => {
                log::info!("No jump sound loaded ({})", e);
            }
        }

        // Try to load background music
        if ctx.audio.play_music(Path::new(EXAMPLES_DIR).join("assets/sounds/music.ogg")).is_ok() {
            self.music_playing = true;
        }

        let total = ctx.world.entity_count();
        let roots = ctx.world.get_root_entities().len();
        log::info!("Game initialised: {} entities ({} roots, {} children)",
                   total, roots, total - roots);
    }

    fn on_play_stopped(&mut self, _ctx: &mut GameContext) {
        // Clear rapier physics world so it re-syncs from restored ECS state
        if let Some(physics) = &mut self.physics {
            physics.clear();
        }
    }

    fn update(&mut self, ctx: &mut GameContext) {
        // Jump sound (the jump itself is driven by the behavior system)
        // Player-aware input: Space/Enter/either-pad-A all trigger the sound
        if ctx.players.just_activated(PlayerId::P1, GameAction::Action1, ctx.input)
            || ctx.players.just_activated(PlayerId::P2, GameAction::Action1, ctx.input)
        {
            if let Some(jump_sound) = &self.jump_sound {
                let settings = SoundSettings::new().with_volume(0.8).with_speed(1.0);
                ctx.audio.play_with_settings(jump_sound, settings).ok();
            }
        }

        // Music toggle
        if self.actions.just_activated(DemoAction::ToggleMusic, ctx.input) {
            self.toggle_music(ctx);
        }

        // Behaviours (player movement)
        self.behaviors.update(
            ctx.world,
            ctx.input,
            ctx.delta_time,
            self.physics.as_mut(),
        );

        // Reset
        if self.actions.just_activated(DemoAction::ResetPlayer, ctx.input) {
            self.reset_player(ctx);
        }

        // Physics
        if let Some(physics) = &mut self.physics {
            physics.update(ctx.world, ctx.delta_time);
        }

        // Hierarchy propagation
        self.transform_hierarchy.update(ctx.world, ctx.delta_time);

        // --- Events: process collection events ---
        let collected: Vec<EntityCollected> = ctx.world.read_events::<EntityCollected>().to_vec();
        for event in &collected {
            if let Some(state) = ctx.world.resource_mut::<GameState>() {
                state.score += event.score_value;
                state.coins_collected += 1;
            }
        }

        // --- State Machine: update player state from velocity ---
        if let Some(player) = self.player_entity() {
            let vel = ctx.world.get::<RigidBody>(player)
                .map(|rb| rb.velocity)
                .unwrap_or(Vec2::ZERO);
            let new_state = if vel.y > 10.0 { PlayerState::Jumping }
                else if vel.y < -10.0 { PlayerState::Falling }
                else if vel.x.abs() > 5.0 { PlayerState::Running }
                else { PlayerState::Idle };
            if let Some(sm) = ctx.world.get_mut::<HierarchicalStateMachine<PlayerState, PlayerGroup>>(player) {
                sm.transition_to(new_state);
                sm.tick(ctx.delta_time);
            }
        }

        // In-game UI (only when the editor delegates update to us)
        if self.actions.just_activated(DemoAction::ToggleUi, ctx.input) {
            self.show_ui = !self.show_ui;
        }

        if self.show_ui {
            let panel_rect = UIRect::new(10.0, 10.0, 220.0, 140.0);
            ctx.ui.panel(panel_rect);
            ctx.ui.label("Game Controls", Vec2::new(20.0, 25.0));

            ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
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
        .with_clear_color(0.1, 0.1, 0.15, 1.0)
        .with_asset_base_path(EXAMPLES_DIR);

    if let Err(e) = run_game_with_editor(PlatformerGame::new(), config) {
        log::error!("Editor error: {}", e);
    }
}
