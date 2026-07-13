//! Change detection for sprite instance uploads (PATTERNS_AUDIT.md GPP-15).
//!
//! Flattening batches and uploading the instance buffer every frame is pure
//! waste when nothing on screen moved. [`InstanceCache`] stages the flattened
//! instances into a reusable buffer and reports whether they (or the batch
//! layout — texture boundaries) differ from what was last staged; the GPU
//! upload is skipped when they don't. Instances are compared as raw bytes
//! (they're `bytemuck::Pod`), so the check is exact and NaN-safe.

use bytemuck::cast_slice;

use crate::sprite::SpriteBatch;
use crate::sprite_data::SpriteInstance;
use crate::texture::TextureHandle;

/// Staging buffer + last-uploaded snapshot for sprite instances.
#[derive(Default)]
pub struct InstanceCache {
    /// Instances as last staged for upload.
    instances: Vec<SpriteInstance>,
    /// Batch layout as last staged: (texture, instance count) per batch.
    layout: Vec<(TextureHandle, usize)>,
    /// Scratch buffers reused across frames (no per-frame allocations).
    staging: Vec<SpriteInstance>,
    staging_layout: Vec<(TextureHandle, usize)>,
    uploads_performed: u64,
    uploads_skipped: u64,
}

impl InstanceCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Flatten `batches` into the staging buffer and report whether the
    /// result differs from what was last staged (i.e. whether a GPU upload
    /// is needed). On change the staged data becomes the new snapshot.
    pub fn stage(&mut self, batches: &[&SpriteBatch]) -> bool {
        self.staging.clear();
        self.staging_layout.clear();
        for batch in batches {
            self.staging.extend_from_slice(&batch.instances);
            self.staging_layout.push((batch.texture_handle, batch.instances.len()));
        }

        let unchanged = self.staging_layout == self.layout
            && cast_slice::<SpriteInstance, u8>(&self.staging)
                == cast_slice::<SpriteInstance, u8>(&self.instances);

        if unchanged {
            self.uploads_skipped += 1;
            false
        } else {
            std::mem::swap(&mut self.instances, &mut self.staging);
            std::mem::swap(&mut self.layout, &mut self.staging_layout);
            self.uploads_performed += 1;
            true
        }
    }

    /// The instances staged by the last [`stage`](Self::stage) call that
    /// reported a change — the data to upload.
    pub fn staged(&self) -> &[SpriteInstance] {
        &self.instances
    }

    /// Total number of `stage` calls that required an upload.
    pub fn uploads_performed(&self) -> u64 {
        self.uploads_performed
    }

    /// Total number of `stage` calls skipped because nothing changed.
    pub fn uploads_skipped(&self) -> u64 {
        self.uploads_skipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec4};

    fn instance(x: f32) -> SpriteInstance {
        SpriteInstance::new(Vec2::new(x, 0.0), 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 0.0)
    }

    fn batch_with(instances: &[SpriteInstance], texture: TextureHandle) -> SpriteBatch {
        let mut batch = SpriteBatch::new(texture);
        batch.add_instances(instances);
        batch
    }

    #[test]
    fn test_identical_batches_skip_upload() {
        let mut cache = InstanceCache::new();
        let batch = batch_with(&[instance(1.0), instance(2.0)], TextureHandle::WHITE);
        let refs = [&batch];

        assert!(cache.stage(&refs), "first stage must upload");
        assert!(!cache.stage(&refs), "identical restage must skip");
        assert!(!cache.stage(&refs), "and keep skipping");
        assert_eq!(cache.uploads_performed(), 1);
        assert_eq!(cache.uploads_skipped(), 2);
        assert_eq!(cache.staged().len(), 2, "snapshot holds the staged data");
    }

    #[test]
    fn test_instance_change_triggers_upload() {
        let mut cache = InstanceCache::new();
        let batch = batch_with(&[instance(1.0)], TextureHandle::WHITE);
        assert!(cache.stage(&[&batch]));

        // The sprite moved.
        let moved = batch_with(&[instance(5.0)], TextureHandle::WHITE);
        assert!(cache.stage(&[&moved]), "a moved instance must re-upload");
        assert_eq!(cache.staged()[0].position, [5.0, 0.0]);
    }

    #[test]
    fn test_layout_change_triggers_upload_even_with_same_bytes() {
        let mut cache = InstanceCache::new();
        // Two instances on one texture...
        let one = batch_with(&[instance(1.0), instance(2.0)], TextureHandle::WHITE);
        assert!(cache.stage(&[&one]));

        // ...vs the same flattened instances split across two textures: the
        // draw ranges differ, so this must count as changed.
        let a = batch_with(&[instance(1.0)], TextureHandle::WHITE);
        let b = batch_with(&[instance(2.0)], TextureHandle { id: 7 });
        assert!(
            cache.stage(&[&a, &b]),
            "same bytes with different batch boundaries must re-upload"
        );
    }

    #[test]
    fn test_empty_to_content_and_back() {
        let mut cache = InstanceCache::new();
        let empty: [&SpriteBatch; 0] = [];
        assert!(!cache.stage(&empty), "empty first frame stages nothing new");

        let batch = batch_with(&[instance(1.0)], TextureHandle::WHITE);
        assert!(cache.stage(&[&batch]), "content after empty must upload");

        assert!(cache.stage(&empty), "content -> empty is a change");
        assert!(cache.staged().is_empty());
    }
}
