/// scene.rs
use crate::game_object::GameObject;

pub struct Scene {
    pub objects: Vec<GameObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
    }

    /// Called once per frame by your main loop
    pub fn update(&mut self, delta_time: f32) {
        for obj in &mut self.objects {
            obj.update(delta_time);
        }
    }
}
