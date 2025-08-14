//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use winit::{
    application::ApplicationHandler,
    window::Window,
};

mod error;
mod renderer;
pub mod sprite;
mod window;

pub mod prelude;

// Re-export for convenience
pub use error::*;
pub use renderer::*;
pub use sprite::*;
pub use window::*;

/// Time resource for tracking delta time
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// Delta time in seconds
    pub delta_seconds: f32,
    /// Total elapsed time in seconds
    pub elapsed_seconds: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
        }
    }
}

/// Engine state that outlives the event-loop callback
pub struct EngineState {
    /// The renderer
    pub renderer: Renderer<'static>,
    /// The tokio runtime
    pub runtime: Runtime,
    /// Time resource
    pub time: Time,
    /// Sprite pipeline
    pub sprite_pipeline: Option<SpritePipeline>,
}

/// Create a tokio runtime for the renderer
pub fn create_tokio_runtime() -> Result<Runtime, RendererError> {
    // Create a tokio runtime with the current_thread flavor
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| RendererError::RuntimeCreationError(e.to_string()))?;

    log::debug!("Tokio runtime created");
    Ok(runtime)
}

/// Initialize the renderer with an existing window
pub async fn init(window: Arc<Window>) -> Result<Renderer<'static>, RendererError> {
    log::info!("Initializing renderer");
    Renderer::new(window).await
}

/// Run the renderer with a custom application handler
pub fn run_with_app<T>(app: &mut T) -> Result<(), RendererError>
where
    T: ApplicationHandler<()> + 'static
{
    log::info!("Running renderer with custom application handler");
    Renderer::run_with_app(app)
}

/// Initialize the engine state
pub async fn init_engine_state(window: Arc<Window>) -> Result<EngineState, RendererError> {
    // Create tokio runtime
    let runtime = create_tokio_runtime()?;

    // Initialize renderer inside tokio runtime
    let renderer = tokio::task::block_in_place(|| {
        runtime.block_on(async {
            init(window).await
        })
    })?;

    log::debug!("Renderer initialized with adapter: {}", renderer.adapter_info());
    log::debug!("Surface format: {:?}", renderer.surface_format());
    log::debug!("Swap chain size: {}x{}", renderer.surface_width(), renderer.surface_height());

    // Create sprite pipeline
    let sprite_pipeline = Some(SpritePipeline::new(renderer.device()));

    Ok(EngineState {
        renderer,
        runtime,
        time: Time::default(),
        sprite_pipeline,
    })
}
