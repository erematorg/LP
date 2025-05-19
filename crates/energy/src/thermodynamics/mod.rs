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
   pub use super::thermal::{Temperature, ThermalConductivity, ThermalDiffusivity, thermal_utils::heat_conduction};
   pub use super::entropy::{Entropy, Reversibility, entropy_change_heat_transfer, entropy_change_irreversible, is_valid_process, total_entropy_change};
   pub use super::equilibrium::{ThermalEquilibrium, PhaseState, ThermalProperties, is_in_equilibrium, equilibrium_time_estimate};
}