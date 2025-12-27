//! 🔧 **VERTEX DEBUG TEST** - Shader outputs vertex position colors

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::Vec2;

struct VertexDebugTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl VertexDebugTest {
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

impl ApplicationHandler<()> for VertexDebugTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔧 VERTEX DEBUG TEST");
        println("====================");
        println("Shader outputs colors based on vertex position:");
        println("  Top-left = RED");
        println("  Top-right = GREEN");
        println("  Bottom-left = BLUE/YELLOW");
        println("  Bottom-right = YELLOW");
        println("");
        println("If you see ANY colors: vertices ARE being processed");
        println("If you see ONLY background: vertex shader not running");
        println("====================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔧 VERTEX DEBUG - Look for colors!")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        // We need to patch the shader module in the pipeline
        // For now, use the debug shader
        let sprite_pipeline = SpritePipeline::new_with_debug_shader(renderer.device(), 100);
        println!("✅ Sprite pipeline created (with debug shader)");
        
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
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            if FRAME_COUNT < 180 { // 3 seconds
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME_COUNT += 1;
            } else {
                event_loop.exit();
            }
        }
    }
}

impl VertexDebugTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Large sprite so we can see the vertex colors
        let sprite = Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(glam::Vec4::new(1.0, 1.0, 1.0, 1.0))  // White color (will be overridden by debug)
            .with_scale(Vec2::new(300.0, 300.0));
        
        let mut batcher = SpriteBatcher::new(100);
        batcher.add_sprite(&sprite);
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println("🎨 Rendering debug frames... LOOK FOR COLORS!");
                println("   Expected: Gradient of colors showing vertex positions");
            }
            
            let _ = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
        }
        
        self.frame_count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 VERTEX DEBUG TEST");
    println("====================");
    println("Testing if vertex shader is running...");
    println("");
    println("👀 LOOK FOR: Colored gradients (red, green, blue, yellow)");
    println("");
    println("📋 Results:");
    println("   ✅ See colors = Vertex shader IS running");
    println("   ❌ See only background = Vertex shader NOT running");
    println("====================\n");
    
    // Patch the sprite pipeline to use debug shader
    // We need to modify the sprite.rs to support this
    
    let event_loop = EventLoop::new()?;
    let mut app = VertexDebugTest::new();
    event_loop.run_app(&mut app)?;
    
    println("\n✅ Test complete!");
    println("Did you see colored gradients?");
    Ok(())
}
