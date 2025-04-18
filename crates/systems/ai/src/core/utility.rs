//use bevy::prelude::*; Might be needed later
use rand::prelude::*; //Gotta use Bevy_Rand as well later
use crate::prelude::*;
/// Represents a utility score for decision-making
/// Normalized between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UtilityScore(f32);

impl UtilityScore {
    /// Create a new utility score, clamping to valid range
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw score value
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Normalize a collection of scores to sum to 1.0
    pub fn normalize_scores(scores: &mut [UtilityScore]) {
        let total: f32 = scores.iter().map(|score| score.value()).sum();
        if total > 0.0 {
            for score in scores.iter_mut() {
                *score = UtilityScore::new(score.value() / total);
            }
        } else {
            // If all scores are 0, distribute evenly
            let equal_value = if scores.is_empty() { 0.0 } else { 1.0 / scores.len() as f32 };
            for score in scores.iter_mut() {
                *score = UtilityScore::new(equal_value);
            }
        }
    }

    /// Multiply the score by a factor (adjusts importance)
    pub fn multiply_by_factor(&self, factor: f32) -> Self {
        Self::new(self.0 * factor)
    }

    /// Combine two scores with AND logic (both must be true)
    /// Returns lower values as both scores must be high
    pub fn and_with(&self, other: &Self) -> Self {
        Self::new(self.0 * other.0)
    }

    /// Blend two scores with custom importance weights
    /// weight_a and weight_b should ideally sum to 1.0
    pub fn blend_weighted(&self, other: &Self, weight_a: f32, weight_b: f32) -> Self {
        Self::new(self.0 * weight_a + other.0 * weight_b)
    }

    /// Combine scores with OR logic (either can be true)
    /// P(A or B) = P(A) + P(B) - P(A and B)
    pub fn or_with(&self, other: &Self) -> Self {
        Self::new(self.0 + other.0 - (self.0 * other.0))
    }

    /// Get the opposite score (1 - score)
    /// Useful for negating or inverting priorities
    pub fn opposite(&self) -> Self {
        Self::new(1.0 - self.0)
    }

    /// Perform weighted random selection between multiple options
    pub fn weighted_select<T: Clone, R: Rng>(options: &[(T, UtilityScore)], rng: &mut R) -> Option<T> {
        let total_weight: f32 = options.iter().map(|(_, score)| score.value()).sum();
        if total_weight <= 0.0 || options.is_empty() {
            return None;
        }
        let random_point = rng.random_range(0.0..total_weight);
        let mut cumulative_weight = 0.0;
        for (option, score) in options {
            cumulative_weight += score.value();
            if random_point <= cumulative_weight {
                return Some(option.clone());
            }
        }
        // Fallback to last option (shouldn't happen with proper weights)
        options.last().map(|(opt, _)| opt.clone())
    }
}

/// Possible AI behaviors that can be selected based on utility scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Behavior {
    Idle,      // Default state, minimal activity
    Hunt,      // Pursuing prey or resource
    Flee,      // Escaping from threat
    Explore,   // Discovering new areas
    Fight,     // Engaging in combat
    Rest,      // Recovering energy
    Socialize, // Interacting with others
}

/// Selects the most appropriate behavior based on module utility scores
pub fn determine_behavior<'a>(
    modules: &[(&'a dyn AIModule, UtilityScore, Behavior)],
) -> (Behavior, UtilityScore) {
    if modules.is_empty() {
        return (Behavior::Idle, UtilityScore::new(0.0));
    }
    
    // Find highest utility module
    let (_, highest_score, best_behavior) = modules
        .iter()
        .max_by(|(_, score_a, _), (_, score_b, _)| 
            score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    
    (*best_behavior, *highest_score)
}