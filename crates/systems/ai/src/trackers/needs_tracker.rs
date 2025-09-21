use crate::prelude::*;
use bevy::prelude::*;

/// Tracks and manages needs for an entity
#[derive(Component, Default)]
pub struct NeedsTracker {
    needs: Vec<Need>,
    most_urgent_need: Option<(NeedType, UtilityScore)>,
}

impl NeedsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_need(&mut self, need: Need) {
        self.needs.push(need);
    }

    pub fn get_needs(&self) -> &[Need] {
        &self.needs
    }

    pub fn get_need_mut(&mut self, need_type: NeedType) -> Option<&mut Need> {
        self.needs
            .iter_mut()
            .find(|need| need.need_type == need_type)
    }

    pub fn get_most_urgent_need(&self) -> Option<(NeedType, UtilityScore)> {
        self.most_urgent_need
    }
}

impl AIModule for NeedsTracker {
    fn update(&mut self) {
        // Update all needs
        for need in &mut self.needs {
            need.update();
        }

        // Find most urgent need
        let mut most_urgent = None;
        let mut highest_urgency = UtilityScore::ZERO;

        for need in &self.needs {
            let urgency = need.urgency();
            if urgency.value() > highest_urgency.value() {
                most_urgent = Some(need.need_type);
                highest_urgency = urgency;
            }
        }

        self.most_urgent_need = most_urgent.map(|need_type| (need_type, highest_urgency));
    }

    fn utility(&self) -> UtilityScore {
        // Return the urgency of the most urgent need, or zero if no needs
        self.most_urgent_need
            .map(|(_, urgency)| urgency)
            .unwrap_or(UtilityScore::ZERO)
    }
}
