//! Window management for the renderer.

use winit::{
    event_loop::{EventLoop, ActiveEventLoop},
    window::{Window, WindowAttributes},
    dpi::PhysicalSize,
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

/// Create a new window with the given configuration using an EventLoop
/// This is a wrapper around create_window_with_active_loop that uses the deprecated create_window method
pub fn create_window(
    config: &WindowConfig,
    event_loop: &EventLoop<()>,
) -> Result<Window, RendererError> {
    // Create window attributes
    let mut attributes = WindowAttributes::default();
    attributes.title = config.title.clone();
    attributes.inner_size = Some(PhysicalSize::new(config.width, config.height).into());
    attributes.resizable = config.resizable;

    // Create the window using the deprecated create_window method
    // Note: create_window is deprecated but still works in winit 0.30
    // In a future update, this should be replaced with ActiveEventLoop::create_window
    #[allow(deprecated)]
    let window = event_loop.create_window(attributes)
        .map_err(|e| RendererError::WindowCreationError(e.to_string()))?;

    Ok(window)
}
