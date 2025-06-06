use crate::prelude::*;
use bevy::prelude::*;

/// Core personality traits for AI entities
#[derive(Component, Debug, Clone, Reflect)]
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

/// Component marking an entity as having altruistic tendencies
/// Universal trait applicable to any creature type in LP ecosystem
#[derive(Component, Debug, Clone, Reflect)]
pub struct Altruistic {
    /// Strength of altruistic behavior (0.0-1.0)
    pub strength: f32,
    /// Hunger threshold below which altruism is active (0.0-1.0) 
    pub activation_threshold: f32,
}

impl Default for Altruistic {
    fn default() -> Self {
        Self {
            strength: 0.7,
            activation_threshold: 0.7, // Only altruistic when hunger < 70%
        }
    }
}

impl Altruistic {
    pub fn new(strength: f32, activation_threshold: f32) -> Self {
        Self {
            strength: strength.clamp(0.0, 1.0),
            activation_threshold: activation_threshold.clamp(0.0, 1.0),
        }
    }

    /// Check if entity should exhibit altruistic behavior given current hunger
    pub fn should_be_altruistic(&self, hunger_level: f32) -> bool {
        hunger_level < self.activation_threshold
    }

    /// Get altruistic behavior utility score
    pub fn altruistic_utility(&self, hunger_level: f32) -> UtilityScore {
        if self.should_be_altruistic(hunger_level) {
            UtilityScore::new(self.strength)
        } else {
            UtilityScore::new(0.0)
        }
    }
}
