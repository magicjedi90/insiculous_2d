//! UI styling system with colors and visual properties.

// Re-export Color from common crate
pub use common::Color;

/// Style configuration for buttons.
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    /// Background color in normal state
    pub background: Color,
    /// Background color when hovered
    pub background_hovered: Color,
    /// Background color when pressed
    pub background_pressed: Color,
    /// Background color when disabled
    pub background_disabled: Color,
    /// Border color
    pub border: Color,
    /// Border width in pixels
    pub border_width: f32,
    /// Corner radius in pixels
    pub corner_radius: f32,
    /// Text color
    pub text_color: Color,
    /// Text color when disabled
    pub text_color_disabled: Color,
    /// Padding inside the button
    pub padding: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            background: Color::from_hex(0x3A3A3A),
            background_hovered: Color::from_hex(0x4A4A4A),
            background_pressed: Color::from_hex(0x2A2A2A),
            background_disabled: Color::from_hex(0x2A2A2A),
            border: Color::from_hex(0x5A5A5A),
            border_width: 1.0,
            corner_radius: 4.0,
            text_color: Color::WHITE,
            text_color_disabled: Color::GRAY,
            padding: 8.0,
        }
    }
}

/// Style configuration for panels/containers.
#[derive(Debug, Clone)]
pub struct PanelStyle {
    /// Background color
    pub background: Color,
    /// Border color
    pub border: Color,
    /// Border width in pixels
    pub border_width: f32,
    /// Corner radius in pixels
    pub corner_radius: f32,
    /// Padding inside the panel
    pub padding: f32,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            background: Color::from_hex(0x2A2A2A).with_alpha(0.9),
            border: Color::from_hex(0x4A4A4A),
            border_width: 1.0,
            corner_radius: 4.0,
            padding: 8.0,
        }
    }
}

/// Style configuration for sliders.
#[derive(Debug, Clone)]
pub struct SliderStyle {
    /// Track background color
    pub track_background: Color,
    /// Track fill color (portion before thumb)
    pub track_fill: Color,
    /// Track height in pixels
    pub track_height: f32,
    /// Thumb (handle) color
    pub thumb_color: Color,
    /// Thumb color when hovered
    pub thumb_hovered: Color,
    /// Thumb color when pressed
    pub thumb_pressed: Color,
    /// Thumb radius in pixels
    pub thumb_radius: f32,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_background: Color::from_hex(0x3A3A3A),
            track_fill: Color::from_hex(0x4A90D9),
            track_height: 6.0,
            thumb_color: Color::WHITE,
            thumb_hovered: Color::LIGHT_GRAY,
            thumb_pressed: Color::from_hex(0x4A90D9),
            thumb_radius: 8.0,
        }
    }
}

/// Style configuration for text labels.
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Text color
    pub color: Color,
    /// Font size in pixels
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            font_size: 16.0,
            line_height: 1.2,
        }
    }
}

/// Global UI theme containing all widget styles.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Theme {
    /// Button style
    pub button: ButtonStyle,
    /// Panel style
    pub panel: PanelStyle,
    /// Slider style
    pub slider: SliderStyle,
    /// Text style
    pub text: TextStyle,
}


impl Theme {
    /// Create a dark theme (default).
    pub fn dark() -> Self {
        Self::default()
    }

    /// Create a light theme.
    pub fn light() -> Self {
        Self {
            button: ButtonStyle {
                background: Color::from_hex(0xE0E0E0),
                background_hovered: Color::from_hex(0xD0D0D0),
                background_pressed: Color::from_hex(0xC0C0C0),
                background_disabled: Color::from_hex(0xF0F0F0),
                border: Color::from_hex(0xB0B0B0),
                border_width: 1.0,
                corner_radius: 4.0,
                text_color: Color::BLACK,
                text_color_disabled: Color::GRAY,
                padding: 8.0,
            },
            panel: PanelStyle {
                background: Color::from_hex(0xF5F5F5).with_alpha(0.95),
                border: Color::from_hex(0xD0D0D0),
                border_width: 1.0,
                corner_radius: 4.0,
                padding: 8.0,
            },
            slider: SliderStyle {
                track_background: Color::from_hex(0xD0D0D0),
                track_fill: Color::from_hex(0x4A90D9),
                track_height: 6.0,
                thumb_color: Color::WHITE,
                thumb_hovered: Color::from_hex(0xF0F0F0),
                thumb_pressed: Color::from_hex(0x4A90D9),
                thumb_radius: 8.0,
            },
            text: TextStyle {
                color: Color::BLACK,
                font_size: 16.0,
                line_height: 1.2,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_defaults() {
        let theme = Theme::default();
        assert!(theme.button.padding > 0.0);
        assert!(theme.panel.padding > 0.0);
        assert!(theme.slider.thumb_radius > 0.0);
        assert!(theme.text.font_size > 0.0);
    }

    #[test]
    fn test_color_reexport_works() {
        // Verify common::Color is properly re-exported
        let color = Color::from_hex(0xFF0000);
        assert!((color.r - 1.0).abs() < 0.01);
    }
}
