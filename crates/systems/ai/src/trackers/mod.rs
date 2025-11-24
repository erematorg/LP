// Entity tracking with clean separation:
// - entity_tracker: Stores raw data (position, last_seen, metadata)
// - Specialized trackers: Read entity_tracker and evaluate (threat, prey, etc.)

pub mod entity_tracker;
pub mod needs_tracker;
pub mod perception_tracker;
pub mod prey_tracker;
pub mod threat_tracker;

use bevy::prelude::*;
use entity_tracker::{EntityMetadata, EntityTracker};

/// Helper function for common tracker evaluation pattern with time decay and distance factors.
///
/// This consolidates the shared logic between ThreatTracker, PreyTracker, and future trackers.
/// Returns an iterator of (Entity, evaluated_score) pairs.
///
/// # Parameters
/// - `entity_tracker`: The entity tracker to read from
/// - `current_time`: Current simulation time for decay calculation
/// - `decay_rate`: How quickly the value decays per second (exponential)
/// - `max_distance`: Distance beyond which the value becomes zero (linear)
/// - `extract_value`: Closure that extracts the base value from EntityMetadata (returns None if wrong type)
pub(crate) fn evaluate_tracked_entities_with_decay<'a, F>(
    entity_tracker: &'a EntityTracker,
    current_time: f32,
    decay_rate: f32,
    max_distance: f32,
    extract_value: F,
) -> impl Iterator<Item = (Entity, f32)> + 'a
where
    F: Fn(&EntityMetadata) -> Option<f32> + 'a,
{
    entity_tracker.all().filter_map(move |tracked| {
        let base_value = extract_value(&tracked.metadata)?;

        // Time-based exponential decay
        let time_since = tracked.time_since_seen(current_time);
        let decay = (-decay_rate * time_since).exp();
        let decayed_value = base_value * decay;

        // Distance-based linear factor (closer = higher score)
        let distance_factor = if tracked.last_distance > 0.0 {
            1.0 - (tracked.last_distance / max_distance).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let final_score = decayed_value * distance_factor;
        Some((tracked.entity, final_score))
    })
}

/// Plugin for entity tracking systems
#[derive(Default)]
pub struct TrackerPlugin;

impl Plugin for TrackerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<threat_tracker::ThreatConfig>()
            .init_resource::<prey_tracker::PreyConfig>()
            .register_type::<threat_tracker::ThreatConfig>()
            .register_type::<prey_tracker::PreyConfig>()
            .add_systems(
                Update,
                (
                    threat_tracker::threat_tracker_system,
                    prey_tracker::prey_tracker_system,
                ),
            );
    }
}

/// Prelude for the trackers module
pub mod prelude {
    pub use crate::trackers::TrackerPlugin;

    // Data storage
    pub use crate::trackers::entity_tracker::{EntityMetadata, EntityTracker, TrackedEntity};

    // Evaluation trackers
    pub use crate::trackers::prey_tracker::{PreyConfig, PreyTracker};
    pub use crate::trackers::threat_tracker::{ThreatConfig, ThreatTracker};

    // Other trackers
    pub use crate::trackers::needs_tracker::NeedsTracker;
    pub use crate::trackers::perception_tracker::Perception;
}

// Future trackers planned (add as LP grows):
//
// aggression_tracker.rs
//   Evaluates anger/hostility from entity_tracker
//   Used for: Territorial defense, competitive behavior
//
// social_tracker.rs
//   Evaluates pack bonds, herd affiliation from entity_tracker
//   Used for: Pack hunting, colony cooperation, herd movement
//
// territory_tracker.rs
//   Evaluates familiarity with locations (dens, nesting sites)
//   Used for: Migration routes, home territory, safe zones
//
// noise_tracker.rs
//   Evaluates sound sources and their significance
//   Used for: Predator detection, communication
//
// obstacle_tracker.rs
//   Evaluates navigation obstacles and path quality
//   Used for: Pathfinding assistance, stuck detection
//
// injury_tracker.rs
//   Evaluates damage state and healing needs
//   Used for: Retreat behavior, vulnerability assessment
