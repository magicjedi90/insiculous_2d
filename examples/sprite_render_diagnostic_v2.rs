//! 🔍 **SPRITE RENDER DIAGNOSTIC v2** - Detailed logging of every render step

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

struct SpriteRenderDiagnostic {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    frame_count: u32,
}

impl SpriteRenderDiagnostic {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(std::sync::Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
    
    fn log_render_state(&self, sprites: &[Sprite]) {
        println!("\n📊 RENDER STATE - Frame {}", self.frame_count);
        println!("========================================");
        println!("📷 Camera:");
        println!("   Position: {:?}", self.camera.position);
        println!("   Viewport: {:?}", self.camera.viewport_size);
        println!("   Zoom: {}", self.camera.zoom);
        
        let view_proj = self.camera.view_projection_matrix();
        println!("   ViewProj matrix: {:?}", view_proj);
        
        println!("\n🎨 Sprites ({}/{:?}):", sprites.len(), self.sprite_pipeline.as_ref().map(|p| p.max_sprites_per_batch()));
        for (i, sprite) in sprites.iter().enumerate() {
            let world_pos = Vec4::new(sprite.position.x, sprite.position.y, sprite.depth, 1.0);
            let clip_pos = view_proj * world_pos;
            let ndc = clip_pos.xy() / clip_pos.w;
            
            println!("   Sprite {}: pos={:?}, scale={:?}", i, sprite.position, sprite.scale);
            println!("      Clip: {:?}, NDC: {:?}, W={}", clip_pos, ndc, clip_pos.w);
            println!("      Color: {:?}", sprite.color);
            println!("      In bounds: {}", ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0);
        }
        println!("========================================\n");
    }
}

impl ApplicationHandler<()> for SpriteRenderDiagnostic {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔍 SPRITE RENDER DIAGNOSTIC v2");
        println!("================================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔍 Sprite Render Diagnostic v2")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
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

impl SpriteRenderDiagnostic {
    fn render_frame(&mut self) {
        // Create test sprite at exact center
        let white_texture = TextureHandle { id: 0 };
        let sprites = vec![
            Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))  // Bright red
                .with_scale(Vec2::new(200.0, 200.0)),
        ];
        
        if self.frame_count < 5 {
            self.log_render_state(&sprites);
        }
        
        // Batch sprites
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &sprites {
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎨 Attempting to render {} sprites...", sprites.len());
                println!("   Batches created: {}", batch_refs.len());
                for (i, batch) in batch_refs.iter().enumerate() {
                    println!("   Batch {}: texture={:?}, instances={}", i, batch.texture_handle, batch.len());
                }
            }
            
            let result = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
            
            if let Err(e) = result {
                println!("❌ Render error: {}", e);
            } else if self.frame_count == 0 {
                println!("✅ Render call succeeded");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new()?;
    let mut app = SpriteRenderDiagnostic::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
