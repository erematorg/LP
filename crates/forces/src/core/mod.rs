pub mod gravity;
pub mod newton_laws;

/// Prelude for the forces core module.
///
/// This includes the fundamental physics components and systems.
pub mod prelude {
    // Re-export from gravity module
    pub use crate::core::gravity::{
        DEFAULT_GRAVITATIONAL_CONSTANT, GravityAffected, GravityForceMode, GravityParams,
        GravityPlugin, GravitySource, MassiveBody, UniformGravity,
        calculate_elliptical_orbit_velocity, calculate_escape_velocity,
        calculate_gravitational_attraction, calculate_mutual_gravitational_attraction,
        calculate_orbital_velocity, calculate_plummer_orbital_velocity,
    };

    // Re-export from newton_laws module
    pub use crate::core::newton_laws::{
        AppliedForce, AppliedTorque, Distance, ForceImpulse, ForcesDiagnostics,
        ForcesDiagnosticsPlugin, IntegratorKind, Mass, MomentOfInertia, NewtonLawsPlugin, Norm,
        PairedForce, PairedForceInteraction, RotationalWorkEvent, Velocity, WorkDoneEvent,
        calculate_angular_momentum, calculate_kinetic_energy, calculate_momentum,
        calculate_rotational_kinetic_energy, calculate_torque_from_force,
        integrate_newton_second_law, integrate_positions_symplectic_euler, integrate_torques,
        update_forces_diagnostics,
    };
}
