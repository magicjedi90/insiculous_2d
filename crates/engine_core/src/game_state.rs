use std::rc::Rc;
use winit::event::Event;
use winit::window::Window;
use renderer::Renderer;
use crate::prelude::Transition;

/// Public trait every game state must implement.
///
/// *No* `Default` here – that would add `Self: Sized` and break dyn objects.
pub trait GameState: 'static {
    fn update(&mut self, delta_seconds: f32);
    fn render(&mut self, window: &Window, renderer: Rc<Renderer>);    /// Optional hook: handle raw winit input and return a state‐stack command.
    /// Default implementation does nothing, so it’s object‑safe.
    fn handle_winit_event(&mut self, _event: &Event<()>) -> Transition {
        Transition::None
    }
}
