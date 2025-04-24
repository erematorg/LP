pub use electromagnetism;
pub use thermodynamics;
pub use waves;

pub mod conservation;

/// Root energy prelude that re-exports all important items
pub mod prelude {
    // Re-export from conservation
    pub use crate::conservation::{EnergyQuantity, EnergyTransfer, EnergyConversion, 
                            EnergyTransaction, EnergyAccountingLedger, 
                            TransactionType, SystemConservationTracker,
                            verify_conservation, conversion_efficiency};
    
    // Re-export from subcrates
    pub use crate::thermodynamics::prelude::*;
    pub use crate::waves::prelude::*;
    pub use crate::electromagnetism::prelude::*;
}
