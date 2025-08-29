pub mod entropy;
pub mod equilibrium;
pub mod thermal;

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThermodynamicsSet {
    /// Calculate thermal transfers and conduction
    ThermalTransfer,
    /// Update entropy and equilibrium states
    Equilibrium,
}

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
            .configure_sets(
                Update,
                (ThermodynamicsSet::ThermalTransfer, ThermodynamicsSet::Equilibrium).chain(),
            )
            .add_systems(
                Update,
                thermal::calculate_thermal_transfer.in_set(ThermodynamicsSet::ThermalTransfer),
            );
    }
}

pub mod prelude {
    pub use super::entropy::{
        Entropy, Reversibility, entropy_change_heat_transfer, entropy_change_irreversible,
        is_valid_process, total_entropy_change,
    };
    pub use super::equilibrium::{
        PhaseState, ThermalEquilibrium, ThermalProperties, apply_equilibrium_transitivity,
        equilibrium_time_estimate, find_equilibrium_group, is_in_equilibrium,
        validate_equilibrium_group_consistency,
    };
    pub use super::thermal::{
        Temperature, ThermalConductivity, ThermalDiffusivity, thermal_utils::heat_conduction,
    };
}
