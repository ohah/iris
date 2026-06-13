//! JSI-facing Iris contracts.
//!
//! This crate is intentionally thin for now. The C++ JSI boundary will live
//! here, while Rust-owned buffers and runtime state stay behind explicit APIs.

/// Returns the React Native compatibility target for this JSI layer.
#[must_use]
pub fn compatibility_target() -> &'static str {
    iris_core::COMPATIBILITY_TARGET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_compatibility_target() {
        assert!(compatibility_target().contains("Hermes V1"));
    }
}
