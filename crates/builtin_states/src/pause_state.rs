use engine_core::prelude::*;
use input::{interpret_keyboard_input, PlayerCommand}; // your Command enum
use wgpu::Color;
use winit::{window::Window};
use winit::event::{Event, WindowEvent};

/// Temporary stand‑in for drawing helpers
fn draw_fullscreen_quad(_window: &Window, _c: Color) {}
fn draw_text_center(_window: &Window, _s: &str) {}

pub struct PauseState;

impl GameState for PauseState {
    fn update(&mut self, _dt: f32) {}

    fn render(&mut self, window: &Window) {
        draw_fullscreen_quad(
            window,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            },
        );
        draw_text_center(window, "Paused — Press Esc to resume");
    }
}

impl PauseState {
    pub fn handle_winit_event(&mut self, window_event: &Event<()>) -> Transition {
        if let Event::WindowEvent { event: WindowEvent::KeyboardInput { event, .. }, .. } = window_event {
            if let Some(PlayerCommand::TogglePause) = interpret_keyboard_input(&event) {
                return Transition::Pop;      // remove PauseState → gameplay resumes
            }
        }
        Transition::None
    }
}
