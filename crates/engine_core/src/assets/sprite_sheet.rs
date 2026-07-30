//! Loading a sprite sheet: PNG + its `.sheet.ron` sidecar, in one call.
//!
//! Two paths reach a sidecar, and they fail differently on purpose:
//!
//! - [`AssetManager::load_sprite_sheet`] is the explicit API a game calls. It
//!   is **fail-loud** — a malformed sidecar is a `Result::Err` naming the file
//!   and the clip, because the caller asked for this sheet by name and can act
//!   on the answer.
//! - The implicit path, where a scene's `Sprite` happens to reference a PNG
//!   that has a sidecar, is **warn-and-carry-on**: a half-saved sidecar during
//!   authoring must not kill a scene load. That path goes through
//!   [`SidecarCache`], which reads each path at most once per scene load.
//!
//! Child module of `assets` so it can reach `AssetManager`'s private fields,
//! the same arrangement `game/render.rs` uses.

use std::collections::HashMap;
use std::path::Path;

use ecs::sprite_components::{Sprite, SpriteAnimation};
use renderer::{TextureFilter, TextureHandle};

use crate::sheet_file::{parse_sheet_file, sidecar_path_for};
use crate::texture_ref::SheetData;

use super::{AssetError, AssetManager};

/// A loaded sheet: its texture, how it is cut into cells, and its clips.
///
/// Turn it into components with [`animation`](Self::animation) and
/// [`sprite`](Self::sprite) — sheet to entity in two lines.
#[derive(Debug, Clone)]
pub struct SpriteSheet {
    /// Handle of the loaded PNG, sampled with the sidecar's filter.
    pub texture: TextureHandle,
    /// How the sheet is cut into cells (PNG size ÷ the sidecar's cell size).
    pub grid: common::SheetGrid,
    /// Named clips over those cells, in the sidecar's order.
    pub clips: Vec<(String, ecs::sprite_components::AnimationClip)>,
    /// The PNG path exactly as it was passed to
    /// [`AssetManager::load_sprite_sheet`]. Stamped onto every
    /// [`SpriteAnimation`] this sheet builds so a scene reload can find the
    /// sidecar again.
    pub path: String,
}

impl SpriteSheet {
    /// A [`SpriteAnimation`] carrying this sheet's grid, clips, and path.
    /// Nothing is playing yet — call `play("walk")` or set `autoplay` in the
    /// scene file.
    pub fn animation(&self) -> SpriteAnimation {
        SpriteAnimation {
            grid: self.grid,
            clips: self.clips.clone(),
            sheet: Some(self.path.clone()),
            ..SpriteAnimation::default()
        }
    }

    /// A [`Sprite`] bound to this sheet's texture, showing the first cell.
    pub fn sprite(&self) -> Sprite {
        let region = self.grid.uv_rect(0);
        Sprite::new(self.texture.id).with_tex_region(region[0], region[1], region[2], region[3])
    }
}

/// A sidecar resolved against its PNG — everything a sheet contributes
/// except the texture itself.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedSheet {
    /// Grid and clips, in the shape the scene loader consumes.
    pub sheet: SheetData,
    /// How the PNG should be sampled.
    pub filter: TextureFilter,
}

/// Read and validate the sidecar belonging to `texture_ref`, resolving both
/// paths against `base_path`.
///
/// This is everything [`AssetManager::load_sprite_sheet`] does before it
/// touches the GPU, split out so the whole validation chain is testable
/// without a device — and so a failure provably happens before any texture
/// handle is allocated.
pub fn prepare_sheet(base_path: &Path, texture_ref: &str) -> Result<PreparedSheet, AssetError> {
    let sidecar_rel = sidecar_path_for(texture_ref);
    let label = sidecar_rel.to_string_lossy().to_string();
    let full_sidecar = resolve_against(base_path, &sidecar_rel);

    let text = std::fs::read_to_string(&full_sidecar).map_err(|e| {
        AssetError::NotFound(format!("{}: {e}", full_sidecar.to_string_lossy()))
    })?;
    let sheet = parse_sheet_file(&label, &text)?;

    // A header probe, not a decode: the grid needs the PNG's pixel size and
    // nothing else, so validation stays cheap and GPU-free.
    let full_png = resolve_against(base_path, Path::new(texture_ref));
    let (width, height) = image::image_dimensions(&full_png).map_err(|e| {
        AssetError::InvalidData(format!(
            "{}: cannot read image dimensions: {e}",
            full_png.to_string_lossy()
        ))
    })?;

    let (grid, clips, filter) = sheet.into_parts(&label, width, height)?;
    Ok(PreparedSheet {
        sheet: SheetData { grid, clips },
        filter,
    })
}

fn resolve_against(base_path: &Path, path: &Path) -> std::path::PathBuf {
    if path.is_relative() {
        base_path.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Per-path sidecar reads, held for the duration of one scene load.
///
/// Without it, every `Sprite` referencing a sheet would re-read and re-parse
/// the same sidecar. [`clear`](Self::clear) runs at the top of each scene
/// load, so an artist's edit — including fixing a file that was malformed a
/// moment ago — takes effect on reload with no file watcher.
///
/// A path with no sidecar, or an unusable one, caches `None`: the warning is
/// emitted once and the default filter applies.
#[derive(Debug, Default)]
pub struct SidecarCache {
    entries: HashMap<String, Option<PreparedSheet>>,
}

impl SidecarCache {
    /// The sidecar for `texture_ref`, reading it on first ask.
    pub fn get(&mut self, base_path: &Path, texture_ref: &str) -> Option<&PreparedSheet> {
        if !self.entries.contains_key(texture_ref) {
            let entry = Self::read(base_path, texture_ref);
            self.entries.insert(texture_ref.to_string(), entry);
        }
        self.entries.get(texture_ref).and_then(Option::as_ref)
    }

    /// Forget every cached read.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn read(base_path: &Path, texture_ref: &str) -> Option<PreparedSheet> {
        // Generated-texture references (`#white`, `#solid:...`) are not files
        // and never have sidecars.
        if texture_ref.starts_with('#') {
            return None;
        }
        // No sidecar is the ordinary case for a plain image — not worth a
        // warning, and not worth a parse attempt.
        if !resolve_against(base_path, &sidecar_path_for(texture_ref)).exists() {
            return None;
        }

        match prepare_sheet(base_path, texture_ref) {
            Ok(prepared) => Some(prepared),
            Err(e) => {
                log::warn!(
                    "Ignoring the sheet sidecar for '{texture_ref}': {e}. \
                     Sampling with the default filter and the scene's baked values instead."
                );
                None
            }
        }
    }
}

impl AssetManager {
    /// Load a sprite sheet: its PNG plus the `.sheet.ron` sidecar sitting
    /// next to it under the same stem.
    ///
    /// `path` names the **PNG**, relative to the asset base path; the sidecar
    /// path is derived from it, so `load_sprite_sheet("sprites/deion_16.png")`
    /// reads `sprites/deion_16.sheet.ron`.
    ///
    /// The sidecar is read, parsed and validated against the PNG's dimensions
    /// *before* the texture is loaded, so a sheet that fails validation leaves
    /// no texture behind to clean up. Errors name the file and, where it
    /// applies, the clip.
    ///
    /// # Example
    /// ```no_run
    /// # use engine_core::prelude::*;
    /// # fn load(assets: &mut AssetManager, world: &mut ecs::World) -> Result<(), AssetError> {
    /// let sheet = assets.load_sprite_sheet("sprites/deion_16.png")?;
    /// let hero = world.create_entity();
    /// world.add_component(&hero, sheet.sprite()).ok();
    /// let mut animation = sheet.animation();
    /// animation.play("walk");
    /// world.add_component(&hero, animation).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_sprite_sheet<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<SpriteSheet, AssetError> {
        let path = path.as_ref();
        let texture_ref = path.to_string_lossy().to_string();

        let prepared = prepare_sheet(Path::new(&self.config.base_path), &texture_ref)?;
        // Only now, with everything validated, is a texture handle allocated.
        let texture = self.load_texture_filtered(path, prepared.filter)?;

        Ok(SpriteSheet {
            texture,
            grid: prepared.sheet.grid,
            clips: prepared.sheet.clips,
            path: texture_ref,
        })
    }

    /// Drop every cached sidecar read. Called at the top of each scene load.
    pub fn clear_sidecar_cache(&mut self) {
        self.sidecar_cache.clear();
    }

    /// The sampling filter a texture's sidecar asks for, if it has a usable
    /// one.
    pub(crate) fn sidecar_filter(&mut self, texture_ref: &str) -> Option<TextureFilter> {
        let base = self.config.base_path.clone();
        self.sidecar_cache
            .get(Path::new(&base), texture_ref)
            .map(|prepared| prepared.filter)
    }

    /// The grid and clips a texture's sidecar declares, if it has a usable
    /// one.
    pub(crate) fn sidecar_sheet(&mut self, texture_ref: &str) -> Option<SheetData> {
        let base = self.config.base_path.clone();
        self.sidecar_cache
            .get(Path::new(&base), texture_ref)
            .map(|prepared| prepared.sheet.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SIDECAR: &str = r#"SheetFile(
    version: 1,
    cell: (16, 16),
    clips: [("walk", (frames: [0, 1, 2, 3], fps: 8.0))],
)"#;

    /// A temp asset root holding a 64x32 PNG and, optionally, a sidecar.
    fn asset_root(sidecar: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let png = image::RgbaImage::new(64, 32);
        png.save(dir.path().join("deion.png")).expect("write png");
        if let Some(text) = sidecar {
            fs::write(dir.path().join("deion.sheet.ron"), text).expect("write sidecar");
        }
        dir
    }

    #[test]
    fn test_prepare_sheet_resolves_the_grid_and_filter() {
        let dir = asset_root(Some(SIDECAR));

        let prepared = prepare_sheet(dir.path(), "deion.png").expect("prepares");

        assert_eq!((prepared.sheet.grid.cols, prepared.sheet.grid.rows), (4, 2));
        assert_eq!(prepared.filter, TextureFilter::Nearest);
        assert_eq!(prepared.sheet.clips[0].0, "walk");
        assert_eq!(prepared.sheet.clips[0].1.frame_indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_prepare_sheet_reads_the_same_stem_sidecar() {
        let dir = asset_root(Some(SIDECAR));
        // The wrong derivation would look for `deion.png.sheet.ron`.
        fs::write(dir.path().join("deion.png.sheet.ron"), "garbage").expect("write decoy");

        assert!(prepare_sheet(dir.path(), "deion.png").is_ok());
    }

    #[test]
    fn test_prepare_sheet_fails_before_any_texture_is_loaded() {
        // A clip past the end of the 8-cell sheet: the error must come from
        // validation, which runs before `load_sprite_sheet` reaches the GPU.
        let dir = asset_root(Some(
            r#"SheetFile(version: 1, cell: (16, 16), clips: [("walk", (frames: [99], fps: 8.0))])"#,
        ));

        let err = prepare_sheet(dir.path(), "deion.png").expect_err("invalid clip index");

        let message = err.to_string();
        assert!(message.contains("'walk'"), "{message}");
        assert!(message.contains("99"), "{message}");
    }

    #[test]
    fn test_prepare_sheet_reports_a_missing_sidecar() {
        let dir = asset_root(None);

        let err = prepare_sheet(dir.path(), "deion.png").expect_err("no sidecar");

        assert!(err.to_string().contains("deion.sheet.ron"), "{err}");
    }

    #[test]
    fn test_prepare_sheet_reports_a_missing_png() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("deion.sheet.ron"), SIDECAR).expect("write sidecar");

        let err = prepare_sheet(dir.path(), "deion.png").expect_err("no png to size the grid");

        assert!(err.to_string().contains("dimensions"), "{err}");
    }

    #[test]
    fn test_cache_serves_a_sidecar_filter() {
        let dir = asset_root(Some(SIDECAR));
        let mut cache = SidecarCache::default();

        let prepared = cache.get(dir.path(), "deion.png").expect("sidecar found");
        assert_eq!(prepared.filter, TextureFilter::Nearest);
    }

    #[test]
    fn test_cache_does_not_re_read_the_filesystem() {
        let dir = asset_root(Some(SIDECAR));
        let mut cache = SidecarCache::default();
        assert!(cache.get(dir.path(), "deion.png").is_some());

        // Delete the sidecar: a cached answer survives, a re-read could not.
        fs::remove_file(dir.path().join("deion.sheet.ron")).expect("remove sidecar");
        assert!(cache.get(dir.path(), "deion.png").is_some());
    }

    #[test]
    fn test_clearing_the_cache_picks_up_an_edited_sidecar() {
        let dir = asset_root(Some(SIDECAR));
        let mut cache = SidecarCache::default();
        assert_eq!(cache.get(dir.path(), "deion.png").unwrap().sheet.grid.cols, 4);

        // The artist re-cut the sheet into 8px cells.
        fs::write(
            dir.path().join("deion.sheet.ron"),
            r#"SheetFile(version: 1, cell: (8, 8), clips: [("walk", (frames: [0], fps: 8.0))])"#,
        )
        .expect("rewrite sidecar");

        cache.clear();
        let prepared = cache.get(dir.path(), "deion.png").expect("re-read");
        assert_eq!((prepared.sheet.grid.cols, prepared.sheet.grid.rows), (8, 4));
    }

    #[test]
    fn test_cache_returns_nothing_for_a_texture_without_a_sidecar() {
        let dir = asset_root(None);
        let mut cache = SidecarCache::default();

        assert!(cache.get(dir.path(), "deion.png").is_none());
    }

    #[test]
    fn test_cache_falls_back_quietly_on_a_malformed_sidecar() {
        let dir = asset_root(Some("this is not RON at all"));
        let mut cache = SidecarCache::default();

        // Warned, cached as absent — a half-saved file must not fail a load.
        assert!(cache.get(dir.path(), "deion.png").is_none());
        assert!(cache.get(dir.path(), "deion.png").is_none());
    }

    #[test]
    fn test_cache_ignores_generated_texture_references() {
        let dir = asset_root(Some(SIDECAR));
        let mut cache = SidecarCache::default();

        assert!(cache.get(dir.path(), "#white").is_none());
        assert!(cache.get(dir.path(), "#solid:FF00FF").is_none());
    }

    #[test]
    fn test_sprite_sheet_builds_components_bound_to_the_sheet() {
        let sheet = SpriteSheet {
            texture: TextureHandle { id: 7 },
            grid: common::SheetGrid::new(4, 2),
            clips: vec![(
                "walk".to_string(),
                ecs::sprite_components::AnimationClip::new(vec![0, 1], 8.0),
            )],
            path: "sprites/deion.png".to_string(),
        };

        let animation = sheet.animation();
        assert_eq!(animation.sheet.as_deref(), Some("sprites/deion.png"));
        assert_eq!((animation.grid.cols, animation.grid.rows), (4, 2));
        assert!(animation.has_clip("walk"));
        // Nothing plays until the game or the scene says so.
        assert!(!animation.playing);

        let sprite = sheet.sprite();
        assert_eq!(sprite.texture_handle, 7);
        assert_eq!(sprite.tex_region, [0.0, 0.0, 0.25, 0.5]);
    }
}
