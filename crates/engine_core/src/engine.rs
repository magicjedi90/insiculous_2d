// crates/engine_core/src/engine.rs

use std::rc::Rc;
use std::marker::PhantomData;
use anyhow::Result;
use log::{error, info};
use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use egui::Context as EguiContext;
use egui_wgpu::Renderer as EguiRenderer;

use crate::game_state::GameState;
use crate::state_stack::StateStack;
use crate::time::ApplicationClock;
use renderer::Renderer;  // your GPU renderer

/// Public entry point: start the winit loop with your root state.
pub fn launch<Root: GameState + Default>() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = EngineApplication::<Root>::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct EngineApplication<Root: GameState + Default> {
    stack:      StateStack,
    clock:      ApplicationClock,
    window:     Option<Window>,
    main_id:    Option<WindowId>,
    renderer:   Option<Rc<Renderer>>,
    egui_ctx:   EguiContext,
    egui_rpass: Option<Arc<Mutex<EguiRenderer>>>,
    _phantom:   PhantomData<Root>,
}

impl<Root: GameState + Default> Default for EngineApplication<Root> {
    fn default() -> Self {
        Self {
            stack:      StateStack::new(Box::new(Root::default())),
            clock:      ApplicationClock::new(),
            window:     None,
            main_id:    None,
            renderer:   None,
            egui_ctx:   EguiContext::default(),
            egui_rpass: None,
            _phantom:   PhantomData,
        }
    }
}

impl<Root: GameState + Default> ApplicationHandler for EngineApplication<Root> {
    fn new_events(&mut self, _loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            // avoid huge dt on first frame
            self.clock.advance_frame();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 1. Create the OS window
        let attrs = Window::default_attributes().with_title("Insiculous2D");
        match event_loop.create_window(attrs) {
            Ok(win) => {
                info!("Window created, id={:?}", win.id());
                self.main_id = Some(win.id());

                // 2. Initialize your Renderer once
                let renderer = Rc::new(pollster::block_on(Renderer::new(&win)));
                let fmt    = renderer.target_format();

                // 3. Now that you have a real Device+Format, create EguiRenderer
                let egui_render_pass = Arc::new(Mutex::new(EguiRenderer::new(
                    renderer.device(),    // &wgpu::Device
                    fmt,                  // wgpu::TextureFormat
                    None,                 // optional MSAA samples
                    1,                    // max textures per frame
                    true,                 // auto-submit render pass
                )));

                // Set handles with the new initialization structure
                StateStack::set_gui_handles(self.egui_ctx.clone(), egui_render_pass.clone());

                self.renderer   = Some(renderer.clone());
                self.egui_rpass = Some(egui_render_pass);
                self.window     = Some(win);

                // 4. Start polling so we redraw continuously
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Err(e) => {
                error!("Failed to create window: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id:          WindowId,
        event:       WindowEvent,
    ) {
        if Some(id) != self.main_id {
            return; // ignore
        }

        // 7. Forward input to the top state
        let wrapped = Event::WindowEvent { window_id: id, event: event.clone() };
        let transition = self.stack.active_mut().handle_winit_event(&wrapped);
        self.stack.apply(transition);

        match event {
            WindowEvent::RedrawRequested => {
                if let (Some(win), Some(renderer), Some(_)) =
                    (&self.window, &self.renderer, &self.egui_rpass)
                {
                    // 8. Render game states + egui overlays
                    self.stack.render(win, renderer.clone());
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let (Some(win), Some(_)) = (&self.window, &self.renderer) {
            // 5. Update your game logic
            self.clock.advance_frame();
            self.stack.update(self.clock.delta_seconds());

            // 6. Request a redraw and keep polling
            win.request_redraw();
            event_loop.set_control_flow(ControlFlow::Poll);
        } else {
            // no window yet -> wait for events
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}