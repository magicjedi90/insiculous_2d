use anyhow::Result;
use log::{error, info};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::time::ApplicationClock;

/// Trait implemented by the game to hook into the engine lifecycle.
pub trait GameState: Default {
    fn update(&mut self, delta_seconds: f32);
    fn render(&mut self, window: &Window);
}

/// Internal glue that lets the engine talk to winit’s trait system.
struct EngineApplication<RootState: GameState> {
    state:               RootState,
    clock:               ApplicationClock,
    primary_window:      Option<Window>,
    primary_window_id:   Option<WindowId>,
}

impl<RootState: GameState> Default for EngineApplication<RootState> {
    fn default() -> Self {
        Self {
            state: RootState::default(),
            clock: ApplicationClock::new(),
            primary_window: None,
            primary_window_id: None,
        }
    }
}

impl<RootState: GameState> ApplicationHandler for EngineApplication<RootState> {
    // Optional: react to NewEvents StartCause for fixed‑timestep work
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            self.clock.advance_frame();
        }
    }

    /// Called once the OS allows surface creation.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes().with_title("Insiculous2D");
        match event_loop.create_window(attributes) {
            Ok(window) => {
                self.primary_window_id = Some(window.id());
                self.primary_window = Some(window);
                info!("Window created");
            }
            Err(error) => {
                error!("Failed to create window: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.primary_window_id {
            return;
        }
        let window = match self.primary_window.as_ref() {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),         // new exit helper :contentReference[oaicite:5]{index=5}
            WindowEvent::RedrawRequested => self.state.render(window),
            _ => {}
        }
    }

    /// Per‑frame callback; good place to tick game logic.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.clock.advance_frame();
        if let Some(window) = self.primary_window.as_ref() {
            self.state.update(self.clock.delta_seconds());
            window.request_redraw();                                  // redraw request in about_to_wait :contentReference[oaicite:4]{index=4}
        }
    }
}

/// **Public entry point** the game calls.
pub fn launch<RootState: GameState>() -> Result<()> {
    let mut event_loop: EventLoop<()> = EventLoop::new()?;            // Result‑returning ctor :contentReference[oaicite:6]{index=6}
    let mut application = EngineApplication::<RootState>::default();

    // `run_app` is the stable loop since winit 0.30 :contentReference[oaicite:7]{index=7}
    event_loop.run_app(&mut application)?;

    Ok(())
}
