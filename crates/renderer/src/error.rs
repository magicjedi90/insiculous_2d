//! Error types for the renderer.

/// Errors that can occur in the renderer
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Failed to create window: {0}")]
    WindowCreationError(String),

    #[error("Failed to create surface: {0}")]
    SurfaceCreationError(String),

    #[error("Failed to create adapter: {0}")]
    AdapterCreationError(String),

    #[error("Failed to create device: {0}")]
    DeviceCreationError(String),

    #[error("Failed to create swap chain: {0}")]
    SwapChainCreationError(String),

    #[error("Failed to create render pipeline: {0}")]
    PipelineCreationError(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),
}
