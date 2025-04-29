pub mod conservation;
pub mod thermodynamics;
pub mod electromagnetism;
pub mod waves;

// Add these new energy system related definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// This complements the existing EnergyQuantity and EnergyTransfer components
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
    fn create_transaction(&self, amount: f32, source: Option<bevy::prelude::Entity>, 
                         destination: Option<bevy::prelude::Entity>) -> conservation::EnergyTransaction {
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

/// Root energy prelude that re-exports all important items
pub mod prelude {
    // Add the new trait and types to the prelude
    pub use super::{EnergySystem, EnergyTransferError, EnergyType};
    
    // Re-export from conservation
    pub use crate::conservation::{EnergyQuantity, EnergyTransfer, EnergyConversion, 
                            EnergyTransaction, EnergyAccountingLedger, 
                            TransactionType, SystemConservationTracker,
                            verify_conservation, conversion_efficiency};
    
    // Re-export from submodules
    pub use crate::thermodynamics::prelude::*;
    pub use crate::waves::prelude::*;
    pub use crate::electromagnetism::prelude::*;
}