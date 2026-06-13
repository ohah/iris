//! Core contracts shared by Iris runtime experiments.

/// Public engine name used in diagnostics and benchmark output.
pub const ENGINE_NAME: &str = "Iris";

/// Compatibility target for the first proof-of-concept line.
pub const COMPATIBILITY_TARGET: &str = "React Native 0.85 with Hermes V1 behavioral compatibility";

/// Compatibility modes supported by Iris.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityMode {
    /// Preserve React Native and Hermes-observable behavior before taking
    /// performance shortcuts.
    Strict,
}

impl CompatibilityMode {
    /// Returns whether this mode allows observable JavaScript behavior changes.
    #[must_use]
    pub const fn allows_observable_behavior_changes(self) -> bool {
        match self {
            Self::Strict => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_mode_preserves_observable_behavior() {
        assert!(!CompatibilityMode::Strict.allows_observable_behavior_changes());
    }

    #[test]
    fn engine_metadata_is_stable() {
        assert_eq!(ENGINE_NAME, "Iris");
        assert!(COMPATIBILITY_TARGET.contains("React Native 0.85"));
    }
}
