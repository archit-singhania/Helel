//! Stable, UI-independent foundations shared by Helel subsystems.

/// Returns the human-readable product name.
#[must_use]
pub const fn product_name() -> &'static str {
    "Helel"
}

pub mod agent;
pub mod audit;
pub mod intelligence;
pub mod mcp;
pub mod orchestrator;
pub mod project;
pub mod proposal;
pub mod system;
pub mod workspace;

#[cfg(test)]
mod tests {
    use super::product_name;

    #[test]
    fn product_name_is_stable() {
        assert_eq!(product_name(), "Helel");
    }
}
