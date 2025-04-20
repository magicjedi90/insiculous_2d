// crates/builtin_states/src/pause_state.rs
use engine_core::prelude::*;
use egui::{CentralPanel, Layout, Direction};
use egui_wgpu::ScreenDescriptor;
use once_cell::unsync::OnceCell;
use std::rc::Rc;
use winit::{event::Event, window::Window};

// Pull in your WGPU renderer and egui handles
use renderer::Renderer;
use input::{PlayerCommand, event_mapper::command_from_event};

pub struct PauseState {
    // Cache the translucent panel style once
    overlay_style: OnceCell<egui::Frame>,
}

impl Default for PauseState {
    fn default() -> Self {
        Self { overlay_style: OnceCell::new() }
    }
}

impl GameState for PauseState {
    fn update(&mut self, _dt: f32) {}

    fn render(&mut self, window: &Window, renderer: Rc<Renderer>) {
        // 1) Dark translucent clear (assuming clear_screen now takes &self instead of &mut self)
        renderer.clear_screen([0.0, 0.0, 0.0, 0.6]);

        // 2) Grab the shared egui context & renderer
        let (egui_ctx, egui_render_pass) = StateStack::egui_handles();

        // 3) Begin egui frame
        egui_ctx.begin_pass(Default::default());

        // 4) Build a centered panel
        let style = self.overlay_style.get_or_init(|| {
            egui::Frame::NONE.fill(egui::Color32::BLACK.to_opaque())
        });
        CentralPanel::default()
            .frame(style.clone())
            .show(&egui_ctx, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.heading("Paused");
                    ui.add_space(8.0);
                    ui.label("Press Esc to resume");
                });
            });

        // 5) End frame & tessellate
        let full_output = egui_ctx.end_pass();
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, 1.0);

        // 6) Prepare screen descriptor
        let [w, h] = [window.inner_size().width, window.inner_size().height];
        let screen_desc = ScreenDescriptor {
            size_in_pixels: [w, h],
            pixels_per_point: egui_ctx.pixels_per_point(),
        };

        // Get mutable access to egui renderer
        if let Ok(mut egui_renderer) = egui_render_pass.lock() {
            // 7) Update textures
            for (id, image_delta) in &full_output.textures_delta.set {
                egui_renderer.update_texture(
                    renderer.device(),
                    renderer.queue(),
                    *id,
                    image_delta
                );
            }

            // 8) Render egui
            renderer.render_egui(
                &mut egui_renderer, 
                &paint_jobs, 
                &screen_desc
            );

            // 9) Clean up old textures
            for id in &full_output.textures_delta.free {
                egui_renderer.free_texture(id);
            }
        }
    }
    
    fn handle_winit_event(&mut self, e: &Event<()>) -> Transition {
        if matches!(command_from_event(e), Some(PlayerCommand::TogglePause)) {
            Transition::Pop
        } else {
            Transition::None
        }
    }
}