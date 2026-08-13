//! 2D Transform type for position, rotation, and scale.

use glam::{Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 2D transformation component.
///
/// Represents position, rotation, and scale in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale factors
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    /// Create a new transform at the given position.
    #[inline]
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a transform from position, rotation, and scale.
    #[inline]
    pub fn from_parts(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }

    /// Set rotation (builder pattern).
    #[inline]
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set scale (builder pattern).
    #[inline]
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Set uniform scale (builder pattern).
    #[inline]
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec2::splat(scale);
        self
    }

    /// Get the 3x3 transformation matrix (T * R * S order).
    pub fn matrix(&self) -> Mat3 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        // Rotation matrix
        let rot = Mat3::from_cols_array(&[
            cos_r, sin_r, 0.0,
            -sin_r, cos_r, 0.0,
            0.0, 0.0, 1.0,
        ]);

        // Scale matrix
        let scale = Mat3::from_diagonal(Vec3::new(self.scale.x, self.scale.y, 1.0));

        // Translation matrix
        let translate = Mat3::from_cols_array(&[
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            self.position.x, self.position.y, 1.0,
        ]);

        // Combine: T * R * S
        translate * rot * scale
    }

    /// Get the inverse transformation matrix.
    #[inline]
    pub fn inverse_matrix(&self) -> Mat3 {
        self.matrix().inverse()
    }

    /// Transform a point by this transform.
    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        let transformed = self.matrix() * Vec3::new(point.x, point.y, 1.0);
        Vec2::new(transformed.x, transformed.y)
    }

    /// Transform a point by the inverse of this transform.
    #[inline]
    pub fn inverse_transform_point(&self, point: Vec2) -> Vec2 {
        let transformed = self.inverse_matrix() * Vec3::new(point.x, point.y, 1.0);
        Vec2::new(transformed.x, transformed.y)
    }

    /// Transform a direction vector (rotation and scale only, no translation).
    ///
    /// Applies scale in local axes then rotation (the linear part of
    /// [`matrix`](Self::matrix)'s `T * R * S`), so directions agree with
    /// point transforms under non-uniform scale.
    #[inline]
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let scaled = direction * self.scale;
        Vec2::new(
            scaled.x * cos_r - scaled.y * sin_r,
            scaled.x * sin_r + scaled.y * cos_r,
        )
    }

    /// Get the forward direction (positive X axis rotated).
    #[inline]
    pub fn forward(&self) -> Vec2 {
        Vec2::new(self.rotation.cos(), self.rotation.sin())
    }

    /// Get the right direction (positive Y axis rotated).
    #[inline]
    pub fn right(&self) -> Vec2 {
        Vec2::new(-self.rotation.sin(), self.rotation.cos())
    }

    /// Translate by the given offset.
    #[inline]
    pub fn translate(&mut self, offset: Vec2) {
        self.position += offset;
    }

    /// Rotate by the given angle in radians.
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Interpolate between two transforms.
    #[inline]
    pub fn lerp(self, other: Transform2D, t: f32) -> Transform2D {
        Transform2D {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_point() {
        let t = Transform2D::new(Vec2::new(100.0, 50.0));
        let point = t.transform_point(Vec2::ZERO);
        assert!((point.x - 100.0).abs() < 0.001);
        assert!((point.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_forward() {
        let t = Transform2D::default();
        let forward = t.forward();
        assert!((forward.x - 1.0).abs() < 0.001);
        assert!(forward.y.abs() < 0.001);
    }

    #[test]
    fn test_transform_lerp() {
        let a = Transform2D::new(Vec2::ZERO);
        let b = Transform2D::new(Vec2::new(100.0, 100.0));
        let mid = a.lerp(b, 0.5);
        assert!((mid.position.x - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_inverse_transform_point_round_trips_translated_rotated_point() {
        let t = Transform2D::new(Vec2::new(40.0, -15.0)).with_rotation(std::f32::consts::FRAC_PI_3);
        let local = Vec2::new(12.0, -7.0);
        let world = t.transform_point(local);
        let back = t.inverse_transform_point(world);
        assert!(
            (back - local).length() < 0.001,
            "expected {local:?}, got {back:?}"
        );
    }

    #[test]
    fn test_transform_direction_ignores_translation() {
        let rotated = Transform2D::new(Vec2::new(100.0, 50.0)).with_rotation(std::f32::consts::FRAC_PI_2);
        let origin = Transform2D::default().with_rotation(std::f32::consts::FRAC_PI_2);
        let direction = Vec2::new(8.0, 0.0);

        let from_translated = rotated.transform_direction(direction);
        let from_origin = origin.transform_direction(direction);

        assert!(
            (from_translated - from_origin).length() < 0.001,
            "translation must not affect directions: {from_translated:?} vs {from_origin:?}"
        );
        assert!(
            (from_translated - Vec2::new(0.0, 8.0)).length() < 0.001,
            "+X should rotate to +Y, got {from_translated:?}"
        );
    }

    #[test]
    fn test_matrix_applies_scale_before_rotation_before_translation() {
        // T * R * S with non-uniform scale: (1,0) scales to (2,0),
        // rotates 90° to (0,2), translates to (10,22). Guards the
        // composition order — a T*S*R swap would yield (10,23).
        let t = Transform2D::new(Vec2::new(10.0, 20.0))
            .with_rotation(std::f32::consts::FRAC_PI_2)
            .with_scale(Vec2::new(2.0, 3.0));

        let transformed = t.transform_point(Vec2::new(1.0, 0.0));
        assert!(
            (transformed - Vec2::new(10.0, 22.0)).length() < 0.001,
            "expected (10, 22), got {transformed:?}"
        );
    }

    #[test]
    fn test_transform_direction_agrees_with_matrix_under_nonuniform_scale() {
        // Directions must be the linear part of matrix(): the same
        // point-delta computed via transform_point.
        let t = Transform2D::new(Vec2::new(10.0, 20.0))
            .with_rotation(std::f32::consts::FRAC_PI_2)
            .with_scale(Vec2::new(2.0, 1.0));
        let direction = Vec2::new(1.0, 0.0);

        let via_points = t.transform_point(direction) - t.transform_point(Vec2::ZERO);
        let via_direction = t.transform_direction(direction);

        assert!(
            (via_direction - via_points).length() < 0.001,
            "direction {via_direction:?} disagrees with point delta {via_points:?}"
        );
        assert!(
            (via_direction - Vec2::new(0.0, 2.0)).length() < 0.001,
            "scale (2,1) then 90° rotation should map +X to (0,2), got {via_direction:?}"
        );
    }
}
