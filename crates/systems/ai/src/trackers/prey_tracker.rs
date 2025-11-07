//! Prey evaluation - reads entity tracker, outputs food assessment
//!
//! Data storage separate from evaluation.
//! This reads EntityTracker and calculates food attractiveness.

use super::entity_tracker::{EntityMetadata, EntityTracker};
use crate::core::scorers::Score;
use crate::prelude::*;
use bevy::prelude::*;

/// Configuration for prey evaluation
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct PreyConfig {
    /// How quickly food memory fades per second
    pub memory_decay_per_second: f32,

    /// Time before completely forgetting food
    pub forget_after: f32,

    /// Max distance to consider food attractive
    pub max_attractive_distance: f32,
}

impl Default for PreyConfig {
    fn default() -> Self {
        Self {
            memory_decay_per_second: 0.1,
            forget_after: 10.0,
            max_attractive_distance: 200.0,
        }
    }
}

/// Evaluates food sources from entity tracker (no data storage)
#[derive(Component, Debug, Default)]
pub struct PreyTracker {
    /// Most attractive prey entity
    best_prey: Option<Entity>,

    /// Attractiveness of best prey (0.0-1.0)
    best_attractiveness: f32,
}

impl PreyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get most attractive prey
    pub fn best_prey(&self) -> Option<Entity> {
        self.best_prey
    }

    /// Get attractiveness of best prey
    pub fn best_attractiveness(&self) -> f32 {
        self.best_attractiveness
    }

    /// Update prey evaluation from entity tracker
    pub fn update(
        &mut self,
        entity_tracker: &EntityTracker,
        current_time: f32,
        config: &PreyConfig,
    ) {
        let mut best_entity = None;
        let mut best_score = 0.0;

        // Evaluate all prey from entity tracker
        for tracked in entity_tracker.filter_by_metadata(|m| {
            matches!(m, EntityMetadata::Prey { .. })
        }) {
            if let EntityMetadata::Prey { attractiveness } = tracked.metadata {
                // Apply memory decay
                let time_since = tracked.time_since_seen(current_time);
                let decay = (-config.memory_decay_per_second * time_since).exp();
                let current_attractiveness = attractiveness * decay;

                // Distance factor (closer = more attractive)
                let distance_factor = if tracked.last_distance > 0.0 {
                    1.0 - (tracked.last_distance / config.max_attractive_distance)
                        .clamp(0.0, 1.0)
                } else {
                    1.0
                };

                let total_score = current_attractiveness * distance_factor;

                if total_score > best_score {
                    best_score = total_score;
                    best_entity = Some(tracked.entity);
                }
            }
        }

        self.best_prey = best_entity;
        self.best_attractiveness = best_score;
    }
}

impl AIModule for PreyTracker {
    fn update(&mut self) {
        // Update happens in system with access to EntityTracker
    }

    fn utility(&self) -> Score {
        Score::new(self.best_attractiveness)
    }
}

/// System that updates all prey trackers
pub fn prey_tracker_system(
    time: Res<Time>,
    config: Res<PreyConfig>,
    mut query: Query<(&mut PreyTracker, &EntityTracker)>,
) {
    let current_time = time.elapsed_secs();

    for (mut prey_tracker, entity_tracker) in &mut query {
        prey_tracker.update(entity_tracker, current_time, &config);
    }
}
