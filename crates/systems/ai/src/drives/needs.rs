use crate::core::scorers::Score;
use crate::prelude::*;
use bevy::prelude::*;

/// Universal need types that apply to all life forms
/// These represent fundamental biological drives that emerge from physics and chemistry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum NeedType {
    /// Need for energy to sustain life
    /// - Animals: food consumption
    /// - Plants: sunlight absorption
    /// - Fungi: nutrient decomposition
    /// - Microbes: chemical energy sources
    Energy,

    /// Need for essential resources (water, minerals, nutrients)
    /// Universal across all life forms
    Resources,

    /// Need to avoid threats and maintain safety
    /// - Animals: predators, environmental hazards
    /// - Plants: herbivores, drought, fire
    /// - All: temperature extremes, toxins
    Safety,

    /// Need to maintain stable internal state (homeostasis)
    /// - Temperature regulation
    /// - pH balance
    /// - Osmotic pressure
    /// Universal physiological stability
    Homeostasis,

    /// Biological drive to reproduce and pass on genetic information
    /// Universal across all life forms
    Reproduction,
}

/// Component representing a single need
#[derive(Component, Debug, Clone, Reflect)]
pub struct Need {
    /// Type of need
    pub need_type: NeedType,
    /// Current satiation level (0.0 = completely unsatisfied, 1.0 = fully satisfied)
    pub satisfaction: f32,
    /// How quickly this need depletes (higher = faster)
    pub depletion_rate: f32,
    /// Relative importance of this need
    pub priority: f32,
}

impl Need {
    pub fn new(need_type: NeedType, satisfaction: f32, depletion_rate: f32, priority: f32) -> Self {
        Self {
            need_type,
            satisfaction: Score::clamp_trait_value(satisfaction),
            depletion_rate: depletion_rate.max(0.0),
            priority: Score::clamp_trait_value(priority),
        }
    }

    /// Apply depletion based on elapsed time
    pub fn decay(&mut self, delta_secs: f32) {
        if delta_secs <= 0.0 || self.depletion_rate <= 0.0 {
            return;
        }

        self.satisfaction = (self.satisfaction - self.depletion_rate * delta_secs).max(0.0);
    }

    /// Calculate need urgency as utility score
    pub fn urgency(&self) -> Score {
        Score::new((1.0 - self.satisfaction) * self.priority)
    }

    /// Satisfy this need by the given amount
    pub fn satisfy(&mut self, amount: f32) {
        self.satisfaction = (self.satisfaction + amount).min(1.0);
    }
}

/// System for updating needs over time
pub fn update_needs(time: Res<Time>, mut needs: Query<&mut Need>) {
    for mut need in &mut needs {
        need.decay(time.delta_secs());
    }
}

/// System for selecting most urgent need
pub fn get_most_urgent_need(entity: Entity, needs: Query<&Need>) -> Option<(NeedType, Score)> {
    let mut most_urgent = None;
    let mut highest_urgency = Score::ZERO;

    for need in needs.iter_many(std::iter::once(entity)) {
        let urgency = need.urgency();
        if urgency.value() > highest_urgency.value() {
            most_urgent = Some(need.need_type);
            highest_urgency = urgency;
        }
    }

    most_urgent.map(|need_type| (need_type, highest_urgency))
}

impl AIModule for Need {
    fn update(&mut self) {
        // Clamp to valid range; depletion is handled by systems with proper delta time
        self.satisfaction = self.satisfaction.clamp(0.0, 1.0);
    }

    fn utility(&self) -> Score {
        // Return urgency as utility
        self.urgency()
    }
}
