pub mod gravity;
pub mod newton_laws;

/// Prelude for the forces core module.
///
/// This includes the fundamental physics components and systems.
pub mod prelude {
    // Re-export from gravity module
    pub use crate::core::gravity::{
        calculate_elliptical_orbit_velocity, calculate_escape_velocity,
        calculate_gravitational_attraction, calculate_orbital_velocity, GravityAffected,
        GravityParams, GravitySource, MassiveBody, UniformGravity, GRAVITATIONAL_CONSTANT,
    };

    // Re-export from newton_laws module
    pub use crate::core::newton_laws::{
        apply_forces, calculate_kinetic_energy, calculate_momentum, integrate_positions,
        AppliedForce, Distance, ForceImpulse, Mass, Norm, PairedForce, PairedForceInteraction,
        PhysicsPlugin, Velocity,
    };
}
