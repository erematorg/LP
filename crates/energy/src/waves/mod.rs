pub mod oscillation;
pub mod propagation;
pub mod superposition;
pub mod wave_equation;

/// The waves prelude.
///
/// This includes the most common types for wave systems.
pub mod prelude {
    pub use crate::waves::oscillation::{
        WaveParameters, wave_number, angular_frequency, damping_from_half_life
    };
    pub use crate::waves::propagation::{
        WavePosition, WaveType, WaveCenterMarker, 
        solve_wave, solve_radial_wave, update_wave_displacements, create_linear_wave
    };
    pub use crate::waves::superposition::{
        StandingWaveMarker, solve_standing_wave, 
        update_standing_waves, create_standing_wave
    };
    pub use crate::waves::wave_equation::{
        WaveEquation2D, WaveEquationComponent, update_wave_equation
    };
}