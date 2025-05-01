pub mod gravity;
pub mod newton_laws;

/// Prelude for the forces core module.
///
/// This includes the fundamental physics components and systems.
pub mod prelude {
    // Re-export from gravity module
    pub use crate::core::gravity::{
        GravityParams, UniformGravity, GravityAffected, GravitySource, MassiveBody,
        GRAVITATIONAL_CONSTANT, calculate_gravitational_attraction, 
        calculate_orbital_velocity, calculate_elliptical_orbit_velocity, calculate_escape_velocity
    };
    
    // Re-export from newton_laws module
    pub use crate::core::newton_laws::{
        Mass, AppliedForce, Velocity, PhysicsPlugin,
        apply_forces, integrate_positions, calculate_momentum, calculate_kinetic_energy,
        Norm, Distance, ForceImpulse, PairedForce, PairedForceInteraction
    };
}