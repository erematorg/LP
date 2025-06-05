pub mod oscillation;
pub mod propagation;
pub mod superposition;
pub mod wave_equation;

use bevy::prelude::*;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        // Register wave components
        app.register_type::<oscillation::WaveParameters>()
            .register_type::<propagation::WavePosition>()
            .register_type::<propagation::WaveType>()
            .register_type::<propagation::WaveCenterMarker>()
            .register_type::<superposition::StandingWaveMarker>()
            .register_type::<wave_equation::WaveEquationComponent>()
            .add_event::<oscillation::WaveGenerationEvent>()
            .add_systems(Update, propagation::update_wave_displacements)
            .add_systems(Update, superposition::update_standing_waves)
            .add_systems(Update, wave_equation::update_wave_equation);
    }
}

/// The waves prelude.
///
/// This includes the most common types for wave systems.
pub mod prelude {
    pub use crate::waves::oscillation::{
        angular_frequency, damping_from_half_life, wave_number, WaveParameters,
    };
    pub use crate::waves::propagation::{
        create_linear_wave, solve_radial_wave, solve_wave, update_wave_displacements,
        WaveCenterMarker, WavePosition, WaveType,
    };
    pub use crate::waves::superposition::{
        create_standing_wave, solve_standing_wave, update_standing_waves, StandingWaveMarker,
    };
    pub use crate::waves::wave_equation::{
        update_wave_equation, WaveEquation2D, WaveEquationComponent,
    };
}
