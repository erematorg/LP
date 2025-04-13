use bevy::prelude::*;
use crate::core::utility::UtilityScore;


/// Base trait for all AI modules
pub trait AIModule {
    /// Update the module's internal state
    fn update(&mut self);
    
    /// Calculate the utility value of this module
    fn utility(&self) -> UtilityScore;
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
}