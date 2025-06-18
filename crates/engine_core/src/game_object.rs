/// game_object.rs
use components::component::Updatable;

pub struct GameObject {
    /// A unique name or ID for debugging.
    pub name: String,
    /// All behaviors that need per-frame updates.
    components: Vec<Box<dyn Updatable>>,
}

impl GameObject {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            components: Vec::new(),
        }
    }

    /// Attach a new component
    pub fn add_component<C: Updatable + 'static>(&mut self, component: C) {
        self.components.push(Box::new(component));
    }

    /// Call update on each component
    pub fn update(&mut self, delta_time: f32) {
        for comp in &mut self.components {
            comp.update(delta_time);
        }
    }
}
