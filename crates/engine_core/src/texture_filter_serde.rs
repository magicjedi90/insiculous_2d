//! Serde bridge for [`TextureFilter`].
//!
//! The renderer crate has no serde dependency, so the variants are mirrored
//! here; the encoding is the same one a derive on `TextureFilter` itself would
//! produce. Shared by every engine format that lets an author pick a sampling
//! mode — `GameConfig.texture_filter` and `.sheet.ron`'s `filter` — so the two
//! never drift apart on the wire.
//!
//! Use it as `#[serde(with = "crate::texture_filter_serde")]`.

use renderer::TextureFilter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Canonical wire form is `"Linear"` / `"Nearest"`; lowercase aliases are
/// accepted on read for hand-edited files. Unknown values still fail
/// loudly — silently coercing a typo to Linear would hide the error.
#[derive(Serialize, Deserialize)]
enum Repr {
    #[serde(alias = "linear")]
    Linear,
    #[serde(alias = "nearest")]
    Nearest,
}

impl From<TextureFilter> for Repr {
    fn from(filter: TextureFilter) -> Self {
        match filter {
            TextureFilter::Linear => Repr::Linear,
            TextureFilter::Nearest => Repr::Nearest,
        }
    }
}

impl From<Repr> for TextureFilter {
    fn from(repr: Repr) -> Self {
        match repr {
            Repr::Linear => TextureFilter::Linear,
            Repr::Nearest => TextureFilter::Nearest,
        }
    }
}

pub fn serialize<S: Serializer>(filter: &TextureFilter, serializer: S) -> Result<S::Ok, S::Error> {
    Repr::from(*filter).serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TextureFilter, D::Error> {
    Repr::deserialize(deserializer).map(TextureFilter::from)
}
