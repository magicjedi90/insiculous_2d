//! Project-wide gameplay intensity selector.
//!
//! Each game decides what each variant *means*; the engine just carries the
//! selection. A game reads [`GameContext::chaos_mode`] (or checks the field on
//! its [`GameConfig`]) and branches gameplay accordingly.
//!
//! The engine intentionally ships no gameplay logic for these variants — a
//! racing game's "Insane" will look nothing like a Pong "Insane". The enum
//! exists to keep the *vocabulary* consistent across games.

use serde::{Deserialize, Serialize};

/// Recurring gameplay intensity theme: Normal / Insane / Ridiculous / Insiculous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ChaosMode {
    #[default]
    Normal,
    Insane,
    Ridiculous,
    Insiculous,
}

impl ChaosMode {
    pub const ALL: [ChaosMode; 4] = [
        ChaosMode::Normal,
        ChaosMode::Insane,
        ChaosMode::Ridiculous,
        ChaosMode::Insiculous,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ChaosMode::Normal => "Normal",
            ChaosMode::Insane => "Insane",
            ChaosMode::Ridiculous => "Ridiculous",
            ChaosMode::Insiculous => "Insiculous",
        }
    }

    pub fn is_insane(self) -> bool {
        matches!(self, ChaosMode::Insane | ChaosMode::Insiculous)
    }

    pub fn is_ridiculous(self) -> bool {
        matches!(self, ChaosMode::Ridiculous | ChaosMode::Insiculous)
    }

    /// True only for `Insiculous` — the "both at once" combined mode.
    ///
    /// `is_insane()` and `is_ridiculous()` are both true in Insiculous as
    /// well, so games usually branch on those. Use this when a behavior
    /// should fire *only* in the combined mode and not in pure Insane or
    /// pure Ridiculous.
    pub fn is_insiculous(self) -> bool {
        matches!(self, ChaosMode::Insiculous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(ChaosMode::default(), ChaosMode::Normal);
    }

    #[test]
    fn all_variants_have_nonempty_labels() {
        for mode in ChaosMode::ALL {
            assert!(!mode.label().is_empty(), "missing label for {mode:?}");
        }
    }

    #[test]
    fn insane_flag_matches_insane_and_insiculous() {
        assert!(!ChaosMode::Normal.is_insane());
        assert!(ChaosMode::Insane.is_insane());
        assert!(!ChaosMode::Ridiculous.is_insane());
        assert!(ChaosMode::Insiculous.is_insane());
    }

    #[test]
    fn ridiculous_flag_matches_ridiculous_and_insiculous() {
        assert!(!ChaosMode::Normal.is_ridiculous());
        assert!(!ChaosMode::Insane.is_ridiculous());
        assert!(ChaosMode::Ridiculous.is_ridiculous());
        assert!(ChaosMode::Insiculous.is_ridiculous());
    }

    #[test]
    fn insiculous_flag_only_matches_insiculous() {
        assert!(!ChaosMode::Normal.is_insiculous());
        assert!(!ChaosMode::Insane.is_insiculous());
        assert!(!ChaosMode::Ridiculous.is_insiculous());
        assert!(ChaosMode::Insiculous.is_insiculous());
    }

    #[test]
    fn all_covers_four_distinct_variants() {
        let labels: std::collections::HashSet<_> =
            ChaosMode::ALL.iter().map(|m| m.label()).collect();
        assert_eq!(labels.len(), 4);
    }
}
