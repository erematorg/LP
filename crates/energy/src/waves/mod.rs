pub mod oscillation;
pub mod propagation;

use bevy::prelude::Vec2;

/// Helper function to normalize a vector or return a fallback if the vector is too small
#[inline]
pub(crate) fn normalize_or(vec: Vec2, fallback: Vec2) -> Vec2 {
    if vec.length_squared() > f32::EPSILON {
        vec.normalize()
    } else {
        fallback
    }
}
pub mod superposition;
pub mod wave_equation;

use bevy::prelude::*;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(propagation::WaveGrid(utils::SpatialGrid::new(100.0)))
            .register_type::<oscillation::WaveParameters>()
            .register_type::<propagation::WavePosition>()
            .register_type::<propagation::WaveType>()
            .register_type::<propagation::WaveCenterMarker>()
            .register_type::<superposition::StandingWaveMarker>()
            .register_type::<wave_equation::WaveEquationComponent>()
            .add_message::<oscillation::WaveGenerationEvent>()
            .add_systems(
                Update,
                (
                    propagation::update_wave_grid,
                    propagation::update_wave_displacements,
                    superposition::update_standing_waves,
                    wave_equation::update_wave_equation,
                ).chain(),
            );
    }
}

/// The waves prelude.
pub mod prelude {
    pub use crate::waves::oscillation::{
        WaveParameters, angular_frequency, damping_from_half_life, wave_number,
    };
    pub use crate::waves::propagation::{
        WaveCenterMarker, WavePosition, WaveType, create_linear_wave, solve_radial_wave,
        solve_wave, update_wave_displacements,
    };
    pub use crate::waves::superposition::{
        StandingWaveMarker, create_standing_wave_parameters, solve_standing_wave,
        update_standing_waves,
    };
    pub use crate::waves::wave_equation::{
        WaveEquation2D, WaveEquationComponent, update_wave_equation,
    };
}
