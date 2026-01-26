//! Hello World - Demonstrates the simplified Game API with Physics, Audio, UI, and Scene Graph
//!
//! This example shows how easy it is to create a game with the Insiculous 2D engine.
//! All the window, event loop, and rendering boilerplate is handled internally.
//!
//! Features demonstrated:
//! - Simple Game API (Game trait, GameConfig, run_game)
//! - RON scene file loading for entity/component definition
//! - ECS for entity/component management
//! - Asset Manager for loading/creating textures
//! - **Audio System** - sound effects and music playback
//! - **UI System** - immediate-mode UI with buttons, sliders, and panels
//! - **Font Rendering** - load TTF/OTF fonts for text display via fontdue
//! - Input handling with keyboard
//! - 2D Physics with rapier2d integration
//! - **Scene Graph Hierarchy** - parent-child entity relationships with transform propagation
//!
//! Controls: WASD to move player, SPACE to jump, R to reset, M to toggle music, ESC to exit
//!           Click UI buttons for interactive controls!
//!
//! Scene file: examples/assets/scenes/hello_world.scene.ron
//! Font file: examples/assets/fonts/font.ttf (optional - download any TTF font)

use engine_core::prelude::*;
use ecs::hierarchy_system::TransformHierarchySystem;
use ecs::WorldHierarchyExt;
use std::path::Path;

/// Our game state - simplified with BehaviorRunner handling player movement
struct HelloWorld {
    physics: Option<PhysicsSystem>,
    /// Behavior runner - processes all entity behaviors (player movement, AI, etc.)
    behaviors: BehaviorRunner,
    /// Scene instance with named entity lookups
    scene_instance: Option<SceneInstance>,
    /// Transform hierarchy system for parent-child relationships
    transform_hierarchy: TransformHierarchySystem,
    /// The jump sound effect handle (if loaded)
    jump_sound: Option<SoundHandle>,
    /// Whether music is currently playing
    music_playing: bool,
    /// Volume slider value (0.0 to 1.0)
    volume: f32,
    /// Whether to show the UI panel
    show_ui: bool,
    /// Whether a font was successfully loaded
    font_loaded: bool,
}

impl HelloWorld {
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
            font_loaded: false,
        }
    }

    fn reset_player(&mut self, ctx: &mut GameContext) {
        // Get player entity from a scene instance
        let player = self.scene_instance.as_ref()
            .and_then(|scene| scene.get_entity("player"));

        if let Some(player) = player {
            // Reset player position
            if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
                transform.position = Vec2::new(-200.0, 100.0);
            }
            // Reset velocity
            if let Some(body) = ctx.world.get_mut::<RigidBody>(player) {
                body.velocity = Vec2::ZERO;
            }
            // Update physics world
            if let Some(physics) = &mut self.physics {
                physics.physics_world_mut().set_body_transform(player, Vec2::new(-200.0, 100.0), 0.0);
                physics.physics_world_mut().set_body_velocity(player, Vec2::ZERO, 0.0);
            }
        }
    }
}

impl Game for HelloWorld {
    /// Called once at startup - loads a scene from a .scene.ron file
    fn init(&mut self, ctx: &mut GameContext) {
        // Set asset base path to examples directory
        ctx.assets.set_base_path("examples");

        // Try to load the scene from RON file
        let scene_path = Path::new("examples/assets/scenes/hello_world.scene.ron");

        match SceneLoader::load_and_instantiate(scene_path, &mut ctx.world, ctx.assets) {
            Ok(instance) => {
                println!("Loaded scene '{}' with {} entities", instance.name, instance.entity_count);

                // Set named entities on the behavior runner (for FollowEntity and other reference-based behaviors)
                self.behaviors.set_named_entities(instance.named_entities.clone());

                // Create a physics system from scene settings
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
                println!("Failed to load scene: {}", e);
                println!("Creating entities programmatically as fallback...");

                // Fallback: create entities manually with a Behavior component
                use ecs::behavior::Behavior;
                let player = ctx.world.create_entity();
                ctx.world.add_component(&player, Transform2D::new(Vec2::new(-200.0, 100.0))).ok();
                ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
                ctx.world.add_component(&player, RigidBody::player_platformer()).ok();
                ctx.world.add_component(&player, Collider::player_box()).ok();
                // Add behavior for player-controlled platformer movement
                ctx.world.add_component(&player, Behavior::PlayerPlatformer {
                    move_speed: 120.0,
                    jump_impulse: 420.0,
                    jump_cooldown: 0.3,
                    tag: "player".to_string(),
                }).ok();

                // Create ground
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

        // Initialize the physics system
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.initialize(&mut ctx.world).ok();
        }

        // Initialize the transform hierarchy system
        use ecs::System;
        self.transform_hierarchy.initialize(&mut ctx.world).ok();

        // Count entities with hierarchy relationships
        let root_count = ctx.world.get_root_entities().len();
        let total_count = ctx.world.entity_count();
        let child_count = total_count - root_count;

        // Try to load sound effects
        match ctx.audio.load_sound("examples/assets/sounds/snd_jump.wav") {
            Ok(handle) => {
                self.jump_sound = Some(handle);
                println!("Loaded jump sound effect");
            }
            Err(e) => {
                println!("No jump sound loaded ({}). Audio demo will show API usage.", e);
                println!("To enable audio, add a WAV file at examples/assets/sounds/snd_jump.wav");
            }
        }

        // Try to load background music
        match ctx.audio.play_music("examples/assets/sounds/music.ogg") {
            Ok(()) => {
                self.music_playing = true;
                println!("Playing background music");
            }
            Err(_) => {
                println!("No background music found. Add music.ogg to examples/assets/sounds/");
            }
        }

        // Try to load a font for text rendering
        match ctx.ui.load_font_file("examples/assets/fonts/font.ttf") {
            Ok(_handle) => {
                self.font_loaded = true;
                println!("Font loaded - text will render with actual glyphs!");
            }
            Err(_) => {
                // Try fallback paths for common system fonts
                let font_paths = [
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                    "/usr/share/fonts/TTF/DejaVuSans.ttf",
                    "/System/Library/Fonts/Helvetica.ttc",
                    "C:\\Windows\\Fonts\\arial.ttf",
                ];

                for path in font_paths {
                    if let Ok(_) = ctx.ui.load_font_file(path) {
                        self.font_loaded = true;
                        println!("System font loaded from: {}", path);
                        break;
                    }
                }

                if !self.font_loaded {
                    println!("No font loaded. Text will render as placeholders.");
                    println!("To enable font rendering, add a .ttf file to examples/assets/fonts/font.ttf");
                }
            }
        }

        println!("Game initialized with {} entities ({} root, {} children)",
                 total_count, root_count, child_count);
        println!("Controls: WASD to move, SPACE to jump, R to reset, M to toggle music, H to toggle UI, ESC to exit");
        println!("Physics enabled - push the wood boxes around!");
        if child_count > 0 {
            println!("Scene Graph: {} child entities will follow their parents!", child_count);
        }
        println!("Audio system ready - master volume: {:.0}%", ctx.audio.master_volume() * 100.0);
        println!("UI system ready - click buttons and drag sliders!");
        if self.font_loaded {
            println!("Font system ready - text renders with actual glyphs!");
        }
    }

    /// Called every frame - update game logic
    fn update(&mut self, ctx: &mut GameContext) {
        // Play jump sound when SPACE is pressed (if sound is loaded)
        if ctx.input.is_key_just_pressed(KeyCode::Space) {
            if let Some(jump_sound) = &self.jump_sound {
                // Play with slight random pitch variation for variety
                let settings = SoundSettings::new()
                    .with_volume(0.8)
                    .with_speed(1.0);
                if let Err(e) = ctx.audio.play_with_settings(jump_sound, settings) {
                    eprintln!("Failed to play jump sound: {}", e);
                }
            }
        }

        // Toggle music with M key
        if ctx.input.is_key_just_pressed(KeyCode::KeyM) {
            if self.music_playing {
                ctx.audio.pause_music();
                self.music_playing = false;
                println!("Music paused");
            } else {
                ctx.audio.resume_music();
                self.music_playing = true;
                println!("Music resumed");
            }
        }

        // Adjust master volume with +/- keys
        if ctx.input.is_key_just_pressed(KeyCode::Equal) {
            let new_volume = (ctx.audio.master_volume() + 0.1).min(1.0);
            ctx.audio.set_master_volume(new_volume);
            println!("Volume: {:.0}%", new_volume * 100.0);
        }
        if ctx.input.is_key_just_pressed(KeyCode::Minus) {
            let new_volume = (ctx.audio.master_volume() - 0.1).max(0.0);
            ctx.audio.set_master_volume(new_volume);
            println!("Volume: {:.0}%", new_volume * 100.0);
        }

        // Process all entity behaviors (player movement, AI, etc.)
        // This single call replaces 40+ lines of hardcoded movement logic!
        self.behaviors.update(
            &mut ctx.world,
            ctx.input,
            ctx.delta_time,
            self.physics.as_mut(),
        );

        // Reset player position (manual action, not a behavior)
        if ctx.input.is_key_pressed(KeyCode::KeyR) {
            self.reset_player(ctx);
        }

        // Step physics simulation
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.update(&mut ctx.world, ctx.delta_time);
        }

        // Update transform hierarchy - propagates transforms from parents to children
        // This must run after physics so child entities follow their parents
        {
            use ecs::System;
            self.transform_hierarchy.update(&mut ctx.world, ctx.delta_time);
        }

        // ==================== UI Demo ====================
        // Toggle UI visibility with H key
        if ctx.input.is_key_just_pressed(KeyCode::KeyH) {
            self.show_ui = !self.show_ui;
        }

        // Create UI elements (immediate-mode - describe UI every frame)
        // Labels render with actual fonts if loaded, otherwise as placeholder rectangles
        if self.show_ui {
            // Draw a semi-transparent control panel in the top-left
            let panel_rect = UIRect::new(10.0, 10.0, 220.0, 200.0);
            ctx.ui.panel(panel_rect);

            // Title label (renders with font glyphs if font loaded)
            ctx.ui.label("Controls", Vec2::new(20.0, 25.0));

            // Volume slider
            ctx.ui.label("Volume:", Vec2::new(20.0, 55.0));
            let slider_rect = UIRect::new(20.0, 70.0, 190.0, 20.0);
            let new_volume = ctx.ui.slider("volume_slider", self.volume, slider_rect);
            if new_volume != self.volume {
                self.volume = new_volume;
                ctx.audio.set_master_volume(self.volume);
            }

            // Music toggle button
            let music_btn_rect = UIRect::new(20.0, 100.0, 90.0, 30.0);
            let music_label = if self.music_playing { "Pause" } else { "Play" };
            if ctx.ui.button("music_btn", music_label, music_btn_rect) {
                if self.music_playing {
                    ctx.audio.pause_music();
                    self.music_playing = false;
                } else {
                    ctx.audio.resume_music();
                    self.music_playing = true;
                }
            }

            // Reset button
            let reset_btn_rect = UIRect::new(120.0, 100.0, 90.0, 30.0);
            if ctx.ui.button("reset_btn", "Reset", reset_btn_rect) {
                self.reset_player(ctx);
            }

            // Progress bar showing current volume level
            ctx.ui.label("Volume Bar:", Vec2::new(20.0, 145.0));
            let progress_rect = UIRect::new(20.0, 160.0, 190.0, 15.0);
            ctx.ui.progress_bar(self.volume, progress_rect);

            // Help text and status at bottom
            ctx.ui.label("H: Toggle UI", Vec2::new(20.0, 185.0));

            // Show font status
            let font_status = if self.font_loaded { "Font: ON" } else { "Font: OFF" };
            ctx.ui.label(font_status, Vec2::new(140.0, 185.0));
        }
    }

    // render() uses the default implementation which extracts sprites from ECS
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create game configuration
    let game_config = GameConfig::new("Hello World - Insiculous 2D Physics Demo")
        .with_size(800, 600)
        .with_clear_color(0.1, 0.1, 0.15, 1.0);

    // Create and run the game
    let game = HelloWorld::new();
    run_game(game, game_config)
}
