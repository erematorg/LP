//use bevy::prelude::*; Might be needed later
use rand::prelude::*;

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