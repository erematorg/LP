use bevy::prelude::*;
use crate::prelude::*;


/// Core need types representing basic drives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeedType {
    Hunger,     // Need for nourishment
    Safety,     // Need to avoid danger
    Rest,       // Need for recuperation
    Social,     // Need for group interaction
}

/// Component representing a single need
#[derive(Component, Debug, Clone)]
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
            satisfaction: satisfaction.clamp(0.0, 1.0),
            depletion_rate: depletion_rate.max(0.0),
            priority: priority.clamp(0.0, 1.0),
        }
    }

    /// Calculate need urgency as utility score
    pub fn urgency(&self) -> UtilityScore {
        UtilityScore::new((1.0 - self.satisfaction) * self.priority)
    }

    /// Satisfy this need by the given amount
    pub fn satisfy(&mut self, amount: f32) {
        self.satisfaction = (self.satisfaction + amount).min(1.0);
    }
}

/// System for updating needs over time
pub fn update_needs(
    time: Res<Time>,
    mut needs: Query<&mut Need>,
) {
    for mut need in &mut needs {
        need.satisfaction = (need.satisfaction - need.depletion_rate * time.delta_secs()).max(0.0);
    }
}

/// System for selecting most urgent need
pub fn get_most_urgent_need(
    entity: Entity,
    needs: Query<&Need>,
) -> Option<(NeedType, UtilityScore)> {
    let mut most_urgent = None;
    let mut highest_urgency = UtilityScore::new(0.0);

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
        // Naturally decrease satisfaction over time
        // (Note: Full implementation would use time delta)
        self.satisfaction = (self.satisfaction - self.depletion_rate * 0.01).max(0.0);
    }
    
    fn utility(&self) -> UtilityScore {
        // Return urgency as utility
        self.urgency()
    }
}

//TODO: Adding a NeedTracker to monitor and manage needs over time, would be in systems/ai/src/trackers/needs_tracker.rs