pub mod entropy;
pub mod equilibrium;
pub mod thermal;

use bevy::prelude::*;

pub struct ThermodynamicsPlugin;

impl Plugin for ThermodynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<thermal::Temperature>()
            .register_type::<thermal::ThermalConductivity>()
            .register_type::<thermal::ThermalDiffusivity>()
            .register_type::<entropy::Entropy>()
            .register_type::<entropy::Reversibility>()
            .register_type::<equilibrium::ThermalEquilibrium>()
            .register_type::<equilibrium::PhaseState>()
            .add_event::<thermal::ThermalTransferEvent>()
            .add_systems(Update, thermal::calculate_thermal_transfer);
    }
}

pub mod prelude {
    pub use super::entropy::{
        entropy_change_heat_transfer, entropy_change_irreversible, is_valid_process,
        total_entropy_change, Entropy, Reversibility,
    };
    pub use super::equilibrium::{
        apply_equilibrium_transitivity, equilibrium_time_estimate, find_equilibrium_group,
        is_in_equilibrium, validate_equilibrium_group_consistency, PhaseState, ThermalEquilibrium,
        ThermalProperties,
    };
    pub use super::thermal::{
        thermal_utils::heat_conduction, Temperature, ThermalConductivity, ThermalDiffusivity,
    };
}
