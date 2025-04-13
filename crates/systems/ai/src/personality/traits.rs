use bevy::prelude::*;

/// Core personality traits for AI entities
#[derive(Component, Debug, Clone)]
pub struct Personality {
    /// Affects aggression and attack likelihood (0.0-1.0)
    pub aggression: f32,
    /// Determines fear response threshold (0.0-1.0)
    pub bravery: f32,
    /// Affects social hierarchy position (0.0-1.0)
    pub dominance: f32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            bravery: 0.5,
            dominance: 0.5,
        }
    }
}

impl Personality {
    /// Create a new personality with specified values
    pub fn new(aggression: f32, bravery: f32, dominance: f32) -> Self {
        Self {
            aggression: aggression.clamp(0.0, 1.0),
            bravery: bravery.clamp(0.0, 1.0),
            dominance: dominance.clamp(0.0, 1.0),
        }
    }
    
    /// How likely the entity is to attack
    pub fn attack_likelihood(&self) -> f32 {
        // Higher aggression and bravery increase attack likelihood
        self.aggression * 0.7 + self.bravery * 0.3
    }
    
    /// How likely the entity is to flee from threats
    pub fn flee_likelihood(&self) -> f32 {
        // Lower bravery increases flee likelihood
        1.0 - self.bravery
    }
}