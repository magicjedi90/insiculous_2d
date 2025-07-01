use renderer::prelude::*;

#[test]
fn test_renderer_error() {
    // Test creating renderer errors
    let window_error = RendererError::WindowCreationError("Failed to create window".to_string());
    let surface_error = RendererError::SurfaceCreationError("Failed to create surface".to_string());
    let adapter_error = RendererError::AdapterCreationError("Failed to create adapter".to_string());
    let device_error = RendererError::DeviceCreationError("Failed to create device".to_string());
    let swap_chain_error =
        RendererError::SwapChainCreationError("Failed to create swap chain".to_string());
    let pipeline_error =
        RendererError::PipelineCreationError("Failed to create pipeline".to_string());
    let rendering_error = RendererError::RenderingError("Failed to render".to_string());

    // TODO: Assert that the errors are correctly created
    // Test the Display implementation for each error
    assert!(format!("{}", window_error).contains("Failed to create window"));
    assert!(format!("{}", surface_error).contains("Failed to create surface"));
    assert!(format!("{}", adapter_error).contains("Failed to create adapter"));
    assert!(format!("{}", device_error).contains("Failed to create device"));
    assert!(format!("{}", swap_chain_error).contains("Failed to create swap chain"));
    assert!(format!("{}", pipeline_error).contains("Failed to create pipeline"));
    assert!(format!("{}", rendering_error).contains("Failed to render"));

    // Test the Debug implementation
    assert!(format!("{:?}", window_error).contains("WindowCreationError"));
    assert!(format!("{:?}", surface_error).contains("SurfaceCreationError"));
    assert!(format!("{:?}", adapter_error).contains("AdapterCreationError"));
    assert!(format!("{:?}", device_error).contains("DeviceCreationError"));
    assert!(format!("{:?}", swap_chain_error).contains("SwapChainCreationError"));
    assert!(format!("{:?}", pipeline_error).contains("PipelineCreationError"));
    assert!(format!("{:?}", rendering_error).contains("RenderingError"));
}
