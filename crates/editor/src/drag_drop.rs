//! Cross-panel drag-and-drop state for the editor.
//!
//! One drag can be in flight at a time. Sources arm a payload on mouse press;
//! the state machine promotes it to a drag once the mouse travels
//! [`DRAG_THRESHOLD`] pixels, and to a one-frame `Dropped` state on release.
//! Drop targets consume the drop geometrically via [`DragDropState::take_drop_in`]
//! — first target to claim the position wins; unclaimed drops expire after
//! one frame.

use glam::Vec2;

/// Pixels the mouse must travel from the press point before an armed
/// payload becomes a drag (below this, release is a plain click).
pub const DRAG_THRESHOLD: f32 = 4.0;

/// What is being dragged.
#[derive(Debug, Clone, PartialEq)]
pub enum DragPayload {
    /// A texture from the asset browser. `path` is the asset-relative path
    /// (used for labels and status messages); `handle` is the loaded
    /// renderer texture id.
    Texture { handle: u32, path: String },
}

#[derive(Debug, Default)]
enum DragState {
    #[default]
    Idle,
    /// Mouse pressed on a source; not yet moved past the threshold.
    Armed { payload: DragPayload, press_pos: Vec2 },
    /// Actively dragging.
    Dragging { payload: DragPayload },
    /// Released this frame at `pos`; consumable by a target for ONE frame.
    Dropped { payload: DragPayload, pos: Vec2 },
}

/// Editor-wide drag-and-drop coordinator (a field on `EditorContext`).
#[derive(Debug, Default)]
pub struct DragDropState {
    state: DragState,
}

impl DragDropState {
    /// Create a new idle drag-drop state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the state machine. Call ONCE per frame, before panels render.
    pub fn begin_frame(&mut self, mouse_pos: Vec2, mouse_down: bool, mouse_just_released: bool) {
        self.state = match std::mem::take(&mut self.state) {
            DragState::Idle => DragState::Idle,
            DragState::Armed { payload, press_pos } => {
                if mouse_just_released || !mouse_down {
                    // Released before the threshold: a plain click, not a drag
                    DragState::Idle
                } else if (mouse_pos - press_pos).length() >= DRAG_THRESHOLD {
                    DragState::Dragging { payload }
                } else {
                    DragState::Armed { payload, press_pos }
                }
            }
            DragState::Dragging { payload } => {
                if mouse_just_released {
                    DragState::Dropped { payload, pos: mouse_pos }
                } else if !mouse_down {
                    // Release happened while we weren't looking (e.g. focus
                    // loss) — treat like a drop at the current position
                    DragState::Dropped { payload, pos: mouse_pos }
                } else {
                    DragState::Dragging { payload }
                }
            }
            // A drop no target consumed last frame expires
            DragState::Dropped { .. } => DragState::Idle,
        };
    }

    /// Arm a drag from a source widget on mouse press. Ignored unless idle.
    pub fn arm(&mut self, payload: DragPayload, press_pos: Vec2) {
        if matches!(self.state, DragState::Idle) {
            self.state = DragState::Armed { payload, press_pos };
        }
    }

    /// The payload currently being dragged (for ghost rendering and
    /// drop-target hover highlights). `None` unless in the Dragging state.
    pub fn dragging_payload(&self) -> Option<&DragPayload> {
        match &self.state {
            DragState::Dragging { payload } => Some(payload),
            _ => None,
        }
    }

    /// Consume the drop if it landed inside `bounds`. First caller wins.
    pub fn take_drop_in(&mut self, bounds: common::Rect) -> Option<(DragPayload, Vec2)> {
        if let DragState::Dropped { pos, .. } = &self.state {
            if bounds.contains(*pos) {
                if let DragState::Dropped { payload, pos } = std::mem::take(&mut self.state) {
                    return Some((payload, pos));
                }
            }
        }
        None
    }

    /// Whether click handlers should ignore this frame's click: true while
    /// dragging and on the release frame, so a drop never doubles as a
    /// click-select underneath the cursor.
    pub fn suppresses_click(&self) -> bool {
        matches!(self.state, DragState::Dragging { .. } | DragState::Dropped { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texture_payload() -> DragPayload {
        DragPayload::Texture { handle: 7, path: "player.png".into() }
    }

    #[test]
    fn test_release_under_threshold_is_a_click_not_a_drag() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        // Mouse released 2px away — below the threshold
        dd.begin_frame(Vec2::new(102.0, 100.0), false, true);
        assert!(dd.dragging_payload().is_none());
        assert!(!dd.suppresses_click(), "a plain click must not be suppressed");
        assert!(dd.take_drop_in(common::Rect::new(0.0, 0.0, 500.0, 500.0)).is_none());
    }

    #[test]
    fn test_moving_past_threshold_starts_drag() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        dd.begin_frame(Vec2::new(106.0, 100.0), true, false);
        assert_eq!(dd.dragging_payload(), Some(&texture_payload()));
        assert!(dd.suppresses_click());
    }

    #[test]
    fn test_release_while_dragging_drops_at_position() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        dd.begin_frame(Vec2::new(150.0, 150.0), true, false);
        dd.begin_frame(Vec2::new(300.0, 200.0), false, true);

        assert!(dd.suppresses_click(), "the release frame still suppresses clicks");
        let (payload, pos) = dd
            .take_drop_in(common::Rect::new(250.0, 150.0, 200.0, 200.0))
            .expect("drop inside bounds is consumable");
        assert_eq!(payload, texture_payload());
        assert_eq!(pos, Vec2::new(300.0, 200.0));
        // Consumed — a second target gets nothing
        assert!(dd.take_drop_in(common::Rect::new(0.0, 0.0, 1000.0, 1000.0)).is_none());
    }

    #[test]
    fn test_drop_outside_target_bounds_is_left_for_others() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        dd.begin_frame(Vec2::new(150.0, 150.0), true, false);
        dd.begin_frame(Vec2::new(300.0, 200.0), false, true);

        // A target whose bounds don't contain the drop leaves it in place
        assert!(dd.take_drop_in(common::Rect::new(0.0, 0.0, 50.0, 50.0)).is_none());
        // The right target can still take it this frame
        assert!(dd.take_drop_in(common::Rect::new(290.0, 190.0, 20.0, 20.0)).is_some());
    }

    #[test]
    fn test_unconsumed_drop_expires_after_one_frame() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        dd.begin_frame(Vec2::new(150.0, 150.0), true, false);
        dd.begin_frame(Vec2::new(300.0, 200.0), false, true);
        assert!(dd.suppresses_click());

        // Next frame: nobody consumed it — it expires
        dd.begin_frame(Vec2::new(300.0, 200.0), false, false);
        assert!(!dd.suppresses_click());
        assert!(dd.take_drop_in(common::Rect::new(0.0, 0.0, 1000.0, 1000.0)).is_none());
    }

    #[test]
    fn test_arm_ignored_while_drag_in_flight() {
        let mut dd = DragDropState::new();
        dd.arm(texture_payload(), Vec2::new(100.0, 100.0));
        dd.begin_frame(Vec2::new(150.0, 150.0), true, false);

        let other = DragPayload::Texture { handle: 99, path: "other.png".into() };
        dd.arm(other, Vec2::new(0.0, 0.0));
        assert_eq!(dd.dragging_payload(), Some(&texture_payload()), "in-flight drag wins");
    }
}
