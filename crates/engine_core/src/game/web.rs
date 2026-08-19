//! Wasm-side renderer bring-up for `GameRunner`, split out of `game.rs`.
//!
//! wgpu adapter/device requests are genuinely async on WebGPU and the
//! browser main thread cannot block, so `resumed()` spawns the setup as a
//! browser task; the frame driver adopts the finished renderer on a later
//! frame via [`GameRunner::drain_pending_renderer`]. Until then
//! `update_and_render` early-returns on its existing "no asset manager yet"
//! guard while the redraw chain keeps polling.

use std::cell::RefCell;
use std::rc::Rc;

use renderer::{Renderer, RendererError};

use super::{Game, GameRunner};

/// Shared slot the spawned init task fills and the frame loop drains.
pub(super) type PendingRenderer = Rc<RefCell<Option<Result<Renderer, RendererError>>>>;

/// The page canvas's pixel size (its `width`/`height` attributes), the
/// authoritative render size for an embedded game.
fn canvas_pixel_size() -> Option<(u32, u32)> {
    use wasm_bindgen::JsCast;
    let canvas = web_sys::window()?
        .document()?
        .get_element_by_id("game-canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()?;
    Some((canvas.width(), canvas.height()))
}

impl<G: Game> GameRunner<G> {
    /// Kick off async renderer creation as a browser task. Called from
    /// `resumed()`; re-fire is prevented by its window-created guard.
    pub(super) fn spawn_renderer_init(&mut self) {
        let Some(window) = self.window_manager.window_clone() else {
            log::error!("spawn_renderer_init: no window to render to");
            return;
        };
        let pending = Rc::clone(&self.pending_renderer);
        let renderer_config = renderer::RendererConfig { vsync: self.config.vsync };
        wasm_bindgen_futures::spawn_local(async move {
            let result = renderer::init_with_config(window.clone(), renderer_config).await;
            *pending.borrow_mut() = Some(result);
            // Wake the frame loop so the drain below runs promptly.
            window.request_redraw();
        });
    }

    /// Adopt a finished renderer (or surface its error). Called at the top
    /// of every frame until the render manager is initialized.
    pub(super) fn drain_pending_renderer(&mut self) {
        if self.render_manager.is_initialized() {
            return;
        }
        let taken = self.pending_renderer.borrow_mut().take();
        match taken {
            Some(Ok(renderer)) => {
                self.render_manager.complete_init(renderer, self.config.clear_color);
                self.finish_renderer_setup();
                // The adopted canvas can report a 0 inner size during async
                // init and winit may never deliver a Resized for it — push
                // the canvas's actual pixel size through the resize path.
                if let Some((width, height)) = canvas_pixel_size() {
                    if width > 0 && height > 0 {
                        self.window_manager.resize(width, height);
                        self.render_manager.resize(width, height);
                    }
                }
                // The game draws its own frames from here on.
                crate::web::set_boot_status("");
            }
            Some(Err(e)) => {
                log::error!("Renderer init failed: {e}");
                crate::web::set_boot_status(&format!("Renderer init failed: {e}"));
            }
            None => {}
        }
    }
}
