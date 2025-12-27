# 🚨 Emergency GPU Diagnostic Tools

Comprehensive diagnostic tools for investigating GPU presentation issues in the Insiculous 2D game engine.

## Problem Statement

When rendering appears to work (no errors, successful command submission) but only a black screen is displayed, these tools help identify the exact failure point in the presentation pipeline.

## 🔍 Diagnostic Tools

### 1. GPU Emergency Diagnostic (`gpu_emergency_diagnostic.rs`)

**Purpose**: Comprehensive analysis of every level of the WGPU pipeline

**Checks**:
- ✅ Surface presentation status
- ✅ Swap chain health and texture acquisition
- ✅ Command buffer execution
- ✅ Texture view creation and validation
- ✅ WGPU validation layers
- ✅ OpenGL ES backend issues (EGL warnings)
- ✅ Surface format compatibility
- ✅ Present mode issues
- ✅ Pixel data readback validation

**Usage**:
```bash
cargo run --example gpu_emergency_diagnostic --features tokio
```

### 2. Render Pipeline Inspector (`render_pipeline_inspector.rs`)

**Purpose**: Detailed logging of every single render operation

**Features**:
- 🔍 Logs every render pass operation
- 📊 Shows exact texture formats and pixel data
- ✅ Validates every GPU resource creation
- ⚠️ Detects silent failures in the pipeline
- ⏱️ Performance timing analysis
- 📈 Success rate statistics

**Usage**:
```rust
let inspector = RenderPipelineInspector::new();
// Inspector automatically logs all GPU operations
```

### 3. Sprite Rendering Diagnostic (`sprite_rendering_diagnostic.rs`)

**Purpose**: Combines sprite rendering with real-time pipeline inspection

**Features**:
- 🎯 Renders diagnostic sprites (high-visibility colors)
- 🔍 Integrated pipeline monitoring
- 📊 Real-time success/failure tracking
- ⚡ Performance metrics
- 🎨 Pixel data validation

**Usage**:
```bash
cargo run --example sprite_rendering_diagnostic --features tokio
```

### 4. Simple GPU Diagnostic (`simple_gpu_diagnostic.rs`)

**Purpose**: Quick basic functionality test

**Usage**:
```bash
cargo run --example simple_gpu_diagnostic --features tokio
```

## 🚀 Quick Start

### Run All Diagnostics

Use the comprehensive diagnostic script:

```bash
./run_gpu_diagnostics.sh
```

This runs:
1. Simple GPU diagnostic
2. Emergency GPU diagnostic  
3. Sprite rendering with diagnostics
4. Backend-specific testing
5. Validation layer analysis

### Individual Tests

```bash
# Basic functionality
cargo run --example simple_gpu_diagnostic --features tokio

# Comprehensive analysis
cargo run --example gpu_emergency_diagnostic --features tokio

# Sprite rendering with monitoring
cargo run --example sprite_rendering_diagnostic --features tokio
```

## 📊 Understanding Results

### Key Metrics to Check

1. **Surface Acquisition**: `Can Acquire Texture: true/false`
   - If false: Swap chain is broken
   - Check surface format compatibility

2. **Command Buffer Submission**: `Commands Submitted: true/false`
   - If false: GPU commands not executing
   - Check validation layer errors

3. **Presentation Success**: `Presentation Successful: true/false`
   - If false: Frame not being displayed
   - Check present mode compatibility

4. **Pixel Data Validation**: Check sample pixel color
   - Should match expected clear color
   - If mismatch: Rendering pipeline issue

5. **Backend Issues**: Look for OpenGL ES warnings
   - "can present but not natively" warnings
   - EGL compatibility issues

### Present Mode Results

```
✅ Fifo: Success     # Most compatible, try first
✅ Mailbox: Success  # Good performance
✅ Immediate: Success # Lowest latency
❌ AutoVsync: Failed # May indicate fundamental issues
```

### Common Error Patterns

#### Pattern 1: Surface Acquisition Fails
```
❌ Can Acquire Texture: false
Acquisition Error: SurfaceLost
```
**Solution**: Recreate surface, check window lifecycle

#### Pattern 2: Command Submission Fails
```
❌ Commands Submitted: false
Submission Error: Validation failed
```
**Solution**: Enable validation layers, check resource creation

#### Pattern 3: Presentation Fails
```
✅ Render Successful: true
❌ Presentation Successful: false
```
**Solution**: Try different present modes, check compositor

#### Pattern 4: OpenGL ES Issues
```
Backend: Gl
⚠️ EGL Warnings: OpenGL ES may have EGL presentation limitations
```
**Solution**: Force Vulkan backend if available

## 🔧 Common Fixes

### Present Mode Issues
```rust
// Try different present modes
config.present_mode = wgpu::PresentMode::Mailbox; // or Immediate
```

### Backend Issues
```bash
# Force Vulkan backend
export WGPU_BACKEND=vulkan

# Force OpenGL ES
export WGPU_BACKEND=gl
```

### Validation Layers
```bash
# Enable maximum validation
export WGPU_VALIDATION=1
export WGPU_DEBUG=1
```

### Surface Format
```rust
// Try different surface formats
let format = surface_caps.formats.iter()
    .find(|f| f.is_srgb())
    .unwrap_or(&surface_caps.formats[0]);
```

## 🐛 Reporting Issues

When reporting GPU issues, include:

1. **Complete diagnostic output** from all tools
2. **GPU model and driver version**
3. **Operating system and window manager**
4. **Backend compatibility results**
5. **Present mode test results**
6. **Validation layer output**

### Debug Information to Collect

```bash
# System info
lspci | grep -i vga
 glxinfo | grep "OpenGL version"

# Environment
env | grep WGPU
echo $DISPLAY
echo $WAYLAND_DISPLAY
```

## 🎯 Integration with Existing Code

### Adding Diagnostics to Your App

```rust
use renderer::{
    render_pipeline_inspector::RenderPipelineInspector,
    gpu_emergency_diagnostic::GPUEmergencyDiagnostic,
};

// Add inspector to your app
let inspector = RenderPipelineInspector::new();

// Use inspected operations
let frame = inspector.inspect_surface_acquisition(renderer.surface(), || {
    renderer.surface().get_current_texture()
})?;
```

### Conditional Diagnostics

```rust
#[cfg(feature = "diagnostics")]
{
    let mut diagnostic = GPUEmergencyDiagnostic::new(window).await?;
    let results = diagnostic.run_full_diagnostics().await;
    log::info!("{}", diagnostic.generate_report(&results));
}
```

## 🚀 Advanced Usage

### Custom Validation

```rust
// Validate texture content
let is_valid = inspector.validate_texture_content(
    device, queue, texture, [1.0, 0.0, 0.0, 1.0] // Expected red color
)?;
```

### Performance Analysis

```rust
// Enable timing analysis
inspector.configure(true, true, true); // logging, validation, timing

// Generate performance report
let report = inspector.generate_report();
```

### Backend Testing

```rust
// Test multiple backends
for backend in ["vulkan", "gl", "dx12", "metal"] {
    std::env::set_var("WGPU_BACKEND", backend);
    // Run diagnostic...
}
```

## 📚 References

- [WGPU Documentation](https://docs.rs/wgpu/)
- [WGPU Examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
- [Game Programming Patterns](https://gameprogrammingpatterns.com/)
- [OpenGL ES Presentation Issues](https://www.khronos.org/opengles/)

## 🤝 Contributing

When adding new diagnostic capabilities:

1. Follow the existing logging patterns
2. Add comprehensive error messages
3. Include timing information
4. Document expected vs actual results
5. Test on multiple backends
6. Update this README with new findings

---

**Remember**: These tools are designed to expose every detail of the GPU pipeline. When in doubt, enable maximum logging and collect all diagnostic output for analysis.