//! Window management for the renderer.

use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::error::RendererError;

/// Configuration for the window
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Title of the window
    pub title: String,
    /// Width of the window
    pub width: u32,
    /// Height of the window
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "insiculous_2d".to_string(),
            width: 800,
            height: 600,
            resizable: true,
        }
    }
}

/// Create a new window with the given configuration using an ActiveEventLoop
pub fn create_window_with_active_loop(
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
) -> Result<Window, RendererError> {
    // Create window attributes
    let mut attributes = WindowAttributes::default();
    attributes.title = config.title.clone();
    attributes.inner_size = Some(PhysicalSize::new(config.width, config.height).into());
    attributes.resizable = config.resizable;

    // Create the window using ActiveEventLoop's create_window method
    let window = event_loop.create_window(attributes)
        .map_err(|e| RendererError::WindowCreationError(e.to_string()))?;

    Ok(window)
}

// The deprecated create_window function has been removed.
// Use create_window_with_active_loop instead with an ActiveEventLoop.
