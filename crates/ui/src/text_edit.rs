//! Pure text-editing state: cursor, selection, and edit operations.
//!
//! [`TextEditState`] is the model behind text-input widgets. It knows nothing
//! about drawing or input devices — widgets translate key presses into calls
//! on this state, which keeps every editing rule headless-testable.
//!
//! Indices are `char` indices (not bytes) so the state stays correct even
//! though today's inputs are ASCII-numeric.

/// Editable text buffer with a cursor and an optional selection.
///
/// The selection spans `selection_anchor..cursor` (either order); typing with
/// a selection active replaces it, matching standard text-field behavior.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextEditState {
    /// The text being edited.
    pub text: String,
    /// Cursor position as a char index in `0..=char_len`.
    pub cursor: usize,
    /// Selection anchor (char index). `Some` means text between the anchor
    /// and the cursor is selected; `None` means no selection.
    pub selection_anchor: Option<usize>,
}

impl TextEditState {
    /// Number of chars in the buffer.
    fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    /// Byte offset of char index `i` (clamped to the end).
    fn byte_at(&self, i: usize) -> usize {
        self.text
            .char_indices()
            .nth(i)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    /// Replace the buffer and select all of it, cursor at the end.
    /// This is the click-to-focus behavior: typing overwrites the old value.
    pub fn set_text_select_all(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.char_len();
        self.selection_anchor = if self.text.is_empty() { None } else { Some(0) };
    }

    /// The selected range as `(start, end)` char indices, normalized so
    /// `start <= end`. `None` when there is no (or an empty) selection.
    pub fn selected_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        if anchor == self.cursor {
            return None;
        }
        Some((anchor.min(self.cursor), anchor.max(self.cursor)))
    }

    /// Select the entire buffer, cursor at the end.
    pub fn select_all(&mut self) {
        self.cursor = self.char_len();
        self.selection_anchor = if self.text.is_empty() { None } else { Some(0) };
    }

    /// Delete the selected text (if any) and collapse the cursor to the
    /// selection start. Returns true if something was deleted.
    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selected_range() else {
            self.selection_anchor = None;
            return false;
        };
        let (bs, be) = (self.byte_at(start), self.byte_at(end));
        self.text.replace_range(bs..be, "");
        self.cursor = start;
        self.selection_anchor = None;
        true
    }

    /// Insert a char at the cursor, replacing the selection if one is active.
    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        let at = self.byte_at(self.cursor);
        self.text.insert(at, c);
        self.cursor += 1;
    }

    /// Backspace: delete the selection, or the char before the cursor.
    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            let bs = self.byte_at(self.cursor - 1);
            let be = self.byte_at(self.cursor);
            self.text.replace_range(bs..be, "");
            self.cursor -= 1;
        }
    }

    /// Forward delete: delete the selection, or the char after the cursor.
    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.char_len() {
            let bs = self.byte_at(self.cursor);
            let be = self.byte_at(self.cursor + 1);
            self.text.replace_range(bs..be, "");
        }
    }

    /// Begin or extend a selection when `shift` is held; otherwise drop it.
    /// Returns the previous selection range for collapse handling.
    fn prepare_move(&mut self, shift: bool) -> Option<(usize, usize)> {
        let range = self.selected_range();
        if shift {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else {
            self.selection_anchor = None;
        }
        range
    }

    /// Move the cursor one char left. Plain arrow with an active selection
    /// collapses to the selection start without moving further.
    pub fn move_left(&mut self, shift: bool) {
        let prev = self.prepare_move(shift);
        if !shift {
            if let Some((start, _)) = prev {
                self.cursor = start;
                return;
            }
        }
        self.cursor = self.cursor.saturating_sub(1);
        self.drop_empty_selection();
    }

    /// Move the cursor one char right. Plain arrow with an active selection
    /// collapses to the selection end without moving further.
    pub fn move_right(&mut self, shift: bool) {
        let prev = self.prepare_move(shift);
        if !shift {
            if let Some((_, end)) = prev {
                self.cursor = end;
                return;
            }
        }
        self.cursor = (self.cursor + 1).min(self.char_len());
        self.drop_empty_selection();
    }

    /// Move the cursor to the start of the buffer.
    pub fn home(&mut self, shift: bool) {
        self.prepare_move(shift);
        self.cursor = 0;
        self.drop_empty_selection();
    }

    /// Move the cursor to the end of the buffer.
    pub fn end(&mut self, shift: bool) {
        self.prepare_move(shift);
        self.cursor = self.char_len();
        self.drop_empty_selection();
    }

    /// A shift-move that lands back on the anchor leaves no selection.
    fn drop_empty_selection(&mut self) {
        if self.selection_anchor == Some(self.cursor) {
            self.selection_anchor = None;
        }
    }

    /// Place the cursor from a click at `click_x` (widget-local pixels),
    /// given the prefix widths of the text: `prefix_widths[i]` is the pixel
    /// width of the first `i` chars, so it has `char_len + 1` entries.
    /// Picks the nearest char boundary and clears the selection.
    pub fn cursor_from_click(&mut self, prefix_widths: &[f32], click_x: f32) {
        let mut best = 0usize;
        let mut best_dist = f32::MAX;
        for (i, w) in prefix_widths.iter().enumerate() {
            let d = (click_x - w).abs();
            if d < best_dist {
                best_dist = d;
                best = i;
            }
        }
        self.cursor = best.min(self.char_len());
        self.selection_anchor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(text: &str, cursor: usize) -> TextEditState {
        TextEditState { text: text.to_string(), cursor, selection_anchor: None }
    }

    #[test]
    fn test_set_text_select_all_selects_everything() {
        let mut s = TextEditState::default();
        s.set_text_select_all("123.45");
        assert_eq!(s.selected_range(), Some((0, 6)));
        assert_eq!(s.cursor, 6);
    }

    #[test]
    fn test_set_text_select_all_empty_has_no_selection() {
        let mut s = TextEditState::default();
        s.set_text_select_all("");
        assert_eq!(s.selected_range(), None);
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_typing_replaces_selection() {
        let mut s = TextEditState::default();
        s.set_text_select_all("168.40");
        s.insert_char('5');
        assert_eq!(s.text, "5");
        assert_eq!(s.cursor, 1);
        assert_eq!(s.selected_range(), None);
    }

    #[test]
    fn test_insert_at_cursor_middle() {
        let mut s = state("129", 2);
        s.insert_char('8');
        assert_eq!(s.text, "1289");
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn test_backspace_at_middle_and_start() {
        let mut s = state("123", 2);
        s.backspace();
        assert_eq!(s.text, "13");
        assert_eq!(s.cursor, 1);

        let mut s = state("123", 0);
        s.backspace();
        assert_eq!(s.text, "123");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_backspace_deletes_selection() {
        // Selection covers chars 2..5 ("3.4") — backspace removes exactly
        // that range, not the char before the cursor.
        let mut s = state("123.45", 5);
        s.selection_anchor = Some(2);
        s.backspace();
        assert_eq!(s.text, "125");
        assert_eq!(s.cursor, 2);
        assert_eq!(s.selected_range(), None);
    }

    #[test]
    fn test_delete_forward_and_at_end() {
        let mut s = state("123", 1);
        s.delete();
        assert_eq!(s.text, "13");
        assert_eq!(s.cursor, 1);

        let mut s = state("123", 3);
        s.delete();
        assert_eq!(s.text, "123");
    }

    #[test]
    fn test_arrow_moves_and_clamps() {
        let mut s = state("12", 0);
        s.move_left(false);
        assert_eq!(s.cursor, 0);
        s.move_right(false);
        s.move_right(false);
        s.move_right(false);
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_plain_arrow_collapses_selection_to_edge() {
        let mut s = state("12345", 4);
        s.selection_anchor = Some(1);
        s.move_left(false);
        assert_eq!(s.cursor, 1, "left collapses to selection start");
        assert_eq!(s.selected_range(), None);

        let mut s = state("12345", 4);
        s.selection_anchor = Some(1);
        s.move_right(false);
        assert_eq!(s.cursor, 4, "right collapses to selection end");
    }

    #[test]
    fn test_shift_arrow_extends_selection() {
        let mut s = state("1234", 2);
        s.move_right(true);
        assert_eq!(s.selected_range(), Some((2, 3)));
        s.move_right(true);
        assert_eq!(s.selected_range(), Some((2, 4)));
        s.move_left(true);
        assert_eq!(s.selected_range(), Some((2, 3)));
        // Landing back on the anchor drops the selection
        s.move_left(true);
        assert_eq!(s.selected_range(), None);
    }

    #[test]
    fn test_home_end_with_and_without_shift() {
        let mut s = state("12345", 2);
        s.home(false);
        assert_eq!(s.cursor, 0);
        s.end(false);
        assert_eq!(s.cursor, 5);

        let mut s = state("12345", 2);
        s.end(true);
        assert_eq!(s.selected_range(), Some((2, 5)));
        s.home(true);
        assert_eq!(s.selected_range(), Some((0, 2)));
    }

    #[test]
    fn test_select_all_then_home_collapses() {
        let mut s = state("999", 0);
        s.select_all();
        assert_eq!(s.selected_range(), Some((0, 3)));
        s.home(false);
        assert_eq!(s.cursor, 0);
        assert_eq!(s.selected_range(), None);
    }

    #[test]
    fn test_cursor_from_click_picks_nearest_boundary() {
        let mut s = state("124", 0);
        // Prefix widths for "124": 0, 8, 16, 24 px
        let widths = [0.0, 8.0, 16.0, 24.0];
        s.cursor_from_click(&widths, 3.0);
        assert_eq!(s.cursor, 0);
        s.cursor_from_click(&widths, 5.0);
        assert_eq!(s.cursor, 1);
        s.cursor_from_click(&widths, 19.0);
        assert_eq!(s.cursor, 2);
        s.cursor_from_click(&widths, 100.0);
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn test_empty_string_operations_are_safe() {
        let mut s = TextEditState::default();
        s.backspace();
        s.delete();
        s.move_left(true);
        s.move_right(true);
        s.home(false);
        s.end(false);
        s.select_all();
        assert_eq!(s.text, "");
        assert_eq!(s.cursor, 0);
        assert_eq!(s.selected_range(), None);
    }
}
