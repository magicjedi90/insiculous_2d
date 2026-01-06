//! Hello World - Demonstrates full integration of all engine crates
//!
//! This example uses:
//! - engine_core: Scene lifecycle management
//! - ecs: Entity-Component-System with typed component access
//! - input: Keyboard handling via InputHandler
//! - renderer: WGPU sprite rendering
//!
//! Controls: WASD to move player, ESC to exit

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
    keyboard::{PhysicalKey, KeyCode},
};

// Import from engine crates
use engine_core::Scene;
use ecs::{EntityId, sprite_components::{Sprite as EcsSprite, Transform2D}};
use input::InputHandler;
use renderer::prelude::*;
use glam::{Vec2, Vec4};

/// Game state demonstrating full crate integration
struct HelloWorld {
    // Renderer crate
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,

    // Engine core crate (contains ECS World)
    scene: Scene,

    // Input crate
    input: InputHandler,

    // Entity references
    player_entity: Option<EntityId>,
    target_entity: Option<EntityId>,
}

impl HelloWorld {
    fn new() -> Self {
        let scene = Scene::new("main");

        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            scene,
            input: InputHandler::new(),
            player_entity: None,
            target_entity: None,
        }
    }

    /// Create entities with ECS components
    fn setup_entities(&mut self) {
        let world = &mut self.scene.world;

        // Create player entity with Transform2D and Sprite components
        let player = world.create_entity();
        world.add_component(&player, Transform2D::new(Vec2::new(-100.0, 0.0))).ok();
        world.add_component(&player, EcsSprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
        self.player_entity = Some(player);

        // Create target entity (stationary)
        let target = world.create_entity();
        world.add_component(&target, Transform2D::new(Vec2::new(100.0, 0.0))).ok();
        world.add_component(&target, EcsSprite::new(0).with_color(Vec4::new(1.0, 0.2, 0.2, 1.0))).ok();
        self.target_entity = Some(target);

        println!("Created {} entities in ECS world", world.entity_count());
    }

    /// Update player position using input and ECS
    fn update(&mut self) {
        let speed = 5.0;

        // Query input state
        let move_left = self.input.is_key_pressed(KeyCode::KeyA);
        let move_right = self.input.is_key_pressed(KeyCode::KeyD);
        let move_up = self.input.is_key_pressed(KeyCode::KeyW);
        let move_down = self.input.is_key_pressed(KeyCode::KeyS);

        // Update player transform in ECS using typed getter
        if let Some(player) = self.player_entity {
            if let Some(transform) = self.scene.world.get_mut::<Transform2D>(player) {
                if move_left { transform.position.x -= speed; }
                if move_right { transform.position.x += speed; }
                if move_up { transform.position.y += speed; }
                if move_down { transform.position.y -= speed; }
            }
        }

        self.input.update();
    }

    /// Render sprites by extracting data from ECS
    fn render(&mut self) {
        let (Some(renderer), Some(pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) else {
            return;
        };

        let white_texture = TextureHandle { id: 0 };
        let mut batcher = SpriteBatcher::new(10);

        // Extract sprites from ECS world using typed API
        for entity_id in self.scene.world.entities() {
            let transform = self.scene.world.get::<Transform2D>(entity_id);
            let sprite = self.scene.world.get::<EcsSprite>(entity_id);

            if let (Some(transform), Some(ecs_sprite)) = (transform, sprite) {
                // Convert ECS sprite to renderer sprite
                let renderer_sprite = Sprite::new(white_texture)
                    .with_position(transform.position)
                    .with_rotation(transform.rotation)
                    .with_scale(transform.scale * ecs_sprite.scale * 80.0)
                    .with_color(ecs_sprite.color)
                    .with_depth(ecs_sprite.depth);

                batcher.add_sprite(&renderer_sprite);
            }
        }

        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let textures = std::collections::HashMap::new();

        if let Err(e) = renderer.render_with_sprites(pipeline, &self.camera, &textures, &batch_refs) {
            eprintln!("Render error: {}", e);
        }
    }
}

impl ApplicationHandler<()> for HelloWorld {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Hello World - Insiculous 2D Engine");
        println!("==================================");
        println!("Crates integrated:");
        println!("  [OK] engine_core - Scene lifecycle");
        println!("  [OK] ecs - Typed component access (get/get_mut)");
        println!("  [OK] input - Keyboard handling");
        println!("  [OK] renderer - Sprite rendering");
        println!();
        println!("Controls: WASD to move blue sprite, ESC to exit");
        println!();

        let window_attributes = WindowAttributes::default()
            .with_title("Hello World - Insiculous 2D")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window.clone());

        match pollster::block_on(renderer::init(window)) {
            Ok(mut r) => {
                r.set_clear_color(0.1, 0.1, 0.15, 1.0);
                let pipeline = SpritePipeline::new(r.device(), 100);
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                self.renderer = Some(r);
                self.sprite_pipeline = Some(pipeline);

                // Initialize scene lifecycle
                if let Err(e) = self.scene.initialize() {
                    eprintln!("Scene init error: {}", e);
                }
                if let Err(e) = self.scene.start() {
                    eprintln!("Scene start error: {}", e);
                }

                // Setup ECS entities with components
                self.setup_entities();

                println!("Scene '{}' running with {} entities",
                    self.scene.name(),
                    self.scene.world.entity_count());
            }
            Err(e) => {
                eprintln!("Failed to initialize renderer: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.input.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                let _ = self.scene.stop();
                let _ = self.scene.shutdown();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        let _ = self.scene.stop();
                        let _ = self.scene.shutdown();
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.update();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = HelloWorld::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
