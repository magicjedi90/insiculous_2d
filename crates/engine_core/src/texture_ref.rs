//! Texture reference resolution for scene files.
//!
//! Texture references in scene RON can be:
//! - `#white` — the built-in white texture (handle 0) for color tinting
//! - `#solid:RRGGBB` (or `RRGGBBAA`) — a generated solid-color texture
//! - Any other string — loaded as a file path

use renderer::TextureHandle;

use crate::assets::AssetManager;
use crate::scene_data::SceneLoadError;

/// Resolves scene texture references (`#white`, `#solid:RRGGBB`, file paths)
/// to texture handles.
///
/// [`AssetManager`] is the production implementation; tests substitute a
/// stub so scene and prefab instantiation stay headless-testable (no GPU).
pub trait TextureResolver {
    /// Resolve a texture reference string to a handle.
    fn resolve_texture(&mut self, texture_ref: &str) -> Result<TextureHandle, SceneLoadError>;
}

impl TextureResolver for AssetManager {
    fn resolve_texture(&mut self, texture_ref: &str) -> Result<TextureHandle, SceneLoadError> {
        resolve_texture(texture_ref, self)
    }
}

/// Resolve a texture reference to a TextureHandle.
pub(crate) fn resolve_texture(
    texture_ref: &str,
    assets: &mut AssetManager,
) -> Result<TextureHandle, SceneLoadError> {
    if texture_ref == "#white" {
        // White texture is always handle 0
        return Ok(TextureHandle { id: 0 });
    }

    if let Some(hex) = texture_ref.strip_prefix("#solid:") {
        // Parse hex color and create solid color texture
        let color = parse_hex_color(hex)?;
        assets
            .create_solid_color(1, 1, color)
            .map_err(|e| SceneLoadError::TextureLoadError(e.to_string()))
    } else if is_unresolvable_sentinel(texture_ref) {
        // Placeholder paths recorded for runtime-generated textures
        // ("#rgba", bare "#solid") carry no pixel data to rebuild from.
        // Degrade to the white texture instead of failing the scene load.
        log::warn!(
            "texture ref '{texture_ref}' is a generated-texture placeholder and cannot be \
             reloaded; substituting the white texture"
        );
        Ok(TextureHandle { id: 0 })
    } else {
        // Load as file path
        assets
            .load_texture(texture_ref)
            .map_err(|e| SceneLoadError::TextureLoadError(e.to_string()))
    }
}

/// A `#`-prefixed reference that names a runtime-generated texture rather
/// than a reconstructible scheme (`#white`, `#solid:RRGGBB`). Such refs can
/// be *written* by scene save (via `AssetManager::texture_path`) but carry
/// no data to rebuild the texture from on load.
pub(crate) fn is_unresolvable_sentinel(texture_ref: &str) -> bool {
    texture_ref.starts_with('#')
        && texture_ref != "#white"
        && !texture_ref.starts_with("#solid:")
}

/// Parse a hex color string (RRGGBB or RRGGBBAA) to [u8; 4]
pub(crate) fn parse_hex_color(hex: &str) -> Result<[u8; 4], SceneLoadError> {
    let hex = hex.trim_start_matches('#');

    if hex.len() == 6 {
        Ok([parse_hex_byte(hex, 0)?, parse_hex_byte(hex, 2)?, parse_hex_byte(hex, 4)?, 255])
    } else if hex.len() == 8 {
        Ok([parse_hex_byte(hex, 0)?, parse_hex_byte(hex, 2)?, parse_hex_byte(hex, 4)?, parse_hex_byte(hex, 6)?])
    } else {
        Err(SceneLoadError::InvalidTextureRef(format!(
            "Hex color must be 6 or 8 characters: {}",
            hex
        )))
    }
}

/// Parse a 2-character hex byte from a string at the given offset.
fn parse_hex_byte(hex: &str, start: usize) -> Result<u8, SceneLoadError> {
    u8::from_str_radix(&hex[start..start + 2], 16)
        .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_6() {
        let color = parse_hex_color("FF0000").unwrap();
        assert_eq!(color, [255, 0, 0, 255]);

        let color = parse_hex_color("00FF00").unwrap();
        assert_eq!(color, [0, 255, 0, 255]);

        let color = parse_hex_color("0000FF").unwrap();
        assert_eq!(color, [0, 0, 255, 255]);
    }

    #[test]
    fn test_parse_hex_color_8() {
        let color = parse_hex_color("FF000080").unwrap();
        assert_eq!(color, [255, 0, 0, 128]);
    }

    #[test]
    fn test_parse_hex_color_rejects_bad_length() {
        assert!(parse_hex_color("FFF").is_err());
        assert!(parse_hex_color("FF00FF00FF").is_err());
    }

    #[test]
    fn test_parse_hex_color_rejects_non_hex() {
        assert!(parse_hex_color("GGGGGG").is_err());
    }

    #[test]
    fn test_generated_texture_sentinels_are_flagged() {
        // Runtime-generated placeholders: degrade to white, never file-load.
        assert!(is_unresolvable_sentinel("#rgba"));
        assert!(is_unresolvable_sentinel("#solid"));
        // Reconstructible schemes and real paths are not sentinels.
        assert!(!is_unresolvable_sentinel("#white"));
        assert!(!is_unresolvable_sentinel("#solid:FF00FF"));
        assert!(!is_unresolvable_sentinel("sprites/player.png"));
    }
}
