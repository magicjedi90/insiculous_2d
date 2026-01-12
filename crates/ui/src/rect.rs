//! Rectangle type for UI layout and hit detection.
//!
//! Re-exports `common::Rect` for consistency across the engine.

// Re-export Rect from common crate
pub use common::Rect;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_centered() {
        let rect = Rect::centered(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));
        assert_eq!(rect.x, 75.0);
        assert_eq!(rect.y, 85.0);
        assert_eq!(rect.width, 50.0);
        assert_eq!(rect.height, 30.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        assert!(rect.contains(Vec2::new(50.0, 50.0)));
        assert!(!rect.contains(Vec2::new(5.0, 50.0)));
    }

    #[test]
    fn test_rect_offset() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let offset = rect.offset(Vec2::new(5.0, -5.0));
        assert_eq!(offset.x, 15.0);
        assert_eq!(offset.y, 15.0);
    }
}
