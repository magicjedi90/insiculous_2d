//! 👁️ **VISIBLE SPRITE TEST** - Stays open for 3 seconds so you can SEE it

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::Vec2;

struct VisibleSpriteTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
    start_time: Option<std::time::Instant>,
}

impl VisibleSpriteTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
            start_time: None,
        }
    }
}

impl ApplicationHandler<()> for VisibleSpriteTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("👁️ VISIBLE SPRITE TEST - Will stay open for 3 seconds");
        println!("=====================================================");
        println!("🎨 Shader is hard-coded to output MAGENTA (1,0,1,1)");
        println!("");
        println!("WHAT TO LOOK FOR:");
        println!("✅ MAGENTA SQUARE = Shader is working");
        println!("❌ BLUE BACKGROUND ONLY = Fragment shader not running");
        println!("❌ CRASH = Validation error");
        println!("=====================================================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("👁️ VISIBLE SPRITE TEST - MAGENTA expected!")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        println!("✅ Sprite pipeline created");
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        println!("✅ Camera configured");
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        self.start_time = Some(std::time::Instant::now());
        
        println!("\n🎬 Rendering... LOOK AT THE WINDOW NOW!\n");
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
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            if elapsed.as_secs() < 3 {
                // Keep rendering for 3 seconds
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                
                if self.frame_count % 60 == 0 {
                    println!("⏱️  Time elapsed: {}s - Keep looking at the window!", elapsed.as_secs());
                }
            } else {
                println!("\n⏰ 3 seconds elapsed - exiting...");
                event_loop.exit();
            }
        }
    }
}

impl VisibleSpriteTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Large sprite filling center of screen
        let sprite = Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0))  // Should be overridden by shader to MAGENTA
            .with_scale(Vec2::new(300.0, 300.0));
        
        let mut batcher = SpriteBatcher::new(100);
        batcher.add_sprite(&sprite);
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎨 Rendering frame {}... LOOK FOR MAGENTA!", self.frame_count);
            }
            
            let _ = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
            
            if self.frame_count == 0 {
                println!("✅ First render call succeeded");
            }
        }
        
        self.frame_count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("👁️  VISIBLE SPRITE TEST");
    println!("======================");
    println!("This test will STAY OPEN for 3 seconds!");
    println!("");
    println!("👀 LOOK FOR: A MAGENTA COLORED SQUARE");
    println!("");
    println!("SHADER STATUS: Hard-coded to output magenta (1,0,1,1)");
    println!("");
    println!("📋 Expected results:");
    println!("   ✅ SEE MAGENTA = Fragment shader IS running");
    println!("   ❌ BLUE ONLY = Fragment shader is NOT running");
    println!("   💥 CRASH/WINDOW CLOSES = Validation error");
    println!("======================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = VisibleSpriteTest::new();
    event_loop.run_app(&mut app)?;
    
    println!("\n✅ Test complete!");
    println!("🎯 Did you see a MAGENTA square?");
    Ok(())
}
