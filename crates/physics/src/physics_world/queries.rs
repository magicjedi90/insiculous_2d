//! Spatial queries (raycasts).

use glam::Vec2;
use rapier2d::prelude::*;

use ecs::EntityId;

use super::PhysicsWorld;

impl PhysicsWorld {
    /// Cast a ray and return the first hit as `(entity, hit_point, distance)`.
    ///
    /// `direction` does not need to be normalized — it is normalized
    /// internally, so `max_distance` and the returned distance are always in
    /// pixels along the ray regardless of the direction vector's length.
    /// Returns `None` for a zero-length or non-finite direction.
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_distance: f32) -> Option<(EntityId, Vec2, f32)> {
        let dir = direction.try_normalize()?;
        let origin_m = self.pixels_to_meters(origin);
        let ray = Ray::new(
            point![origin_m.x, origin_m.y],
            vector![dir.x, dir.y],
        );
        let max_toi = self.pixels_to_meters_scalar(max_distance);

        if let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            true,
            QueryFilter::default(),
        ) {
            if let Some(&entity) = self.collider_to_entity.get(&handle) {
                let hit_point = ray.point_at(toi);
                let hit_meters = Vec2::new(hit_point.x, hit_point.y);
                return Some((
                    entity,
                    self.meters_to_pixels(hit_meters),
                    self.meters_to_pixels_scalar(toi),
                ));
            }
        }

        None
    }
}
