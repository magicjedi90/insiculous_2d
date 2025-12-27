//! Render pipeline inspector for detailed logging of every GPU operation
//!
//! This module provides comprehensive logging and validation of the entire
//! rendering pipeline to identify silent failures and presentation issues.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Instant, Duration};
use wgpu::{
    Queue, Surface, RenderPass, Buffer, BindGroup, RenderPassDescriptor, LoadOp,
    SurfaceError, CommandEncoder, RenderPipeline,
};

/// Detailed logging of every render operation
#[derive(Debug, Clone)]
pub struct RenderOperation {
    pub timestamp: Instant,
    pub operation_type: RenderOperationType,
    pub details: String,
    pub success: bool,
    pub error: Option<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum RenderOperationType {
    SurfaceAcquisition,
    TextureViewCreation,
    CommandEncoderCreation,
    RenderPassBegin,
    RenderPassEnd,
    PipelineBind,
    BufferBind,
    TextureBind,
    DrawCall,
    CommandSubmission,
    Presentation,
    TextureCreation,
    BufferCreation,
    ResourceValidation,
}

/// Comprehensive render pipeline inspector
pub struct RenderPipelineInspector {
    operations: Arc<Mutex<VecDeque<RenderOperation>>>,
    max_operations: usize,
    enable_detailed_logging: bool,
    enable_resource_validation: bool,
    enable_timing: bool,
}

impl RenderPipelineInspector {
    /// Create a new render pipeline inspector
    pub fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(VecDeque::new())),
            max_operations: 1000,
            enable_detailed_logging: true,
            enable_resource_validation: true,
            enable_timing: true,
        }
    }

    /// Configure inspector settings
    pub fn configure(&mut self, detailed_logging: bool, resource_validation: bool, timing: bool) {
        self.enable_detailed_logging = detailed_logging;
        self.enable_resource_validation = resource_validation;
        self.enable_timing = timing;
    }

    /// Log a render operation
    fn log_operation(&self, operation_type: RenderOperationType, details: String, success: bool, error: Option<String>, duration: Option<Duration>) {
        if !self.enable_detailed_logging {
            return;
        }

        let operation = RenderOperation {
            timestamp: Instant::now(),
            operation_type,
            details,
            success,
            error,
            duration,
        };

        let mut operations = self.operations.lock().unwrap();
        operations.push_back(operation);

        // Keep only recent operations
        while operations.len() > self.max_operations {
            operations.pop_front();
        }
    }

    /// Inspect surface acquisition with detailed logging
    pub fn inspect_surface_acquisition<F, R>(&self, _surface: &Surface, operation: F) -> Result<R, SurfaceError>
    where
        F: FnOnce() -> Result<R, SurfaceError>,
    {
        if !self.enable_detailed_logging {
            return operation();
        }

        let start = if self.enable_timing { Some(Instant::now()) } else { None };
        
        log::info!("🔍 INSPECTING: Surface texture acquisition");
        
        match operation() {
            Ok(result) => {
                let duration = start.map(|s| s.elapsed());
                self.log_operation(
                    RenderOperationType::SurfaceAcquisition,
                    "Successfully acquired surface texture".to_string(),
                    true,
                    None,
                    duration,
                );
                log::info!("✅ Surface texture acquired successfully in {:?}", duration);
                Ok(result)
            }
            Err(e) => {
                let duration = start.map(|s| s.elapsed());
                let error_msg = format!("{:?}", e);
                self.log_operation(
                    RenderOperationType::SurfaceAcquisition,
                    "Failed to acquire surface texture".to_string(),
                    false,
                    Some(error_msg.clone()),
                    duration,
                );
                log::error!("❌ Surface texture acquisition failed: {} in {:?}", error_msg, duration);
                Err(e)
            }
        }
    }

    /// Inspect render pass creation and execution
    pub fn inspect_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        descriptor: &RenderPassDescriptor,
    ) -> InspectedRenderPass<'a> {
        if !self.enable_detailed_logging {
            let render_pass = encoder.begin_render_pass(descriptor);
            return InspectedRenderPass::new(render_pass, None);
        }

        let start = if self.enable_timing { Some(Instant::now()) } else { None };
        
        log::info!("🔍 INSPECTING: Render pass creation");
        log::info!("  Color attachments: {}", descriptor.color_attachments.len());
        
        for (i, attachment) in descriptor.color_attachments.iter().enumerate() {
            if let Some(att) = attachment {
                log::info!("  Attachment {}: {:?} load, {:?} store", i, att.ops.load, att.ops.store);
                match att.ops.load {
                    LoadOp::Clear(color) => {
                        log::info!("    Clear color: [{:.2}, {:.2}, {:.2}, {:.2}]", 
                            color.r, color.g, color.b, color.a);
                    }
                    LoadOp::Load => {
                        log::info!("    Loading existing contents");
                    }
                    _ => {
                        log::info!("    Other load operation");
                    }
                }
            }
        }

        let render_pass = encoder.begin_render_pass(descriptor);
        let duration = start.map(|s| s.elapsed());

        self.log_operation(
            RenderOperationType::RenderPassBegin,
            "Render pass began successfully".to_string(),
            true,
            None,
            duration,
        );

        log::info!("✅ Render pass created in {:?}", duration);

        InspectedRenderPass::new(render_pass, Some(self))
    }

    /// Inspect command buffer submission
    pub fn inspect_command_submission<F>(&self, queue: &Queue, operation: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<wgpu::CommandBuffer, String>,
    {
        if !self.enable_detailed_logging {
            return match operation() {
                Ok(cmd_buffer) => {
                    queue.submit(std::iter::once(cmd_buffer));
                    Ok(())
                }
                Err(e) => Err(e),
            };
        }

        let start = if self.enable_timing { Some(Instant::now()) } else { None };
        
        log::info!("🔍 INSPECTING: Command buffer submission");

        match operation() {
            Ok(cmd_buffer) => {
                log::info!("  Command buffer created successfully");
                
                let submit_start = if self.enable_timing { Some(Instant::now()) } else { None };
                queue.submit(std::iter::once(cmd_buffer));
                let submit_duration = submit_start.map(|s| s.elapsed());

                let total_duration = start.map(|s| s.elapsed());

                self.log_operation(
                    RenderOperationType::CommandSubmission,
                    "Command buffer submitted successfully".to_string(),
                    true,
                    None,
                    total_duration,
                );

                log::info!("✅ Command buffer submitted in {:?} (submit: {:?})", total_duration, submit_duration);
                Ok(())
            }
            Err(e) => {
                let duration = start.map(|s| s.elapsed());
                self.log_operation(
                    RenderOperationType::CommandSubmission,
                    "Command buffer creation/submission failed".to_string(),
                    false,
                    Some(e.clone()),
                    duration,
                );
                log::error!("❌ Command buffer submission failed: {} in {:?}", e, duration);
                Err(e)
            }
        }
    }

    /// Inspect presentation
    pub fn inspect_presentation<F>(&self, operation: F)
    where
        F: FnOnce(),
    {
        if !self.enable_detailed_logging {
            operation();
            return;
        }

        let start = if self.enable_timing { Some(Instant::now()) } else { None };
        
        log::info!("🔍 INSPECTING: Frame presentation");

        operation();

        let duration = start.map(|s| s.elapsed());

        self.log_operation(
            RenderOperationType::Presentation,
            "Frame presented successfully".to_string(),
            true,
            None,
            duration,
        );

        log::info!("✅ Frame presented in {:?}", duration);
    }

    /// Generate a comprehensive inspection report
    pub fn generate_report(&self) -> String {
        let operations = self.operations.lock().unwrap();
        
        let mut report = String::new();
        report.push_str("🔍 RENDER PIPELINE INSPECTION REPORT\n");
        report.push_str("=====================================\n\n");

        if operations.is_empty() {
            report.push_str("No render operations logged.\n");
            return report;
        }

        // Summary statistics
        let total_operations = operations.len();
        let successful_operations = operations.iter().filter(|op| op.success).count();
        let failed_operations = total_operations - successful_operations;

        report.push_str(&format!("Total Operations: {}\n", total_operations));
        report.push_str(&format!("Successful: {}\n", successful_operations));
        report.push_str(&format!("Failed: {}\n", failed_operations));
        report.push_str(&format!("Success Rate: {:.1}%\n\n", 
            (successful_operations as f64 / total_operations as f64) * 100.0));

        // Recent operations (last 20)
        report.push_str("Recent Operations (last 20):\n");
        for op in operations.iter().rev().take(20) {
            let status = if op.success { "✅" } else { "❌" };
            let time_str = format!("{:?}", op.timestamp.elapsed()).replace("{", "").replace("}", "");
            
            report.push_str(&format!("  {} [{} ago] {:?}: {}\n", 
                status, time_str, op.operation_type, op.details));
            
            if let Some(error) = &op.error {
                report.push_str(&format!("    Error: {}\n", error));
            }
            
            if let Some(duration) = op.duration {
                report.push_str(&format!("    Duration: {:?}\n", duration));
            }
        }

        report
    }

    /// Clear operation history
    pub fn clear_history(&self) {
        let mut operations = self.operations.lock().unwrap();
        operations.clear();
        log::info!("🧹 Render pipeline inspector history cleared");
    }
}

/// Inspected render pass that logs all operations
pub struct InspectedRenderPass<'a> {
    render_pass: RenderPass<'a>,
    inspector: Option<&'a RenderPipelineInspector>,
}

impl<'a> InspectedRenderPass<'a> {
    fn new(render_pass: RenderPass<'a>, inspector: Option<&'a RenderPipelineInspector>) -> Self {
        if let Some(inspector) = inspector {
            inspector.log_operation(
                RenderOperationType::RenderPassBegin,
                "Render pass began".to_string(),
                true,
                None,
                None,
            );
        }
        
        Self { render_pass, inspector }
    }

    /// Set pipeline with inspection
    pub fn set_pipeline(&mut self, pipeline: &RenderPipeline) {
        if let Some(inspector) = self.inspector {
            log::info!("🔍 RenderPass: Setting pipeline");
            inspector.log_operation(
                RenderOperationType::PipelineBind,
                "Pipeline bound".to_string(),
                true,
                None,
                None,
            );
        }
        
        self.render_pass.set_pipeline(pipeline);
    }

    /// Set vertex buffer with inspection
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &Buffer) {
        if let Some(inspector) = self.inspector {
            log::info!("🔍 RenderPass: Setting vertex buffer at slot {}", slot);
            inspector.log_operation(
                RenderOperationType::BufferBind,
                format!("Vertex buffer bound at slot {}", slot),
                true,
                None,
                None,
            );
        }
        
        self.render_pass.set_vertex_buffer(slot, buffer.slice(..));
    }

    /// Draw with inspection
    pub fn draw_indexed(&mut self, indices: std::ops::Range<u32>, base_vertex: i32, instances: std::ops::Range<u32>) {
        if let Some(inspector) = self.inspector {
            log::info!("🔍 RenderPass: Drawing indexed {} indices, {} instances (base_vertex: {})", 
                indices.end - indices.start, instances.end - instances.start, base_vertex);
            inspector.log_operation(
                RenderOperationType::DrawCall,
                format!("Draw indexed: {} indices, {} instances (base_vertex: {})", 
                    indices.end - indices.start, instances.end - instances.start, base_vertex),
                true,
                None,
                None,
            );
        }
        
        self.render_pass.draw_indexed(indices, base_vertex, instances);
    }

    /// Set viewport with inspection
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) {
        if let Some(_inspector) = self.inspector {
            log::info!("🔍 RenderPass: Setting viewport [{}, {} {}x{}] depth=[{}, {}]", 
                x, y, width, height, min_depth, max_depth);
        }
        
        self.render_pass.set_viewport(x, y, width, height, min_depth, max_depth);
    }

    /// Set bind group with inspection
    pub fn set_bind_group(&mut self, index: u32, bind_group: &BindGroup) {
        if let Some(_inspector) = self.inspector {
            log::info!("🔍 RenderPass: Setting bind group at index {}", index);
            _inspector.log_operation(
                RenderOperationType::TextureBind,
                format!("Bind group bound at index {}", index),
                true,
                None,
                None,
            );
        }
        
        self.render_pass.set_bind_group(index, bind_group, &[]);
    }
}

impl<'a> Drop for InspectedRenderPass<'a> {
    fn drop(&mut self) {
        if let Some(inspector) = self.inspector {
            inspector.log_operation(
                RenderOperationType::RenderPassEnd,
                "Render pass ended".to_string(),
                true,
                None,
                None,
            );
            log::info!("🔍 RenderPass: Ending render pass");
        }
    }
}