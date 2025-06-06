use crate::prelude::*;
use bevy::prelude::*;
use energy::prelude::*;

/// Universal life adaptation traits for all organisms (plants through animals)
#[derive(Component, Debug, Clone, Reflect)]
pub struct Personality {
    /// How aggressively entity competes for resources (0.0-1.0)
    /// Plants: root/canopy competition, Animals: territorial behavior
    pub resource_assertiveness: f32,
    /// Ability to handle environmental pressure (0.0-1.0)
    /// Plants: drought/heat tolerance, Animals: threat response
    pub stress_tolerance: f32,
    /// Success in territorial/resource conflicts (0.0-1.0)
    /// Plants: canopy dominance, Animals: social hierarchy
    pub competitive_strength: f32,
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            resource_assertiveness: 0.5,
            stress_tolerance: 0.5,
            competitive_strength: 0.5,
        }
    }
}

impl Personality {
    /// Create a new personality with specified values
    pub fn new(resource_assertiveness: f32, stress_tolerance: f32, competitive_strength: f32) -> Self {
        Self {
            resource_assertiveness: resource_assertiveness.clamp(0.0, 1.0),
            stress_tolerance: stress_tolerance.clamp(0.0, 1.0),
            competitive_strength: competitive_strength.clamp(0.0, 1.0),
        }
    }

    /// How likely the entity is to compete for resources
    pub fn resource_competition_likelihood(&self) -> f32 {
        // Higher resource assertiveness and competitive strength increase competition
        self.resource_assertiveness * 0.7 + self.competitive_strength * 0.3
    }

    /// How likely the entity is to retreat from stress/threats  
    pub fn stress_retreat_likelihood(&self) -> f32 {
        // Lower stress tolerance increases retreat likelihood
        1.0 - self.stress_tolerance
    }

    /// Get base resource competition behavior utility score
    pub fn base_resource_competition_utility(&self) -> UtilityScore {
        UtilityScore::new(self.resource_competition_likelihood())
    }

    /// Get base stress retreat behavior utility score
    pub fn base_stress_retreat_utility(&self) -> UtilityScore {
        UtilityScore::new(self.stress_retreat_likelihood())
    }

    /// Get base competitive behavior utility score
    pub fn base_competitive_utility(&self) -> UtilityScore {
        UtilityScore::new(self.competitive_strength)
    }

    /// Get social behavior utility score
    pub fn social_utility(&self) -> f32 {
        // Lower resource competition and higher competitive strength increases cooperation utility
        (1.0 - self.resource_assertiveness * 0.3) + (self.competitive_strength * 0.3)
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

/// Component storing context-aware utility scores (updated by systems)
#[derive(Component, Debug, Clone, Reflect)]
pub struct ContextAwareUtilities {
    pub resource_competition: UtilityScore,
    pub stress_retreat: UtilityScore, 
    pub competitive_behavior: UtilityScore,
    pub cooperation: UtilityScore,
    /// Collective intelligence modifier from nearby entities (swarm effects)
    pub collective_influence: UtilityScore,
}

impl Default for ContextAwareUtilities {
    fn default() -> Self {
        Self {
            resource_competition: UtilityScore::new(0.5),
            stress_retreat: UtilityScore::new(0.5),
            competitive_behavior: UtilityScore::new(0.5),
            cooperation: UtilityScore::new(0.5),
            collective_influence: UtilityScore::new(0.0),
        }
    }
}

/// System that updates personality utilities based on energy and thermal state
pub fn update_context_aware_utilities(
    mut query: Query<(
        &Personality,
        &mut ContextAwareUtilities,
        Option<&EnergyQuantity>,
        Option<&EnergyAccountingLedger>,
        Option<&Temperature>,
    )>,
) {
    for (personality, mut utilities, energy_opt, ledger_opt, temp_opt) in &mut query {
        // Energy context
        let energy_level = energy_opt
            .map(|e| e.value / e.max_capacity.unwrap_or(100.0))
            .unwrap_or(0.5);
            
        let recent_success = ledger_opt
            .map(|l| l.net_energy_change() / 10.0)
            .unwrap_or(0.0)
            .clamp(-0.5, 0.5);

        // Thermal stress context
        let environmental_stress = temp_opt
            .map(|t| (t.value - 298.0).abs() / 50.0)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        // Update utilities with physics context
        utilities.resource_competition = calculate_contextual_resource_competition(
            personality, energy_level, recent_success
        );
        
        utilities.competitive_behavior = calculate_contextual_competitive_strength(
            personality, energy_level, recent_success
        );
        
        utilities.stress_retreat = calculate_contextual_stress_retreat(
            personality, energy_level, environmental_stress
        );
        
        utilities.cooperation = UtilityScore::new(personality.social_utility());
        
        // collective_influence is calculated by separate proximity-based systems
        // when entities are near each other - starts at 0.0 for isolated entities
    }
}

fn calculate_contextual_resource_competition(
    personality: &Personality, 
    energy_level: f32, 
    recent_success: f32
) -> UtilityScore {
    let energy_multiplier = 1.0 + (1.0 - energy_level) * 0.5;
    let confidence_modifier = 1.0 + recent_success.clamp(-0.3, 0.3);
    
    // SYSTEMIC AI: Add trait interdependency to calculations
    let stress_influence = 1.0 + (personality.stress_tolerance - 0.5) * 0.2;
    let competitive_influence = 1.0 + personality.competitive_strength * 0.15;
    
    let base = personality.resource_competition_likelihood();
    let result = base * energy_multiplier * confidence_modifier * stress_influence * competitive_influence;
    
    UtilityScore::new(result)
}

fn calculate_contextual_competitive_strength(
    personality: &Personality,
    energy_level: f32,
    recent_success: f32
) -> UtilityScore {
    let energy_threshold = if energy_level > 0.3 { energy_level } else { energy_level * 0.5 };
    let success_boost = 1.0 + recent_success.clamp(0.0, 0.4);
    
    UtilityScore::new(personality.competitive_strength * energy_threshold * success_boost)
}

fn calculate_contextual_stress_retreat(
    personality: &Personality,
    energy_level: f32,
    environmental_stress: f32
) -> UtilityScore {
    let energy_modifier = 1.0 + (1.0 - energy_level) * 0.4;
    let stress_modifier = 1.0 + environmental_stress * 0.6;
    
    let base = personality.stress_retreat_likelihood();
    UtilityScore::new(base * energy_modifier * stress_modifier)
}
