pub mod core;

use bevy::prelude::*;

/// Force application interface for physical forces
pub trait ForceApplicator: Send + Sync {
    /// Apply a force to an entity
    fn apply_force(&self, entity: Entity, force: Vec3);
    
    /// Get the force magnitude
    fn get_magnitude(&self) -> f32;
    
    /// Get the force direction
    fn get_direction(&self) -> Vec3;
}

/// The forces prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    // Core interfaces from crate root
    pub use crate::ForceApplicator;
    
    // Re-export core module prelude
    pub use crate::core::prelude::*;
}