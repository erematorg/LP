pub mod base_tracker;
pub mod needs_tracker;
pub mod perception_tracker;
pub mod threat_tracker;

//TODO soon: pub mod relationship_tracker;

use bevy::prelude::*;

/// Plugin for entity tracking systems
#[derive(Default)]
pub struct TrackerPlugin;

impl Plugin for TrackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<threat_tracker::ThreatTracker>()
            .add_systems(Update, threat_tracker::threat_tracker_system);
    }
}

/// Prelude for the trackers module.
///
/// This includes the most common tracking components and systems.
pub mod prelude {
    pub use crate::trackers::TrackerPlugin;
    pub use crate::trackers::base_tracker::{EntityTracker, TrackedEntity, TrackingRelation};
    pub use crate::trackers::needs_tracker::NeedsTracker;
    pub use crate::trackers::perception_tracker::Perception;
    pub use crate::trackers::threat_tracker::{threat_tracker_system, ThreatTracker};
}
