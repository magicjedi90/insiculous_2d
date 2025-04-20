use std::rc::Rc;
use std::sync::Arc;
use crate::game_state::GameState;
use winit::window::Window;
use renderer::Renderer;
// For state_stack.rs
// Use interior mutability for the egui renderer
use std::sync::Mutex;
use std::sync::OnceLock;

/// What the active state wants the stack to do after `update()` or input.
pub enum Transition {
    None,
    Push(Box<dyn GameState>),
    Pop,
    Switch(Box<dyn GameState>),
}

/// Simple push‑down automaton (like Bevy's "StateStack") :contentReference[oaicite:2]{index=2}
pub struct StateStack {
    layers: Vec<Box<dyn GameState>>,
}

impl StateStack {
    pub fn new(root: Box<dyn GameState>) -> Self { Self { layers: vec![root] } }

    pub fn update(&mut self, dt: f32) {
        if let Some(top) = self.layers.last_mut() {
            top.update(dt);
        }
    }
    pub fn render(&mut self, window: &Window, renderer: Rc<Renderer>) {
        for state in &mut self.layers {
            state.render(window, renderer.clone());   // ✔ satisfies new trait sig
        }
    }
    pub fn apply(&mut self, trans: Transition) {
        match trans {
            Transition::None          => {}
            Transition::Push(s)       => self.layers.push(s),
            Transition::Pop           => { self.layers.pop(); }
            Transition::Switch(s)     => { self.layers.pop(); self.layers.push(s); }
        }
    }
    pub fn active_mut(&mut self) -> &mut dyn GameState {
        self.layers.last_mut().expect("Stack is never empty").as_mut()
    }
    
    pub fn set_gui_handles(ctx: egui::Context, render_pass: Arc<Mutex<egui_wgpu::Renderer>>) {
        GUI_HANDLES.get_or_init(|| (
            Arc::new(ctx), 
            render_pass
        ));
    }
    
    pub fn egui_handles() -> (Arc<egui::Context>, Arc<Mutex<egui_wgpu::Renderer>>) {
        GUI_HANDLES.get().expect("GUI handles not initialized").clone()
    }
}


static GUI_HANDLES: OnceLock<(Arc<egui::Context>, Arc<Mutex<egui_wgpu::Renderer>>)> = OnceLock::new();