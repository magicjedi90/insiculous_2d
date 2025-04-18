use std::marker::PhantomData;
use anyhow::Result;
use log::{error, info};

use winit::{
    application::ApplicationHandler,                           // trait for run_app :contentReference[oaicite:0]{index=0}
    event::{Event, StartCause, WindowEvent},                   // StartCause::Init :contentReference[oaicite:1]{index=1}
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},              // WindowAttributes builder :contentReference[oaicite:2]{index=2}
};

use crate::{
    game_state::GameState,
    state_stack::{StateStack},
    time::ApplicationClock,
};

/// Internal struct that owns the state‑stack and drives updates / rendering.
struct EngineApplication<RootState: GameState + Default> {
    stack:              StateStack,
    clock:              ApplicationClock,
    primary_window:     Option<Window>,
    primary_window_id:  Option<WindowId>,
    _phantom:           PhantomData<RootState>,

}

impl<RootState: GameState + Default> Default for EngineApplication<RootState> {
    fn default() -> Self {
        Self {
            stack: StateStack::new(Box::new(RootState::default())),
            clock: ApplicationClock::new(),
            primary_window: None,
            primary_window_id: None,
            _phantom: PhantomData,

        }
    }
}

impl<RootState: GameState + Default> ApplicationHandler for EngineApplication<RootState> {
    fn new_events(&mut self, _loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            self.clock.advance_frame();
        }
    }
    
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs: WindowAttributes = Window::default_attributes()
            .with_title("Insiculous2D");                // title helper :contentReference[oaicite:3]{index=3}

        match event_loop.create_window(attrs) {                  // create_window API :contentReference[oaicite:4]{index=4}
            Ok(window) => {
                info!("Window created, id={:?}", window.id());
                self.primary_window_id = Some(window.id());
                self.primary_window = Some(window);
            }
            Err(err) => {
                error!("Could not create window: {err}");
                event_loop.exit();                               // graceful shutdown :contentReference[oaicite:5]{index=5}
            }
        }
    }
    
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id:  WindowId,
        event:      WindowEvent,
    ) {
        // Discard events from non‑primary windows
        if Some(window_id) != self.primary_window_id {
            return;
        }

        // Give the active state a chance to react
        let transition = self
            .stack
            .active_mut()
            .handle_winit_event(&Event::WindowEvent { window_id, event: event.clone() });
        self.stack.apply(transition);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(w) = self.primary_window.as_ref() {
                    self.stack.render(w);
                }
            }
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.clock.advance_frame();
        if let Some(window) = self.primary_window.as_ref() {
            self.stack.update(self.clock.delta_seconds());
            window.request_redraw();
        }
    }
}

/// Public façade: create the event‑loop, spin the engine.
pub fn launch<RootState: GameState + Default>() -> Result<()> {
    let event_loop: EventLoop<()> = EventLoop::new()?;
    let mut app = EngineApplication::<RootState>::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}
