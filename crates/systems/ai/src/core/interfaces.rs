//use bevy::prelude::*;
use crate::core::utility::UtilityScore;

/// Base trait for all AI modules
pub trait AIModule {
    /// Update the module's internal state
    fn update(&mut self);
    
    /// Calculate the utility value of this module
    fn utility(&self) -> UtilityScore;

}