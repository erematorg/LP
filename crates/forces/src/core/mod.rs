pub mod gravity;
pub mod newton_laws;

/// Prelude for the forces core module.
///
/// This includes the fundamental physics components and systems.
pub mod prelude {
    // Re-export from gravity module
    pub use crate::core::gravity::{
        GRAVITATIONAL_CONSTANT, GravityAffected, GravityParams, GravitySource, MassiveBody,
        UniformGravity, calculate_elliptical_orbit_velocity, calculate_escape_velocity,
        calculate_gravitational_attraction, calculate_orbital_velocity,
    };

    // Re-export from newton_laws module
    pub use crate::core::newton_laws::{
        AppliedForce, Distance, ForceImpulse, Mass, NewtonLawsPlugin, Norm, PairedForce,
        PairedForceInteraction, Velocity, apply_forces, calculate_kinetic_energy,
        calculate_momentum, integrate_positions,
    };
}
