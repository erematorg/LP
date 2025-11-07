// Entity tracking with clean separation:
// - entity_tracker: Stores raw data (position, last_seen, metadata)
// - Specialized trackers: Read entity_tracker and evaluate (threat, prey, etc.)

pub mod entity_tracker;
pub mod needs_tracker;
pub mod perception_tracker;
pub mod prey_tracker;
pub mod threat_tracker;

use bevy::prelude::*;

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
