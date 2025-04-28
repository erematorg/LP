use super::grammar;
use std::collections::HashMap;

pub fn generate(axiom: &str, rules: &HashMap<char, &str>, iterations: usize) -> String {
    let mut current = axiom.to_string();

    for _i in 0..iterations {
        current = grammar::apply_rules(&current, rules);
        // println!("Iteration {}: {}", _i, current);
    }

    // println!("Final Iteration {}: {}", iterations, current);
    current
}
