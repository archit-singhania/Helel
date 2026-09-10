//! Stable, UI-independent foundations shared by Helel subsystems.

/// Returns the human-readable product name.
#[must_use]
pub const fn product_name() -> &'static str {
    "Helel"
}

pub mod workspace;

#[cfg(test)]
mod tests {
    use super::product_name;

    #[test]
    fn product_name_is_stable() {
        assert_eq!(product_name(), "Helel");
    }
}
