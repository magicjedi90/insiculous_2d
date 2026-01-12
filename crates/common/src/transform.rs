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
    #[inline]
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        Vec2::new(
            (direction.x * cos_r - direction.y * sin_r) * self.scale.x,
            (direction.x * sin_r + direction.y * cos_r) * self.scale.y,
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
    fn test_transform_default() {
        let t = Transform2D::default();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_builder() {
        let t = Transform2D::new(Vec2::new(10.0, 20.0))
            .with_rotation(1.5)
            .with_scale(Vec2::new(2.0, 3.0));

        assert_eq!(t.position, Vec2::new(10.0, 20.0));
        assert_eq!(t.rotation, 1.5);
        assert_eq!(t.scale, Vec2::new(2.0, 3.0));
    }

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
}
