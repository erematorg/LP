use std::collections::HashMap;

/// Applies L-System rules to the axiom, returning the new string.
pub fn apply_rules(axiom: &str, rules: &HashMap<char, &str>) -> String {
    let mut result = String::new();

    for ch in axiom.chars() {
        if let Some(replacement) = rules.get(&ch) {
            result.push_str(replacement);
        } else {
            result.push(ch); // Keep unchanged if no rule exists
        }
    }

    result
}
