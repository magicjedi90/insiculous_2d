// crates/engine_core/src/state_stack.rs
use crate::game_state::GameState;
use winit::window::Window;

/// What the active state wants the stack to do after `update()` or input.
pub enum Transition {
    None,
    Push(Box<dyn GameState>),
    Pop,
    Switch(Box<dyn GameState>),
}

/// Simple push‑down automaton (like Bevy’s “StateStack”) :contentReference[oaicite:2]{index=2}
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
    pub fn render(&mut self, window: &Window) {
        for state in &mut self.layers {
            state.render(window);
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
}
