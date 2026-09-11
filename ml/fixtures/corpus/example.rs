/// Returns a small deterministic greeting.
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    #[test]
    fn greeting_contains_name() {
        assert_eq!(super::greet("Helel"), "Hello, Helel!");
    }
}
