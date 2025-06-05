pub mod core;

use bevy::prelude::*;
pub use core::newton_laws::NewtonLawsPlugin;

/// Force application interface for physical forces
pub trait ForceApplicator: Send + Sync {
    /// Apply a force to an entity
    fn apply_force(&self, entity: Entity, force: Vec3);

    /// Get the force magnitude
    fn get_magnitude(&self) -> f32;

    /// Get the force direction
    fn get_direction(&self) -> Vec3;
}

/// Main plugin for all force-related systems
#[derive(Default)]
pub struct ForcesPlugin;

impl Plugin for ForcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NewtonLawsPlugin)
            .register_type::<core::newton_laws::Mass>()
            .register_type::<core::newton_laws::Velocity>()
            .register_type::<core::newton_laws::AppliedForce>()
            .register_type::<core::gravity::GravityAffected>()
            .register_type::<core::gravity::GravitySource>()
            .register_type::<core::gravity::MassiveBody>()
            .init_resource::<core::gravity::GravityParams>()
            .init_resource::<core::gravity::UniformGravity>();
    }
}

/// The forces prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    // Core interfaces from crate root
    pub use crate::{ForceApplicator, ForcesPlugin};

    // Re-export core module prelude
    pub use crate::core::prelude::*;
}
