//! 🔍 **SPRITE VISIBILITY DEBUG** - Log every detail to find the bug

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::{Vec2, Vec4};

struct SpriteVisibilityDebug {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl SpriteVisibilityDebug {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
    
    fn log_sprite_details(&self, sprite: &Sprite) {
        let view_proj = self.camera.view_projection_matrix();
        let world_pos = Vec4::new(sprite.position.x, sprite.position.y, sprite.depth, 1.0);
        let clip_pos = view_proj * world_pos;
        let ndc = clip_pos.truncate() / clip_pos.w;
        
        println!("📊 Sprite Debug:");
        println!("   Position: {:?}", sprite.position);
        println!("   Scale: {:?}", sprite.scale);
        println!("   Color: {:?}", sprite.color);
        println!("   Depth: {}", sprite.depth);
        println!("   Texture: {:?}", sprite.texture_handle);
        println!("   Clip: {:?}", clip_pos);
        println!("   NDC: {:?}", ndc);
        
        // Check if within clip bounds
        let half_size = sprite.scale * 0.5;
        let corners = [
            Vec2::new(sprite.position.x - half_size.x, sprite.position.y - half_size.y),
            Vec2::new(sprite.position.x + half_size.x, sprite.position.y - half_size.y),
            Vec2::new(sprite.position.x - half_size.x, sprite.position.y + half_size.y),
            Vec2::new(sprite.position.x + half_size.x, sprite.position.y + half_size.y),
        ];
        
        println!("   Corners in NDC:");
        for (i, corner) in corners.iter().enumerate() {
            let corner_clip = view_proj * Vec4::new(corner.x, corner.y, sprite.depth, 1.0);
            let corner_ndc = corner_clip.truncate() / corner_clip.w;
            let in_bounds = corner_ndc.x.abs() <= 1.0 && corner_ndc.y.abs() <= 1.0;
            println!("      [{}] {:?} -> {:?} (in_bounds: {})", i, corner, corner_ndc, in_bounds);
        }
    }
}

impl ApplicationHandler<()> for SpriteVisibilityDebug {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔍 SPRITE VISIBILITY DEBUG");
        println!("==========================");
        println!("Will log every detail about sprite rendering");
        println!("==========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔍 Sprite Visibility Debug")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer initialized: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        println!("✅ Sprite pipeline created");
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        self.camera.zoom = 1.0;
        println!("✅ Camera configured: viewport={:?}, position={:?}", self.camera.viewport_size, self.camera.position);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        
        println!("\n🎨 Ready to render test sprite...\n");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        if self.frame_count > 0 {
            event_loop.exit();
        }
    }
}

impl SpriteVisibilityDebug {
    fn render_frame(&mut self) {
        if self.frame_count == 0 {
            println!("🎯 Rendering frame {}...", self.frame_count);
            
            // Create a simple, large, bright red sprite at EXACT center with white texture
            let white_texture = TextureHandle { id: 0 };
            let sprite = Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))  // Bright red, 100% opacity
                .with_scale(Vec2::new(200.0, 200.0))        // Large: 200x200 units
                .with_depth(0.5);                           // Middle of depth range
            
            println!("\n📋 Sprite configuration:");
            self.log_sprite_details(&sprite);
            
            // Check viewport bounds
            println!("\n📐 Viewport bounds:");
            println!("   Camera pos: {:?}", self.camera.position);
            println!("   Viewport: {:?}", self.camera.viewport_size);
            println!("   Half-width: {}", self.camera.viewport_size.x * 0.5);
            println!("   Half-height: {}", self.camera.viewport_size.y * 0.5);
            println!("   Near: {}, Far: {}", self.camera.near, self.camera.far);
            
            // Create batch
            let mut batcher = SpriteBatcher::new(100);
            batcher.add_sprite(&sprite);
            
            let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
            let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
            let texture_resources = std::collections::HashMap::new();
            
            println!("\n📦 Batch info:");
            println!("   Batches created: {}", batch_refs.len());
            for (i, batch) in batch_refs.iter().enumerate() {
                println!("   Batch [{}]: texture={:?}, instances={}", i, batch.texture_handle, batch.len());
            }
            
            if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
                println!("\n🎬 Calling render_with_sprites...");
                
                let result = renderer.render_with_sprites(
                    sprite_pipeline, 
                    &self.camera, 
                    &texture_resources, 
                    &batch_refs
                );
                
                match result {
                    Ok(_) => println!("✅ render_with_sprites succeeded"),
                    Err(e) => println!("❌ render_with_sprites failed: {}", e),
                }
            }
            
            println!("\n🎯 Expected: Large RED square at center of window");
            println!("❌ If you see only blue background: sprite not rendering\n");
        }
        
        self.frame_count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new()?;
    let mut app = SpriteVisibilityDebug::new();
    event_loop.run_app(&mut app)?;
    
    println!("\n✅ Debug complete! Check if you saw the RED sprite.");
    Ok(())
}
