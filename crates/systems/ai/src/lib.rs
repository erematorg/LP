pub mod core;
pub mod drives;
pub mod memory;
pub mod personality;
pub mod relationships;
pub mod trackers;

/// Core AI plugin with utility-based decision making
pub use crate::core::LPAIPlugin;

/// Common AI types and plugins
pub mod prelude {
    // Main plugins for easy access
    pub use crate::LPAIPlugin;
    pub use crate::drives::DrivesPlugin;
    pub use crate::personality::PersonalityPlugin;
    pub use crate::relationships::SocialPlugin;
    pub use crate::trackers::TrackerPlugin;

    // Core interfaces - now directly from crate root
    pub use crate::{AIModule, ActionExecutor};

    // Re-export module preludes
    pub use crate::core::prelude::*;
    pub use crate::drives::prelude::*;
    pub use crate::memory::prelude::*;
    pub use crate::personality::prelude::*;
    pub use crate::relationships::prelude::*;
    pub use crate::trackers::prelude::*;

    // Context-aware personality system
    pub use crate::personality::traits::{
        ContextAwareUtilities, PersonalityContextInputs, update_collective_influence,
        update_context_aware_utilities,
    };
}

use crate::core::scorers::Score;
use bevy::prelude::*;

/// Base trait for all AI modules
pub trait AIModule: Send + Sync {
    /// Update the module's internal state
    fn update(&mut self);

    /// Calculate the utility value of this module
    fn utility(&self) -> Score;
}

/// Trait for executing actions based on behavior decisions
pub trait ActionExecutor {
    /// Move toward a target position
    fn move_toward(&mut self, target: Vec2, speed: f32) -> bool;

    /// Perform an attack action
    fn attack(&mut self, target: Option<Entity>) -> bool;

    /// Move away from a threat  
    fn flee_from(&mut self, threat: Vec2) -> bool;

    /// Idle/rest at current position
    fn idle(&mut self, duration: f32) -> bool;

    fn cleanup(&mut self);
}
