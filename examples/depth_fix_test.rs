//! 🔍 **DEPTH FIX TEST** - Check if sprites are being clipped by far plane

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

struct DepthFixTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    frame_count: u32,
}

impl DepthFixTest {
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
}

impl ApplicationHandler<()> for DepthFixTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔍 DEPTH FIX TEST");
        println!("==================");
        println!("Testing different depth values to find clipping issue");
        println!("==================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔍 Depth Fix Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        
        let mut sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        // Setup camera with different near/far values
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        self.camera.near = -1000.0;  // Negative near plane
        self.camera.far = 1000.0;     // Far plane
        
        println!("📷 Camera near: {}, far: {}", self.camera.near, self.camera.far);
        
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

impl DepthFixTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Create sprites at different depths
        let depths = [-500.0, -100.0, -10.0, 0.0, 10.0, 100.0, 500.0];
        let colors = [
            Vec4::new(1.0, 0.0, 0.0, 1.0), // Red
            Vec4::new(0.0, 1.0, 0.0, 1.0), // Green
            Vec4::new(0.0, 0.0, 1.0, 1.0), // Blue
            Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            Vec4::new(1.0, 0.0, 1.0, 1.0), // Magenta
            Vec4::new(0.0, 1.0, 1.0, 1.0), // Cyan
            Vec4::new(1.0, 1.0, 1.0, 1.0), // White
        ];
        
        let mut batcher = SpriteBatcher::new(100);
        
        if self.frame_count == 0 {
            println!("\n🎨 Creating {} sprites at different depths:", depths.len());
        }
        
        for (i, (&depth, &color)) in depths.iter().zip(colors.iter()).enumerate() {
            let x = (i as f32 - 3.0) * 100.0; // Spread horizontally
            let y = (depth / 100.0) * 50.0;   // Small vertical offset based on depth
            
            let sprite = Sprite::new(white_texture)
                .with_position(Vec2::new(x, y))
                .with_color(color)
                .with_scale(Vec2::new(80.0, 80.0))
                .with_depth(depth);
            
            batcher.add_sprite(&sprite);
            
            if self.frame_count == 0 {
                let view_proj = self.camera.view_projection_matrix();
                let world_pos = Vec4::new(x, y, depth, 1.0);
                let clip_pos = view_proj * world_pos;
                let ndc = clip_pos.xy() / clip_pos.w;
                
                println!("   Depth {}: pos={:?}, NDC={:?}, visible={}", 
                    depth, Vec2::new(x, y), ndc, 
                    depth >= self.camera.near && depth <= self.camera.far && ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0);
            }
        }
        
        if self.frame_count == 0 {
            println!("\n🎯 EXPECTING: Rainbow row of squares at different depths");
        }
        
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
            
            if self.frame_count == 0 {
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
    let mut app = DepthFixTest::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
