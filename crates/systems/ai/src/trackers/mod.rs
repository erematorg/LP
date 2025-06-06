pub mod base_tracker;
pub mod needs_tracker;
pub mod perception_tracker;

//TODO soon: pub mod relationship_tracker;

use bevy::prelude::*;

/// Plugin for entity tracking systems
#[derive(Default)]
pub struct TrackerPlugin;

impl Plugin for TrackerPlugin {
    fn build(&self, _app: &mut App) {
        // Simple plugin - just makes trackers available
        // Systems will be added later when we have proper integration
    }
}


/// Prelude for the trackers module.
///
/// This includes the most common tracking components and systems.
pub mod prelude {
    pub use crate::trackers::base_tracker::{EntityTracker, TrackedEntity, TrackingRelation};
    pub use crate::trackers::needs_tracker::NeedsTracker;
    pub use crate::trackers::perception_tracker::Perception;
    pub use crate::trackers::TrackerPlugin;
}
