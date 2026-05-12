//! Bloom post-processing pipeline.
//!
//! Runs three full-screen passes after the main scene has been drawn into the
//! HDR target:
//!
//! 1. **Extract** — read HDR, keep only pixels brighter than `threshold`,
//!    write to `bloom_ping` (half-resolution).
//! 2. **Blur** — separable Gaussian, ping-ponging between `bloom_ping` and
//!    `bloom_pong`. Repeats `blur_iterations` times for a wider glow.
//! 3. **Composite** — read HDR + final bloom, tonemap, write to the
//!    swapchain in sRGB.
//!
//! The output of the chain is the sRGB swapchain texture.

use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, FilterMode,
    PipelineLayout, Queue, RenderPipeline, Sampler, ShaderModule, TextureFormat, TextureView,
};

use crate::render_targets::{RenderTargets, HDR_FORMAT};

/// Tunables for the bloom pipeline. Mutate at runtime via
/// [`RenderManager::bloom_config_mut`](crate::RenderManager::bloom_config_mut).
#[derive(Debug, Clone, Copy)]
pub struct BloomConfig {
    /// Master switch — when false, the composite pass still runs (to do the
    /// tonemap), but bloom contribution is zeroed.
    pub enabled: bool,
    /// Luminance threshold for the bright pass. Pixels below this don't bloom.
    pub threshold: f32,
    /// Soft-knee width around the threshold to avoid banding.
    pub knee: f32,
    /// Multiplier applied to the bloom buffer during composite.
    pub intensity: f32,
    /// Number of horizontal+vertical blur iterations. Each iteration roughly
    /// doubles the effective blur radius. 2–4 is typical.
    pub blur_iterations: u32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 1.0,
            knee: 0.5,
            intensity: 0.8,
            blur_iterations: 3,
        }
    }
}

/// GPU layout for the extract/composite uniform buffer.
/// Padding makes this match the 16-byte alignment required for uniform buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BloomParams {
    threshold: f32,
    knee: f32,
    intensity: f32,
    _pad: f32,
}

/// GPU layout for the blur uniform buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurParams {
    texel_size: [f32; 2],
    direction: [f32; 2],
}

/// Owns the bloom render pipelines, samplers, and bind-group layouts.
pub struct BloomPipeline {
    extract_pipeline: RenderPipeline,
    blur_pipeline: RenderPipeline,
    composite_pipeline: RenderPipeline,

    extract_layout: BindGroupLayout,
    blur_layout: BindGroupLayout,
    composite_layout: BindGroupLayout,

    sampler: Sampler,

    /// Uniform buffer reused for the extract and composite passes.
    bloom_params_buffer: Buffer,
    /// Uniform buffer reused for both horizontal and vertical blur passes.
    blur_params_buffer: Buffer,
}

impl BloomPipeline {
    /// Build all three render pipelines. Format must match the swapchain.
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let extract_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Extract Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/bloom_extract.wgsl"))),
        });
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/bloom_blur.wgsl"))),
        });
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/bloom_composite.wgsl"))),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Bloom Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            // Bloom textures are single-mip so this is academic, but keep it
            // explicit to match the field's type in wgpu 28.
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let extract_layout = create_single_tex_layout(device, "Bloom Extract Layout");
        let blur_layout = create_single_tex_layout(device, "Bloom Blur Layout");
        let composite_layout = create_composite_layout(device);

        let extract_pipeline = build_fullscreen_pipeline(
            device,
            &extract_layout,
            &extract_shader,
            HDR_FORMAT,
            "Bloom Extract Pipeline",
        );
        let blur_pipeline = build_fullscreen_pipeline(
            device,
            &blur_layout,
            &blur_shader,
            HDR_FORMAT,
            "Bloom Blur Pipeline",
        );
        let composite_pipeline = build_fullscreen_pipeline(
            device,
            &composite_layout,
            &composite_shader,
            surface_format,
            "Bloom Composite Pipeline",
        );

        let bloom_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bloom Params Buffer"),
            contents: bytemuck::bytes_of(&BloomParams { threshold: 1.0, knee: 0.5, intensity: 0.8, _pad: 0.0 }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let blur_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur Params Buffer"),
            contents: bytemuck::bytes_of(&BlurParams { texel_size: [0.0, 0.0], direction: [1.0, 0.0] }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            extract_pipeline,
            blur_pipeline,
            composite_pipeline,
            extract_layout,
            blur_layout,
            composite_layout,
            sampler,
            bloom_params_buffer,
            blur_params_buffer,
        }
    }

    /// Run the full extract → blur → composite chain.
    ///
    /// `targets` holds the HDR + bloom ping-pong textures.
    /// `swapchain` is the destination view (sRGB).
    pub fn run(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        targets: &RenderTargets,
        swapchain: &TextureView,
        config: &BloomConfig,
    ) {
        // 1. Update uniform buffers.
        let intensity = if config.enabled { config.intensity } else { 0.0 };
        queue.write_buffer(
            &self.bloom_params_buffer,
            0,
            bytemuck::bytes_of(&BloomParams {
                threshold: config.threshold,
                knee: config.knee,
                intensity,
                _pad: 0.0,
            }),
        );

        // 2. Extract bright pass: HDR -> bloom_ping (half-res).
        let extract_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Extract Bind Group"),
            layout: &self.extract_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.bloom_params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&targets.hdr_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        self.run_fullscreen_pass(
            encoder,
            &self.extract_pipeline,
            &extract_bind,
            &targets.bloom_ping_view,
            "Bloom Extract",
        );

        // 3. Blur iterations — ping-pong horizontal then vertical. The
        // bloom_ping texture holds the live state at the start and end of
        // every iteration, so it's the source the composite pass reads.
        let texel = [1.0 / targets.bloom_width() as f32, 1.0 / targets.bloom_height() as f32];
        let iterations = config.blur_iterations.max(1);
        for _ in 0..iterations {
            self.blur_horizontal(device, queue, encoder, targets, texel);
            self.blur_vertical(device, queue, encoder, targets, texel);
        }

        // 4. Composite HDR + bloom -> swapchain.
        let composite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Composite Bind Group"),
            layout: &self.composite_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.bloom_params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&targets.hdr_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&targets.bloom_ping_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        self.run_fullscreen_pass(
            encoder,
            &self.composite_pipeline,
            &composite_bind,
            swapchain,
            "Bloom Composite",
        );
    }

    /// Horizontal blur: source = `bloom_ping`, destination = `bloom_pong`.
    fn blur_horizontal(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        targets: &RenderTargets,
        texel: [f32; 2],
    ) {
        self.write_blur_params(queue, texel, [1.0, 0.0]);
        let bind = self.blur_bind_group(device, &targets.bloom_ping_view);
        self.run_fullscreen_pass(encoder, &self.blur_pipeline, &bind, &targets.bloom_pong_view, "Bloom Blur H");
    }

    /// Vertical blur: source = `bloom_pong`, destination = `bloom_ping`.
    /// After a horizontal+vertical pair, `bloom_ping` holds the live blur
    /// result that the composite pass reads.
    fn blur_vertical(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        targets: &RenderTargets,
        texel: [f32; 2],
    ) {
        self.write_blur_params(queue, texel, [0.0, 1.0]);
        let bind = self.blur_bind_group(device, &targets.bloom_pong_view);
        self.run_fullscreen_pass(encoder, &self.blur_pipeline, &bind, &targets.bloom_ping_view, "Bloom Blur V");
    }

    fn write_blur_params(&self, queue: &Queue, texel: [f32; 2], direction: [f32; 2]) {
        queue.write_buffer(
            &self.blur_params_buffer,
            0,
            bytemuck::bytes_of(&BlurParams { texel_size: texel, direction }),
        );
    }

    fn blur_bind_group(&self, device: &Device, src_view: &TextureView) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Blur Bind Group"),
            layout: &self.blur_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.blur_params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn run_fullscreen_pass(
        &self,
        encoder: &mut CommandEncoder,
        pipeline: &RenderPipeline,
        bind_group: &BindGroup,
        target: &TextureView,
        label: &str,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn create_single_tex_layout(device: &Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn create_composite_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bloom Composite Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn build_fullscreen_pipeline(
    device: &Device,
    bind_layout: &BindGroupLayout,
    shader: &ShaderModule,
    color_format: TextureFormat,
    label: &str,
) -> RenderPipeline {
    let pipeline_layout: PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[bind_layout],
        ..Default::default()
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview_mask: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_config_defaults_are_sane() {
        let cfg = BloomConfig::default();
        assert!(cfg.enabled);
        assert!(cfg.threshold > 0.0);
        assert!(cfg.intensity > 0.0);
        assert!(cfg.blur_iterations >= 1);
    }

    #[test]
    fn bloom_params_struct_is_16_bytes() {
        // Must match the uniform buffer layout the shaders expect.
        assert_eq!(std::mem::size_of::<BloomParams>(), 16);
    }

    #[test]
    fn blur_params_struct_is_16_bytes() {
        assert_eq!(std::mem::size_of::<BlurParams>(), 16);
    }
}
