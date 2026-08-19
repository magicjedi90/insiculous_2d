//! Window management for the renderer.

pub(crate) use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::error::RendererError;

/// Configuration for the window
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Title of the window
    pub title: String,
    /// Width of the window
    pub width: u32,
    /// Height of the window
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "insiculous_2d v0.1".to_string(),
            width: 800,
            height: 600,
            resizable: true,
        }
    }
}

/// Create a new window with the given configuration using an ActiveEventLoop
///
/// On the web, winit creates a canvas but does NOT insert it into the DOM
/// (a silent blank page). We insert it ourselves: swapped in place of the
/// page's `#game-canvas` placeholder when present (the site-embed contract,
/// id and a11y attributes carried over), else appended to `<body>`. Letting
/// winit own the canvas — rather than adopting an existing element via
/// `with_canvas` — is the path the H1 spike verified end-to-end.
pub fn create_window_with_active_loop(
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
) -> Result<std::sync::Arc<Window>, RendererError> {
    // Create window attributes
    let mut attributes = WindowAttributes::default();
    attributes.title = config.title.clone();
    attributes.inner_size = Some(PhysicalSize::new(config.width, config.height).into());
    attributes.resizable = config.resizable;

    // Create the window using ActiveEventLoop's create_window method
    let window = event_loop
        .create_window(attributes)
        .map_err(|e| RendererError::WindowCreationError(e.to_string()))?;

    #[cfg(target_arch = "wasm32")]
    insert_canvas_into_dom(&window);

    // Wrap the window in an Arc to ensure it outlives the event loop callback
    Ok(std::sync::Arc::new(window))
}

/// Put winit's canvas into the page: replace `#game-canvas` if the page has
/// such a placeholder (keeping its id and accessibility attributes), else
/// append to `<body>`. Focuses the canvas so keyboard input works at once.
///
/// MUST be called for every winit window created on the web, whatever code
/// creates it — winit never inserts its canvas into the DOM, and a detached
/// canvas renders silently into nothing (every pass valid, page black).
#[cfg(target_arch = "wasm32")]
pub fn insert_canvas_into_dom(window: &Window) {
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowExtWebSys;

    let Some(canvas) = window.canvas() else {
        log::error!("winit window has no canvas on web");
        return;
    };
    // Idempotent: if this window's canvas is already in the DOM (a second
    // caller — e.g. both WindowManager and a direct renderer user — or a
    // repeated `resumed`), there is nothing to do.
    if canvas.is_connected() {
        return;
    }
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };

    if let Some(placeholder) = document.get_element_by_id("game-canvas") {
        // width/height carry the embed's intended pixel size — without them
        // the canvas starts at the browser default (300x150) until winit's
        // own sizing kicks in.
        for attr in ["width", "height", "tabindex", "role", "aria-label"] {
            if let Some(value) = placeholder.get_attribute(attr) {
                let _ = canvas.set_attribute(attr, &value);
            }
        }
        if placeholder.replace_with_with_node_1(canvas.unchecked_ref()).is_ok() {
            canvas.set_id("game-canvas");
        } else {
            log::error!("could not replace #game-canvas placeholder");
        }
    } else if let Some(body) = document.body() {
        log::warn!("no #game-canvas placeholder; appending canvas to <body>");
        let _ = canvas.set_attribute("tabindex", "0");
        canvas.set_id("game-canvas");
        let _ = body.append_child(&canvas);
    }

    // Keyboard input needs focus; do it here because the page's own
    // focus-by-id script may have run before this canvas existed.
    let _ = canvas.focus();
}
