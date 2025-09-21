use super::generator;
use std::collections::HashMap;

/// Handles rule management for L-Systems.
pub struct RuleManager<'a> {
    rules: HashMap<char, &'a str>,
}

impl Default for RuleManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RuleManager<'a> {
    /// Create a new RuleManager.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a rule to the manager.
    pub fn add_rule(&mut self, symbol: char, replacement: &'a str) {
        if replacement.is_empty() {
            panic!("Replacement rule cannot be empty for '{}'", symbol);
        }
        self.rules.insert(symbol, replacement);
    }

    /// Get the rules as an immutable reference.
    pub fn get_rules(&self) -> &HashMap<char, &'a str> {
        &self.rules
    }
}

/// Represents an L-System with an axiom and a set of rules.
pub struct LSystem<'a> {
    axiom: &'a str,
    rules_manager: RuleManager<'a>,
    iterations: usize,
}

impl<'a> LSystem<'a> {
    /// Create a new L-System with the given axiom.
    pub fn new(axiom: &'a str) -> Self {
        Self {
            axiom,
            rules_manager: RuleManager::new(),
            iterations: 1,
        }
    }

    /// Add a rule to the L-System.
    pub fn add_rule(mut self, symbol: char, replacement: &'a str) -> Self {
        self.rules_manager.add_rule(symbol, replacement);
        self
    }

    /// Set the number of iterations for the L-System.
    pub fn set_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// Generate the L-System string based on the current configuration.
    pub fn generate(&self) -> String {
        generator::generate(self.axiom, self.rules_manager.get_rules(), self.iterations)
    }
}
