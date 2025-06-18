/// component.rs
pub trait Updatable {
    /// Called every frame with the time since the last tick.
    fn update(&mut self, delta_time: f32);
}
