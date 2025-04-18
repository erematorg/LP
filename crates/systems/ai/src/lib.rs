pub mod core;
pub mod drives;
pub mod memory;
pub mod personality;
pub mod relationships;
pub mod trackers;

/// The AI prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    // Core interfaces
    pub use crate::core::interfaces::{AIModule, ActionExecutor};
    pub use crate::core::utility::{Behavior, UtilityScore, determine_behavior};
    
    // New big-brain inspired components
    pub use crate::core::scorers::{Score, Scorer, ScorerContext};
    pub use crate::core::actions::{Action, ActionContext, ActionState};
    pub use crate::core::thinkers::{Thinker, Choice, ActionType, Picker, FirstToScore, Highest, HasThinker};
    
    // Trackers
    pub use crate::trackers::perception_tracker::Perception;
    pub use crate::trackers::needs_tracker::NeedsTracker;
    pub use crate::trackers::base_tracker::EntityTracker;
    
    // Memory system
    pub use crate::memory::types::{MemoryEvent, MemoryEventType, MemoryTimestamp};
    
    // Personality and relationships
    pub use crate::personality::traits::Personality;
    pub use crate::relationships::social::{SocialNetwork, RelationshipType, RelationshipStrength};
    
    // Needs and drives
    pub use crate::drives::needs::{Need, NeedType};
}