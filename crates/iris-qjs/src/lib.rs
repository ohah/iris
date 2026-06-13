//! QuickJS backend experiment boundary for Iris.
//!
//! The first Iris QuickJS work should prove adapter compatibility before it is
//! treated as a production runtime path.

/// Backend identifier used in benchmarks.
pub const BACKEND_NAME: &str = "iris-qjs";

/// Returns a short backend description for benchmark reports.
#[must_use]
pub fn backend_description() -> String {
    format!("{BACKEND_NAME} for {}", iris_core::COMPATIBILITY_TARGET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_description_names_quickjs_experiment() {
        assert!(backend_description().starts_with(BACKEND_NAME));
    }
}
