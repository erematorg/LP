pub mod conservation;
pub mod thermodynamics;
pub mod electromagnetism;
pub mod waves;

/// Root energy prelude that re-exports all important items
pub mod prelude {
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