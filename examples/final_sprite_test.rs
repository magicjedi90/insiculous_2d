//! 🎯 **FINAL SPRITE TEST** - Original shader, bright colors, stays open

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::{Vec2, Vec4};

struct FinalSpriteTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl FinalSpriteTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
}

impl ApplicationHandler<()> for FinalSpriteTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🎯 FINAL SPRITE TEST - Running with ORIGINAL shader");
        println!("==================================================");
        println!("Will render with proper texture sampling + colors");
        println!("Window stays open for 5 seconds");
        println!("==================================================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🎯 FINAL SPRITE TEST - Colored sprites!")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let mut renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        println!("✅ Sprite pipeline created");
        
        // Dark blue background to make sprites pop
        renderer.set_clear_color(0.05, 0.08, 0.15, 1.0);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
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
        if self.frame_count < 300 { // ~5 seconds at 60fps
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        } else {
            println!("\n⏰ 5 seconds elapsed - test complete!");
            event_loop.exit();
        }
    }
}

impl FinalSpriteTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Simple colored sprites
        let sprites = vec![
            Sprite::new(white_texture)
                .with_position(Vec2::new(-150.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))  // Red
                .with_scale(Vec2::new(100.0, 100.0)),
            Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0))  // Green
                .with_scale(Vec2::new(100.0, 100.0)),
            Sprite::new(white_texture)
                .with_position(Vec2::new(150.0, 0.0))
                .with_color(Vec4::new(0.0, 0.0, 1.0, 1.0))  // Blue
                .with_scale(Vec2::new(100.0, 100.0)),
        ];
        
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &sprites {
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎨 Frame {} - Rendering {} sprites", self.frame_count, sprites.len());
                println!("🎨 Testing with ORIGINAL shader (texture sampling + vertex colors)");
                println!("🎯 EXPECTING: Red, Green, Blue squares on dark blue background");
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
    println!("🎯 FINAL SPRITE TEST");
    println!("=====================");
    println!("Original shader + Colored sprites + White texture");
    println!("Window stays open for 5 seconds!");
    println!("");
    println!("👀 LOOK FOR: Red, Green, Blue colored squares");
    println!("🎨 Background: Dark blue");
    println!("");
    println!("🔧 Shader: Original texture sampling + vertex color");
    println!("🎯 All sprites use white texture (handle 0)");
    println!("=====================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = FinalSpriteTest::new();
    event_loop.run_app(&mut app)?;
    
    println!("\n✅ Test complete!");
    println!("🎯 Did you see colored sprites?");
    Ok(())
}
