//! 🔍 **VERTEX FORMAT TEST** - Verify vertex data is correct

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::{Vec2, Vec3, Vec4};

struct VertexFormatTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl VertexFormatTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
    
    fn dump_vertex_data(&self) {
        println("\n🔍 VERTEX DATA DUMP:");
        println("====================");
        
        // Create vertices manually to see exact layout
        let vertices = [
            // position (3 floats) | tex_coords (2 floats) | color (4 floats)
            -0.5f32,  0.5, 0.0,    0.0, 0.0,    1.0, 1.0, 1.0, 1.0,  // top-left
             0.5,     0.5, 0.0,    1.0, 0.0,    1.0, 1.0, 1.0, 1.0,  // top-right
             0.5,    -0.5, 0.0,    1.0, 1.0,    1.0, 1.0, 1.0, 1.0,  // bottom-right
            -0.5,    -0.5, 0.0,    0.0, 1.0,    1.0, 1.0, 1.0, 1.0,  // bottom-left
        ];
        
        println("Vertex data ({} bytes total):", vertices.len() * 4);
        
        // Print first vertex in detail
        println("\nVertex 0 (first 9 floats):");
        println("  position: {}, {}, {} (bytes 0-11)", vertices[0], vertices[1], vertices[2]);
        println("  tex_coords: {}, {} (bytes 12-19)", vertices[3], vertices[4]);
        println("  color: {}, {}, {}, {} (bytes 20-35)", vertices[5], vertices[6], vertices[7], vertices[8]);
        
        // Check vertex format
        let sprite_vertex = SpriteVertex::new(
            Vec3::new(-0.5, 0.5, 0.0),
            Vec2::new(0.0, 0.0),
            Vec4::ONE
        );
        
        println("\nSpriteVertex struct:");
        println("  size: {} bytes", std::mem::size_of::<SpriteVertex>());
        println("  position offset: 0 (3 floats)");
        println!("  tex_coords offset: {} (2 floats)", std::mem::size_of::<[f32; 3]>());
        println!("  color offset: {} (4 floats)", std::mem::size_of::<[f32; 5]>());
        
        // Check layout
        let layout = SpriteVertex::desc();
        println!("\nVertexBufferLayout:");
        println!("  array_stride: {}", layout.array_stride);
        println!("  attributes count: {}", layout.attributes.len());
        for (i, attr) in layout.attributes.iter().enumerate() {
            println!("  [{}] shader_location={}, format={:?}, offset={}", 
                     i, attr.shader_location, attr.format, attr.offset);
        }
        
        println("\n====================\n");
    }
}

impl ApplicationHandler<()> for VertexFormatTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println("🔍 VERTEX FORMAT TEST");
        println("=====================");
        println("Verifying vertex data and format match");
        println("=====================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔍 Vertex Format Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        
        self.dump_vertex_data();
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        
        println("🎨 Rendering sprite with these vertices...\n");
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

impl VertexFormatTest {
    fn render_frame(&mut self) {
        if self.frame_count == 0 {
            println("🎨 Rendering frame {}...", self.frame_count);
        }
        
        let white_texture = TextureHandle { id: 0 };
        let sprite = Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0))
            .with_scale(Vec2::new(200.0, 200.0));
        
        let mut batcher = SpriteBatcher::new(100);
        batcher.add_sprite(&sprite);
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println("🔍 VERTEX FORMAT TEST");
    println("=====================");
    println("This test checks if vertex data format matches shader");
    println("");
    println("🔧 If format mismatch: vertices won't reach shader");
    println("🔧 If format matches: we should see sprites");
    println("=====================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = VertexFormatTest::new();
    event_loop.run_app(&mut app)?;
    
    println("\n✅ Test complete!");
    Ok(())
}
