use crate::core::scorers::Score;
use crate::prelude::AIModule;
use bevy::prelude::*;
use std::collections::HashMap;

use super::perception_tracker::Perception;

/// Configuration resource for threat tracker default parameters
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ThreatTrackerConfig {
    /// Default severity decay applied per second
    pub default_decay_per_second: f32,
    /// Default time (in seconds) before completely forgetting unseen threats
    pub default_forget_after: f32,
}

impl Default for ThreatTrackerConfig {
    fn default() -> Self {
        Self {
            default_decay_per_second: 0.4,
            default_forget_after: 6.0,
        }
    }
}

impl ThreatTrackerConfig {
    pub fn new(decay_per_second: f32, forget_after: f32) -> Self {
        Self {
            default_decay_per_second: decay_per_second.max(0.0),
            default_forget_after: forget_after.max(0.0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct ThreatEntry {
    severity: f32,
    last_seen_time: f32,
}

/// Tracks recent threats seen by an entity and provides a panic utility.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ThreatTracker {
    #[reflect(ignore)]
    threats: HashMap<Entity, ThreatEntry>,
    /// Cached overall panic level (0.0-1.0).
    pub panic_level: f32,
    /// Sum of individual threat severities (0.0-1.0).
    pub accumulated_threat: f32,
    /// Severity decay applied per second.
    pub decay_per_second: f32,
    /// Time (in seconds) before completely forgetting unseen threats.
    pub forget_after: f32,
    #[reflect(ignore)]
    last_update_time: Option<f32>,
}

impl Default for ThreatTracker {
    fn default() -> Self {
        Self::new(0.4, 6.0)
    }
}

impl ThreatTracker {
    pub fn new(decay_per_second: f32, forget_after: f32) -> Self {
        Self {
            threats: HashMap::with_capacity(8), // Typical organisms track few simultaneous threats
            panic_level: 0.0,
            accumulated_threat: 0.0,
            decay_per_second: decay_per_second.max(0.0),
            forget_after: forget_after.max(0.0),
            last_update_time: None,
        }
    }

    /// Create a new ThreatTracker from configuration resource
    pub fn from_config(config: &ThreatTrackerConfig) -> Self {
        Self::new(
            config.default_decay_per_second,
            config.default_forget_after,
        )
    }

    fn decay_entries(&mut self, now: f32) {
        if let Some(last) = self.last_update_time {
            let delta = (now - last).max(0.0);
            if delta > 0.0 && !self.threats.is_empty() {
                let decay_amount = self.decay_per_second * delta;
                self.threats.retain(|_, entry| {
                    let time_since_seen = now - entry.last_seen_time;
                    if time_since_seen > self.forget_after {
                        return false;
                    }

                    let new_severity = (entry.severity - decay_amount).max(0.0);
                    entry.severity = new_severity;
                    new_severity > 0.01
                });
            }
        }

        self.last_update_time = Some(now);
    }

    fn register_threat(&mut self, entity: Entity, severity: f32, now: f32) {
        let entry = self.threats.entry(entity).or_insert(ThreatEntry {
            severity: 0.0,
            last_seen_time: now,
        });
        entry.severity = entry.severity.max(severity.clamp(0.0, 1.0));
        entry.last_seen_time = now;
    }

    fn recalculate(&mut self) {
        if self.threats.is_empty() {
            self.panic_level = 0.0;
            self.accumulated_threat = 0.0;
            return;
        }

        let mut total = 0.0;
        for entry in self.threats.values() {
            total += entry.severity;
        }

        self.accumulated_threat = total.clamp(0.0, 4.0);
        // Convert accumulated threat to a 0.0-1.0 panic using smooth exponential saturation.
        // This mimics real organism stress response: rises quickly with threat, decays naturally.
        // The exponential curve (1 - e^(-x)) provides biologically realistic behavior:
        // - Rapid response to new threats
        // - Natural saturation at high threat levels
        // - Smooth recovery as threats decay
        // No artificial floor - panic tracks accumulated threat naturally like real stress hormones.
        self.panic_level = (1.0 - (-self.accumulated_threat).exp()).clamp(0.0, 1.0);
    }

    /// Update this tracker using the current perception information.
    pub fn update_from_perception(&mut self, perception: &Perception, now: f32) {
        self.decay_entries(now);

        if perception.detection_radius <= 0.0 {
            self.recalculate();
            return;
        }

        for (entity, _pos, distance) in &perception.visible_entities {
            let severity = 1.0 - (distance / perception.detection_radius).clamp(0.0, 1.0);
            if severity > 0.0 {
                self.register_threat(*entity, severity, now);
            }
        }

        self.recalculate();
    }
}

impl AIModule for ThreatTracker {
    fn update(&mut self) {
        // Temporal decay is handled by [`update_from_perception`]; nothing to do per tick.
    }

    fn utility(&self) -> Score {
        Score::new(self.panic_level)
    }
}

/// Integrates [`ThreatTracker`] with [`Perception`] every frame.
pub fn threat_tracker_system(
    time: Res<Time>,
    mut query: Query<(&Perception, &mut ThreatTracker)>,
) {
    let now = time.elapsed_secs();
    for (perception, mut tracker) in &mut query {
        tracker.update_from_perception(perception, now);
    }
}
