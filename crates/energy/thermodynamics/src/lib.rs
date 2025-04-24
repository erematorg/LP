pub mod entropy;
pub mod equilibrium;
pub mod thermal;


pub mod prelude {
    pub use crate::thermal::{Temperature, ThermalConductivity, ThermalDiffusivity, heat_conduction};
    pub use crate::entropy::{Entropy, Reversibility, entropy_change_heat_transfer, entropy_change_irreversible, is_valid_process, total_entropy_change};
    pub use crate::equilibrium::{ThermalEquilibrium, PhaseState, ThermalProperties, is_in_equilibrium, equilibrium_time_estimate};
}