use bevy::prelude::*;
use crate::prelude::*;

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
    
    /// Get attack behavior utility score
    pub fn attack_utility(&self) -> UtilityScore {
        UtilityScore::new(self.attack_likelihood())
    }
    
    /// Get flee behavior utility score
    pub fn flee_utility(&self) -> UtilityScore {
        UtilityScore::new(self.flee_likelihood())
    }
    
    /// Get dominance behavior utility score
    pub fn dominance_utility(&self) -> UtilityScore {
        UtilityScore::new(self.dominance)
    }
    
    /// Get social behavior utility score
    pub fn social_utility(&self) -> f32 {
        // Lower aggression and higher dominance increases social interaction utility
        (1.0 - self.aggression * 0.3) + (self.dominance * 0.3)
    }
}

impl AIModule for Personality {
    fn update(&mut self) {
        // Personality traits are generally stable
        // but could evolve slowly based on experiences
    }
    
    fn utility(&self) -> UtilityScore {
        // Return a base utility score for personality-driven behaviors
        UtilityScore::new(0.5)
    }
}