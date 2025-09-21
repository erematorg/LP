pub mod core;

use bevy::prelude::*;
pub use core::newton_laws::NewtonLawsPlugin;

/// Interface for applying forces to entities
pub trait ForceApplicator: Send + Sync {
    /// Apply a force to an entity
    fn apply_force(&self, entity: Entity, force: Vec3);
    /// Get the force magnitude
    fn get_magnitude(&self) -> f32;
    /// Get the force direction
    fn get_direction(&self) -> Vec3;
}

/// Forces domain plugin
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

/// Common forces types and functions
pub mod prelude {
    // Core interfaces from crate root
    pub use crate::{ForceApplicator, ForcesPlugin};

    // Re-export core module prelude
    pub use crate::core::prelude::*;
}
