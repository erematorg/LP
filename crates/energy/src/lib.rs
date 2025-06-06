pub mod conservation;
pub mod electromagnetism;
pub mod thermodynamics;
pub mod waves;

use bevy::prelude::*;

pub use conservation::EnergyConservationPlugin;
pub use electromagnetism::ElectromagnetismPlugin;
pub use thermodynamics::ThermodynamicsPlugin;
pub use waves::WavesPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EnergyType {
    Generic,
    Thermal,
    Kinetic,
    Potential,
    Chemical,
    Electromagnetic,
    Solar,
}

#[derive(Debug)]
pub enum EnergyTransferError {
    Overflow,
    Underflow,
    InsufficientCapacity,
    ThermodynamicConstraint,
}

/// Core trait for all energy-based systems in the simulation
/// This complements the existing EnergyQuantity component
pub trait EnergySystem {
    // Core energy tracking
    fn total_energy(&self) -> f32;

    // Energy transfer with entropy consideration
    fn transfer_energy(&mut self, energy: f32) -> Result<f32, EnergyTransferError> {
        // Default implementation could track basic conservation
        Ok(energy)
    }

    // Transformation efficiency
    fn transformation_efficiency(&self) -> f32 {
        1.0 // Default full efficiency
    }

    // Entropy generation during energy transfer
    fn entropy_generation(&self, _energy_transfer: f32) -> f32 {
        0.0 // Default no entropy generation
    }

    // Energy type for this system
    fn energy_type(&self) -> EnergyType {
        EnergyType::Generic
    }

    // Create an EnergyTransaction for the ledger (optional)
    fn create_transaction(
        &self,
        amount: f32,
        source: Option<Entity>,
        destination: Option<Entity>,
    ) -> conservation::EnergyTransaction {
        conservation::EnergyTransaction {
            transaction_type: if amount > 0.0 {
                conservation::TransactionType::Input
            } else {
                conservation::TransactionType::Output
            },
            amount: amount.abs(),
            source,
            destination,
            timestamp: 0.0, // Current time should be passed in a real implementation
        }
    }
}

/// Main plugin for all energy-related systems
#[derive(Default)]
pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnergyType>()
            .add_plugins(EnergyConservationPlugin)
            .add_plugins(ThermodynamicsPlugin)
            .add_plugins(ElectromagnetismPlugin)
            .add_plugins(WavesPlugin);
    }
}

pub mod prelude {
    pub use super::{EnergySystem, EnergyTransferError};

    pub use crate::conservation::{
        conversion_efficiency, verify_conservation, EnergyAccountingLedger,
        EnergyConservationPlugin, EnergyConservationTracker, EnergyQuantity, EnergyTransaction,
        EnergyTransferEvent, EnergyType, TransactionType,
    };

    pub use crate::electromagnetism::prelude::*;
    pub use crate::thermodynamics::prelude::*;
    pub use crate::waves::prelude::*;
}
