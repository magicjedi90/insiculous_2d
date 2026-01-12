//! Rectangle type for 2D bounds and UI layout.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Axis-aligned rectangle defined by position and size.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    /// Top-left position
    pub x: f32,
    pub y: f32,
    /// Dimensions
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle from position and size.
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle from position and size vectors.
    #[inline]
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            width: size.x,
            height: size.y,
        }
    }

    /// Create a rectangle from min and max corners.
    #[inline]
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }

    /// Create a rectangle centered at a position.
    #[inline]
    pub fn centered(center: Vec2, size: Vec2) -> Self {
        Self {
            x: center.x - size.x * 0.5,
            y: center.y - size.y * 0.5,
            width: size.x,
            height: size.y,
        }
    }

    /// Get the position (top-left corner).
    #[inline]
    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get the size.
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Get the center point.
    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// Get the minimum corner (top-left).
    #[inline]
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get the maximum corner (bottom-right).
    #[inline]
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Get left edge X coordinate.
    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Get right edge X coordinate.
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get top edge Y coordinate.
    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Get bottom edge Y coordinate.
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Check if a point is inside the rectangle.
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Check if this rectangle intersects another.
    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Get the intersection of two rectangles, if any.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x1 < x2 && y1 < y2 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Get the bounding box containing both rectangles.
    #[inline]
    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Expand the rectangle by the given amount on all sides.
    #[inline]
    pub fn expand(&self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + amount * 2.0,
            self.height + amount * 2.0,
        )
    }

    /// Shrink the rectangle by the given amount on all sides.
    #[inline]
    pub fn shrink(&self, amount: f32) -> Rect {
        self.expand(-amount)
    }

    /// Translate the rectangle by the given offset.
    #[inline]
    pub fn translate(&self, offset: Vec2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.width, self.height)
    }

    /// Alias for translate (compatibility with UI rect).
    #[inline]
    pub fn offset(&self, delta: Vec2) -> Rect {
        self.translate(delta)
    }

    /// Get the area of the rectangle.
    #[inline]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_new() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.width, 100.0);
    }

    #[test]
    fn test_rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(r.center(), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Vec2::new(50.0, 50.0)));
        assert!(!r.contains(Vec2::new(150.0, 50.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let c = Rect::new(200.0, 200.0, 100.0, 100.0);

        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);

        let i = a.intersection(&b).unwrap();
        assert_eq!(i.x, 50.0);
        assert_eq!(i.y, 50.0);
        assert_eq!(i.width, 50.0);
        assert_eq!(i.height, 50.0);
    }

    #[test]
    fn test_rect_expand() {
        let r = Rect::new(10.0, 10.0, 80.0, 80.0);
        let expanded = r.expand(10.0);
        assert_eq!(expanded.x, 0.0);
        assert_eq!(expanded.width, 100.0);
    }
}
