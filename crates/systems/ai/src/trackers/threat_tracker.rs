//! Threat evaluation - reads entity tracker, outputs threat assessment
//!
//! Data storage separate from evaluation.
//! This reads EntityTracker and calculates threat levels.

use super::entity_tracker::{EntityMetadata, EntityTracker};
use crate::core::scorers::Score;
use crate::prelude::*;
use bevy::prelude::*;

/// Configuration for threat evaluation
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ThreatConfig {
    /// How quickly threat severity decays per second
    pub decay_per_second: f32,

    /// Time before completely forgetting a threat
    pub forget_after: f32,

    /// Distance at which threats are max severity (closer = worse)
    pub max_severity_distance: f32,
}

impl Default for ThreatConfig {
    fn default() -> Self {
        Self {
            decay_per_second: 0.4,
            forget_after: 6.0,
            max_severity_distance: 50.0,
        }
    }
}

/// Evaluates threats from entity tracker (no data storage)
#[derive(Component, Debug, Default)]
pub struct ThreatTracker {
    /// Cached panic level (recalculated each frame)
    panic_level: f32,

    /// Highest individual threat severity
    highest_threat: f32,
}

impl ThreatTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current panic level (0.0-1.0)
    pub fn panic_level(&self) -> f32 {
        self.panic_level
    }

    /// Get highest individual threat
    pub fn highest_threat(&self) -> f32 {
        self.highest_threat
    }

    /// Update threat evaluation from entity tracker
    pub fn update(
        &mut self,
        entity_tracker: &EntityTracker,
        current_time: f32,
        config: &ThreatConfig,
    ) {
        let mut total_threat = 0.0;
        let mut max_threat: f32 = 0.0;

        // Evaluate all threats from entity tracker
        for tracked in entity_tracker.filter_by_metadata(|m| {
            matches!(m, EntityMetadata::Threat { .. })
        }) {
            if let EntityMetadata::Threat { severity } = tracked.metadata {
                // Apply decay based on time since seen
                let time_since = tracked.time_since_seen(current_time);
                let decay = (-config.decay_per_second * time_since).exp();
                let current_severity = severity * decay;

                // Distance modifier (closer = more threatening)
                let distance_factor = if tracked.last_distance > 0.0 {
                    1.0 - (tracked.last_distance / config.max_severity_distance).clamp(0.0, 1.0)
                } else {
                    1.0
                };

                let adjusted_severity = current_severity * distance_factor;

                total_threat += adjusted_severity;
                max_threat = max_threat.max(adjusted_severity);
            }
        }

        // Exponential panic saturation (from original ThreatTracker)
        let accumulated = total_threat.clamp(0.0, 4.0);
        self.panic_level = (1.0 - (-accumulated).exp()).clamp(0.0, 1.0);
        self.highest_threat = max_threat;
    }
}

impl AIModule for ThreatTracker {
    fn update(&mut self) {
        // Update happens in system with access to EntityTracker
    }

    fn utility(&self) -> Score {
        Score::new(self.panic_level)
    }
}

/// System that updates all threat trackers
pub fn threat_tracker_system(
    time: Res<Time>,
    config: Res<ThreatConfig>,
    mut query: Query<(&mut ThreatTracker, &EntityTracker)>,
) {
    let current_time = time.elapsed_secs();

    for (mut threat_tracker, entity_tracker) in &mut query {
        threat_tracker.update(entity_tracker, current_time, &config);
    }
}
