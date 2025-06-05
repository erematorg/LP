pub mod base_tracker;
pub mod needs_tracker;
pub mod perception_tracker;

//TODO soon: pub mod relationship_tracker;

/// Prelude for the trackers module.
///
/// This includes the most common tracking components and systems.
pub mod prelude {
    pub use crate::trackers::base_tracker::{EntityTracker, TrackedEntity};
    pub use crate::trackers::needs_tracker::NeedsTracker;
    pub use crate::trackers::perception_tracker::Perception;
}
