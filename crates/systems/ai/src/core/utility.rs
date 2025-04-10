use bevy::prelude::*;

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
}