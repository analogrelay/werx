//! Agent name generation using Docker-style naming.
//!
//! Generates human-readable names by combining adjectives and nouns
//! in lowercase_snakecase format.

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::namedata::{ADJECTIVES, NOUNS};

/// Generate a unique agent name in lowercase_snakecase format.
///
/// Names are generated Docker-style by combining a random adjective
/// with a random noun (e.g., "arcane_aegis", "blazing_beacon").
///
/// # Arguments
///
/// * `existing_names` - A slice of existing agent names to avoid collisions
///
/// # Returns
///
/// A unique name that doesn't collide with any existing names.
/// Will retry up to 100 times before giving up and appending a number.
pub fn generate_agent_name(existing_names: &[String]) -> String {
    let mut rng = thread_rng();

    for _ in 0..100 {
        let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"unknown");
        let noun = NOUNS.choose(&mut rng).unwrap_or(&"agent");

        // Convert to lowercase_snakecase
        let name = format!("{}_{}", to_snake_case(adjective), to_snake_case(noun));

        if !existing_names.contains(&name) {
            return name;
        }
    }

    // Fallback: append a number to make it unique
    let base_adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"unknown");
    let base_noun = NOUNS.choose(&mut rng).unwrap_or(&"agent");
    let base_name = format!(
        "{}_{}",
        to_snake_case(base_adjective),
        to_snake_case(base_noun)
    );

    let mut counter = 1;
    loop {
        let name = format!("{}_{}", base_name, counter);
        if !existing_names.contains(&name) {
            return name;
        }
        counter += 1;
    }
}

/// Convert a PascalCase or Title Case string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut chars = s.chars().peekable();
    let mut prev_was_separator = false;

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() && !prev_was_separator {
                // Only add underscore if next char isn't also uppercase
                // or if it's the last uppercase in a sequence
                let next_is_lower = chars.peek().map_or(false, |next| next.is_lowercase());
                if next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
            prev_was_separator = false;
        } else if c == ' ' || c == '-' {
            if !result.is_empty() && !prev_was_separator {
                result.push('_');
            }
            prev_was_separator = true;
        } else {
            result.push(c.to_ascii_lowercase());
            prev_was_separator = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_simple() {
        assert_eq!(to_snake_case("Arcane"), "arcane");
        assert_eq!(to_snake_case("Blazing"), "blazing");
    }

    #[test]
    fn test_to_snake_case_already_lowercase() {
        assert_eq!(to_snake_case("arcane"), "arcane");
    }

    #[test]
    fn test_to_snake_case_with_spaces() {
        assert_eq!(to_snake_case("Title Case"), "title_case");
    }

    #[test]
    fn test_generate_agent_name_format() {
        let name = generate_agent_name(&[]);
        assert!(
            name.contains('_'),
            "Name should contain underscore: {}",
            name
        );
        assert!(
            name.chars().all(|c| c.is_lowercase() || c == '_'),
            "Name should be lowercase_snakecase: {}",
            name
        );
    }

    #[test]
    fn test_generate_agent_name_avoids_collisions() {
        let existing = vec!["arcane_aegis".to_string()];

        // Generate many names to ensure we don't get the existing one
        for _ in 0..50 {
            let name = generate_agent_name(&existing);
            assert_ne!(name, "arcane_aegis", "Should avoid collision");
        }
    }

    #[test]
    fn test_generate_agent_name_unique() {
        let mut names = Vec::new();
        for _ in 0..20 {
            let name = generate_agent_name(&names);
            assert!(!names.contains(&name), "Generated duplicate name: {}", name);
            names.push(name);
        }
    }
}
