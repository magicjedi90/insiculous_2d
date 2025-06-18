// crates/engine_core/src/engine.rs

use anyhow::Result;
use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use components::World;
use input::InputMap;
use renderer::core::Renderer;

use crate::{
    time::ApplicationClock,
    state_stack::{StateStack, Transition},
    game_state::GameState,
};

/// Boot the engine with a root state (same public API as before).
pub fn launch<Root: GameState + Default>() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = EngineApplication::<Root>::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct EngineApplication<Root: GameState + Default> {
    stack:      StateStack,
    clock:      ApplicationClock,
    window:     Option<Window>,
    main_id:    Option<WindowId>,
    renderer:   Option<Rc<Renderer>>,
    world:      World,
    input_map:  InputMap,
    _ph:        std::marker::PhantomData<Root>,
}

impl<Root: GameState + Default> EngineApplication<Root> {
    fn new() -> Self {
        Self {
            stack: StateStack::new(Box::new(Root::default())),
            clock: ApplicationClock::new(),
            window: None,
            main_id: None,
            renderer: None,
            world: World::new(),
            input_map: InputMap::default(),
            _ph: std::marker::PhantomData,
        }
    }

    // ——— helper called from `resumed` ———
    fn init_window_and_renderer(&mut self, ev: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("Insiculous 2D");
        match ev.create_window(attrs) {
            Ok(win) => {
                self.main_id = Some(win.id());
                let renderer = Rc::new(pollster::block_on(Renderer::new(&win)));
                self.renderer = Some(renderer);
                self.window = Some(win);
            }
            Err(e) => log::error!("window creation failed: {e:?}"),
        }
    }

    fn tick(&mut self) {
        self.clock.advance_frame();
        let dt = self.clock.delta_seconds();
        self.stack.update(dt);
        // (If you want to run ECS systems directly, do it here.)
    }

    fn draw(&mut self) {
        if let (Some(win), Some(renderer)) = (&self.window, &self.renderer) {
            self.stack.render(win, renderer.clone());
        }
    }

    fn route_event(&mut self, event: &Event<()>) {
        // Map raw -> command   (currently **unused** by GameState, but you can
        // store it somewhere globally or roll your own event bus.)
        let _cmd = self.input_map.map_event(event);

        // Wrap into WindowEvent the same way the original engine did
        if let Event::WindowEvent { window_id: id, event: we } = event {
            if Some(*id) == self.main_id {
                let wrapped = Event::WindowEvent { window_id: *id, event: we.clone() };
                let t = self.stack.active_mut().handle_winit_event(&wrapped);
                self.stack.apply(t);
            }
        }
    }
}

// ————————————————————— winit ApplicationHandler impl ——————————————————— //

impl<Root: GameState + Default> ApplicationHandler for EngineApplication<Root> {
    fn new_events(&mut self, _ev: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            self.clock.advance_frame(); // zero-delta first frame
        }
    }

    fn resumed(&mut self, ev: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_window_and_renderer(ev);
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::RedrawRequested => self.draw(),
            // Route other window events to the state stack
            _ => self.route_event(&Event::WindowEvent { window_id, event }),
        }
    }

    fn about_to_wait(&mut self, ev: &ActiveEventLoop) {
        self.tick();
        if let Some(win) = &self.window {
            win.request_redraw();
            ev.set_control_flow(ControlFlow::Poll);
        }
    }
}
