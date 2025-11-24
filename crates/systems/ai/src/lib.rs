pub mod arbiter;
pub mod drives;
pub mod memory;
pub mod personality;
pub mod relationships;
pub mod trackers;

use bevy::prelude::*;
use bevy::reflect::Reflect;

/// Main plugin exposed by the AI crate. Currently it installs the utility arbiter.
#[derive(Default, Debug, Clone)]
pub struct LPAIPlugin;

impl Plugin for LPAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(arbiter::UtilityArbiterPlugin);
    }
}

/// Common AI types and plugins
pub mod prelude {
    // Main plugins for easy access
    pub use crate::LPAIPlugin;
    pub use crate::arbiter::prelude::*;
    pub use crate::drives::DrivesPlugin;
    pub use crate::personality::PersonalityPlugin;
    pub use crate::relationships::SocialPlugin;
    pub use crate::trackers::TrackerPlugin;

    // Core interfaces
    pub use crate::{AIModule, ActionExecutor, Score};

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

/// Base trait for all AI modules.
pub trait AIModule: Send + Sync {
    /// Update the module's internal state.
    /// Default implementation does nothing - override if needed.
    fn update(&mut self) {}

    /// Calculate the utility value of this module (0.0 - 1.0 range).
    fn utility(&self) -> f32;
}

/// Lightweight score helper (0.0 - 1.0) replacing the legacy Big-Brain type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(PartialEq)]
pub struct Score(f32);

impl Score {
    pub const ZERO: Self = Self(0.0);
    pub const HALF: Self = Self(0.5);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn clamp_trait_value(value: f32) -> f32 {
        value.clamp(0.0, 1.0)
    }
}

/// Trait for executing actions based on behavior decisions.
pub trait ActionExecutor {
    /// Move toward a target position.
    fn move_toward(&mut self, target: Vec2, speed: f32) -> bool;

    /// Perform an attack action.
    fn attack(&mut self, target: Option<Entity>) -> bool;

    /// Move away from a threat.
    fn flee_from(&mut self, threat: Vec2) -> bool;

    /// Idle/rest at current position.
    fn idle(&mut self, duration: f32) -> bool;

    fn cleanup(&mut self);
}
