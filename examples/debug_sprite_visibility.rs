//! 🎯 **DEBUG SPRITE VISIBILITY TEST** - Pinpoint exact rendering issues
//! 
//! This demo renders sprites at KNOWN positions and shows their clip-space coordinates
//! to debug why sprites aren't visible on screen.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
    keyboard::{PhysicalKey, KeyCode},
};
use renderer::prelude::*;
use input::prelude::*;
use glam::{Vec2, Vec4, Vec4Swizzles};

/// 🎯 Debug struct to track sprite visibility
#[derive(Debug)]
struct SpriteVisibilityDebug {
    world_pos: Vec2,
    clip_pos: Vec4,
    ndc_pos: Vec2,
    scale: Vec2,
    color: Vec4,
    visible: bool,
}

/// 🎯 **Debug Sprite Demo** - Shows exact coordinates for debugging visibility
struct DebugSpriteVisibilityDemo {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    frame_count: u32,
    visibility_data: Vec<SpriteVisibilityDebug>,
}

impl DebugSpriteVisibilityDemo {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(std::sync::Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            frame_count: 0,
            visibility_data: Vec::new(),
        }
    }
    
    /// 🎯 Calculate clip-space coordinates for debugging
    fn calculate_clip_space(&self, world_pos: Vec2, scale: Vec2) -> SpriteVisibilityDebug {
        let view_proj = self.camera.view_projection_matrix();
        
        // Calculate corners of sprite in world space (-0.5 to 0.5 * scale)
        let corners = [
            Vec2::new(-0.5, 0.5),  // Top-left
            Vec2::new(0.5, 0.5),   // Top-right
            Vec2::new(0.5, -0.5),  // Bottom-right
            Vec2::new(-0.5, -0.5), // Bottom-left
        ];
        
        // Transform corners
        let mut min_clip = Vec2::new(f32::MAX, f32::MAX);
        let mut max_clip = Vec2::new(f32::MIN, f32::MIN);
        let mut all_visible = true;
        
        for corner in &corners {
            let world_corner = world_pos + (*corner * scale);
            let world_vec4 = Vec4::new(world_corner.x, world_corner.y, 0.0, 1.0);
            let clip_pos = view_proj * world_vec4;
            
            // Check if w > 0 (in front of camera)
            if clip_pos.w <= 0.0 {
                all_visible = false;
            }
            
            // Perspective divide to get NDC
            let ndc = clip_pos.xy() / clip_pos.w;
            
            min_clip = min_clip.min(ndc);
            max_clip = max_clip.max(ndc);
        }
        
        // Check if sprite is within NDC bounds ([-1, 1] for x and y)
        let center_ndc = (min_clip + max_clip) * 0.5;
        let half_size = (max_clip - min_clip) * 0.5;
        
        let visible = all_visible && 
                      center_ndc.x.abs() + half_size.x <= 1.0 &&
                      center_ndc.y.abs() + half_size.y <= 1.0;
        
        SpriteVisibilityDebug {
            world_pos,
            clip_pos: view_proj * Vec4::new(world_pos.x, world_pos.y, 0.0, 1.0),
            ndc_pos: center_ndc,
            scale,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0), // White for debug
            visible,
        }
    }
}

const TEST_POSITIONS: &[Vec2] = &[
    Vec2::new(0.0, 0.0),       // Center
    Vec2::new(-100.0, 0.0),    // Left
    Vec2::new(100.0, 0.0),     // Right
    Vec2::new(0.0, 100.0),     // Top
    Vec2::new(0.0, -100.0),    // Bottom
    Vec2::new(-200.0, -150.0), // Bottom-left
    Vec2::new(200.0, 150.0),   // Top-right
];

impl ApplicationHandler<()> for DebugSpriteVisibilityDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🎯 DEBUG SPRITE VISIBILITY TEST");
        println!("=====================================");
        println!("Rendering {} sprites at known positions", TEST_POSITIONS.len());
        println!("Calculating clip-space coordinates...");
        println!("");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🎯 DEBUG: Sprite Visibility Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Adapter: {}", renderer.adapter_info());
        println!("✅ Initial viewport: 800x600");
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        // Setup camera with EXACT bounds
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        self.camera.zoom = 1.0;
        
        println!("📷 Camera position: {:?}", self.camera.position);
        println!("📷 Viewport size: {:?}", self.camera.viewport_size);
        println!("📷 Zoom: {}", self.camera.zoom);
        
        println!("\n🔍 DEBUG INFO - Calculating clip-space:");
        self.visibility_data.clear();
        
        for (i, &pos) in TEST_POSITIONS.iter().enumerate() {
            let debug = self.calculate_clip_space(pos, Vec2::new(50.0, 50.0));
            self.visibility_data.push(debug);
            
            let debug_ref = &self.visibility_data[i];
            println!("   Sprite {} at {:?}:", i, debug_ref.world_pos);
            println!("      Clip pos: {:?}", debug_ref.clip_pos);
            println!("      NDC pos: {:?}", debug_ref.ndc_pos);
            println!("      Visible: {}", debug_ref.visible);
            println!("      W > 0: {}", debug_ref.clip_pos.w > 0.0);
            println!();
        }
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.input_handler.lock().unwrap().handle_window_event(&event);
        
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.frame_count += 1;
    }
}

impl DebugSpriteVisibilityDemo {
    fn render_frame(&mut self) {
        let mut batcher = SpriteBatcher::new(100);
        let white_texture = TextureHandle { id: 0 };
        
        // Add test sprites with different colors based on visibility
        for (i, &pos) in TEST_POSITIONS.iter().enumerate() {
            let color = if self.visibility_data.get(i).map_or(true, |d| d.visible) {
                // Green for visible, Red for not visible
                match i % 2 {
                    0 => Vec4::new(0.0, 1.0, 0.0, 1.0), // Green
                    _ => Vec4::new(1.0, 0.0, 0.0, 1.0), // Red
                }
            } else {
                Vec4::new(0.5, 0.5, 0.5, 1.0) // Gray for invisible
            };
            
            let sprite = Sprite::new(white_texture)
                .with_position(pos)
                .with_color(color)
                .with_scale(Vec2::new(50.0, 50.0));
            
            batcher.add_sprite(&sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let texture_resources = std::collections::HashMap::new();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            if self.frame_count == 1 {
                println!("🎨 Rendering {} sprites...", TEST_POSITIONS.len());
                println!("   Expected viewport bounds: ±{:?} x ±{:?}", 
                    self.camera.viewport_size.x * 0.5, 
                    self.camera.viewport_size.y * 0.5);
                println!("\n🎯 LOOK FOR: Green/Red squares on dark background");
            }
            
            let _ = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("🎯 DEBUG SPRITE VISIBILITY TEST");
    println!("=====================================");
    println!("This demo will:");
    println!("1. Calculate clip-space coordinates for each sprite");
    println!("2. Show which sprites SHOULD be visible");
    println!("3. Render sprites with different colors based on visibility");
    println!("4. Help diagnose why sprites aren't appearing");
    println!("=====================================\n");

    let event_loop = EventLoop::new()?;
    let mut app = DebugSpriteVisibilityDemo::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
