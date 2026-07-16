//! Shared menu window chrome: an opaque, bordered panel with a title band,
//! accent separator, corner ticks, cursor-highlighted rows, and a hint
//! footer — so menu screens read as real UI instead of floating text.
//!
//! The engine owns the chrome; games own the content and colors (build a
//! [`MenuStyle`] from the game's [`ChaosTheme`]). Structural flair (borders,
//! separator bars, highlight bars, corner ticks) is drawn with rects so it
//! can never fall victim to missing font glyphs; the `▶` selection cursor is
//! verified present in the games' shared `font.ttf`.
//!
//! Typical menu screen:
//! ```no_run
//! use engine_core::prelude::*;
//! use engine_core::menu_panel::{MenuPanel, MenuStyle};
//!
//! # fn draw(ctx: &mut GameContext, selection: u8) {
//! let style = MenuStyle::from_theme(&ChaosTheme::for_mode(ctx.chaos_mode));
//! let panel = MenuPanel::new("MY GAME", ctx.window_size / 2.0, 340.0, 3);
//! let mut y = panel.begin(ctx.ui, &style);
//! for (i, item) in ["Play", "Options", "Quit"].iter().enumerate() {
//!     y = panel.item(ctx.ui, y, item, i as u8 == selection, &style);
//! }
//! panel.hint(ctx.ui, "W/S navigate - SPACE confirm", &style);
//! # }
//! ```

use glam::{Vec2, Vec4};
use ui::{Color, Rect, UIContext};

use crate::chaos_theme::ChaosTheme;

/// Vertical space per selectable row.
const ROW_HEIGHT: f32 = 30.0;
/// Space reserved for the title label + separator at the top of the panel.
const TITLE_BAND: f32 = 52.0;
/// Space reserved for the hint footer at the bottom of the panel.
const HINT_BAND: f32 = 34.0;
/// Inner padding on every side.
const PADDING: f32 = 16.0;
/// Border thickness of the window frame.
const BORDER: f32 = 2.0;
/// Length of the corner accent ticks.
const TICK_LEN: f32 = 14.0;

fn color(v: Vec4) -> Color {
    Color::new(v.x, v.y, v.z, v.w)
}

/// Colors for one menu window, typically derived from the game's chaos
/// theme so menus keep each game's identity.
#[derive(Debug, Clone, Copy)]
pub struct MenuStyle {
    /// Opaque window fill — darker than the game background so the panel
    /// reads as a distinct surface.
    pub background: Vec4,
    /// Window frame color.
    pub border: Vec4,
    /// Title, separator, corner ticks, and selected-item highlight.
    pub accent: Vec4,
    /// Body text.
    pub text: Vec4,
    /// Unselected items and hint text.
    pub dim: Vec4,
}

impl MenuStyle {
    /// Derive a window style from a chaos theme: near-black fill tinted
    /// toward the theme's background, structure-colored frame, accent
    /// highlights.
    pub fn from_theme(theme: &ChaosTheme) -> Self {
        let bg = theme.bg_color;
        Self {
            background: Vec4::new(
                bg.x * 0.35 + 0.02,
                bg.y * 0.35 + 0.02,
                bg.z * 0.35 + 0.05,
                0.97,
            ),
            border: theme.structure_color,
            accent: theme.accent_color,
            text: Vec4::new(0.9, 0.92, 0.95, 1.0),
            dim: Vec4::new(0.55, 0.57, 0.63, 1.0),
        }
    }
}

/// One menu window: fixed title/center/width/row-count, drawn as layered
/// rects + labels. Construct per frame (it's just layout parameters).
pub struct MenuPanel {
    title: String,
    center: Vec2,
    width: f32,
    rows: usize,
}

impl MenuPanel {
    pub fn new(title: &str, center: Vec2, width: f32, rows: usize) -> Self {
        Self { title: title.to_string(), center, width, rows }
    }

    /// The window's bounds: title band + rows + hint band + padding,
    /// centered on `center`.
    pub fn panel_rect(&self) -> Rect {
        let height = TITLE_BAND + self.rows as f32 * ROW_HEIGHT + HINT_BAND + 2.0 * PADDING;
        Rect::new(
            self.center.x - self.width / 2.0,
            self.center.y - height / 2.0,
            self.width,
            height,
        )
    }

    /// Draw the window frame (opaque fill, border, title, accent separator,
    /// corner ticks). Returns the y center of the first content row.
    pub fn begin(&self, ui: &mut UIContext, style: &MenuStyle) -> f32 {
        let rect = self.panel_rect();
        ui.panel_styled(rect, color(style.background), color(style.border), BORDER);

        // Title + accent separator underneath it
        ui.label_centered_styled(
            &self.title,
            Vec2::new(self.center.x, rect.y + PADDING + 14.0),
            color(style.accent),
            20.0,
        );
        let sep_y = rect.y + PADDING + TITLE_BAND - 14.0;
        let sep_inset = PADDING + 6.0;
        ui.panel_styled(
            Rect::new(rect.x + sep_inset, sep_y, rect.width - 2.0 * sep_inset, 2.0),
            color(style.accent),
            color(style.accent),
            0.0,
        );

        // Corner ticks: small accent L-marks just inside each corner
        for (cx, sx) in [(rect.x + 4.0, 1.0), (rect.x + rect.width - 4.0 - TICK_LEN, -1.0)] {
            let _ = sx;
            for cy in [rect.y + 4.0, rect.y + rect.height - 7.0] {
                ui.panel_styled(
                    Rect::new(cx, cy, TICK_LEN, 3.0),
                    color(style.accent),
                    color(style.accent),
                    0.0,
                );
            }
        }

        rect.y + PADDING + TITLE_BAND + ROW_HEIGHT / 2.0
    }

    /// One selectable row at y-center `y`: the selection gets a translucent
    /// accent highlight bar, a `▶` cursor, and accent text; unselected rows
    /// are dim. Returns the next row's y.
    pub fn item(&self, ui: &mut UIContext, y: f32, text: &str, selected: bool, style: &MenuStyle) -> f32 {
        let item_color = if selected { style.accent } else { style.dim };
        self.item_colored(ui, y, text, item_color, selected, style)
    }

    /// Like [`item`](Self::item) but with a caller-chosen text color (e.g.
    /// breakout's per-chaos-mode level entries). Selection still draws the
    /// highlight bar + cursor.
    pub fn item_colored(
        &self,
        ui: &mut UIContext,
        y: f32,
        text: &str,
        text_color: Vec4,
        selected: bool,
        style: &MenuStyle,
    ) -> f32 {
        let rect = self.panel_rect();
        if selected {
            let inset = PADDING + 2.0;
            let bar = Vec4::new(style.accent.x, style.accent.y, style.accent.z, 0.18);
            ui.panel_styled(
                Rect::new(rect.x + inset, y - ROW_HEIGHT / 2.0 + 4.0, rect.width - 2.0 * inset, ROW_HEIGHT - 8.0),
                color(bar),
                color(bar),
                0.0,
            );
            ui.label_centered_styled(
                &format!("▶ {text}"),
                Vec2::new(self.center.x, y),
                color(text_color),
                16.0,
            );
        } else {
            ui.label_centered_styled(text, Vec2::new(self.center.x, y), color(text_color), 16.0);
        }
        y + ROW_HEIGHT
    }

    /// A non-selectable centered text row (status lines, prompts on
    /// game-over panels). Returns the next row's y.
    pub fn line(&self, ui: &mut UIContext, y: f32, text: &str, style: &MenuStyle) -> f32 {
        ui.label_centered_styled(text, Vec2::new(self.center.x, y), color(style.text), 16.0);
        y + ROW_HEIGHT
    }

    /// Dim footer hint inside the bottom band of the window.
    pub fn hint(&self, ui: &mut UIContext, text: &str, style: &MenuStyle) {
        let rect = self.panel_rect();
        ui.label_centered_styled(
            text,
            Vec2::new(self.center.x, rect.y + rect.height - PADDING - 8.0),
            color(style.dim),
            12.0,
        );
    }

    /// Draw this window as an input-blocking overlay (pause menus): dims the
    /// whole screen, blocks clicks through it, then runs `content` with the
    /// first row's y (draw items/hints inside).
    pub fn draw_as_overlay(
        &self,
        ui: &mut UIContext,
        window_size: Vec2,
        style: &MenuStyle,
        content: impl FnOnce(&Self, &mut UIContext, f32),
    ) {
        let screen = Rect::new(0.0, 0.0, window_size.x, window_size.y);
        ui.begin_overlay(screen);
        // Dim the frozen world beneath
        let scrim = Color::new(0.0, 0.0, 0.0, 0.55);
        ui.panel_styled(screen, scrim, scrim, 0.0);
        let first_y = self.begin(ui, style);
        content(self, ui, first_y);
        ui.end_overlay();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(rows: usize) -> MenuPanel {
        MenuPanel::new("TITLE", Vec2::new(400.0, 300.0), 320.0, rows)
    }

    #[test]
    fn panel_rect_is_centered_and_grows_with_rows() {
        let three = panel(3).panel_rect();
        assert_eq!(three.x + three.width / 2.0, 400.0);
        assert_eq!(three.y + three.height / 2.0, 300.0);
        assert_eq!(three.width, 320.0);

        let five = panel(5).panel_rect();
        assert_eq!(five.height - three.height, 2.0 * ROW_HEIGHT);
        // Still centered after growing
        assert_eq!(five.y + five.height / 2.0, 300.0);
    }

    #[test]
    fn panel_fits_inside_a_standard_window() {
        // The biggest roster menu (4 levels + title + hint) must fit 800x600
        let rect = panel(4).panel_rect();
        assert!(rect.y > 0.0 && rect.y + rect.height < 600.0, "{rect:?}");
        assert!(rect.x > 0.0 && rect.x + rect.width < 800.0, "{rect:?}");
    }

    #[test]
    fn menu_style_from_theme_is_opaque_and_darker_than_game_bg() {
        let theme = ChaosTheme::for_mode(crate::chaos_mode::ChaosMode::Normal);
        let style = MenuStyle::from_theme(&theme);
        assert!(style.background.w > 0.9, "panel must read as a solid window");
        assert!(style.background.x < theme.bg_color.x + 0.06);
        assert!(style.background.y < theme.bg_color.y + 0.06);
        assert_eq!(style.accent, theme.accent_color);
        assert_eq!(style.border, theme.structure_color);
    }
}
