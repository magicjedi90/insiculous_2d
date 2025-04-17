pub struct Renderer {
    // fields: device, queue, surface, etc.
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Self {
        // TODO: request adapter/device, configure surface
        todo!()
    }
    pub fn clear(&mut self) {
        // TODO: begin frame, clear, submit
    }
}
