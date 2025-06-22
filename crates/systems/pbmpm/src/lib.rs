use bevy::prelude::*;

pub mod grid;
pub mod particle;
pub mod solver;
pub mod transfer;

/// Plugin for Point-Based Material Point Method physics solver
#[derive(Default)]
pub struct PBMPMPlugin;

impl Plugin for PBMPMPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Will integrate PBMPM systems when implementation is ready
        // Plugin structure prepared for upcoming development
    }
}

/// The PBMPM prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    pub use super::PBMPMPlugin;

    // TODO: Export main types when implementation is complete
    // pub use crate::grid::*;
    // pub use crate::particles::*;
    // pub use crate::solver::*;
    // pub use crate::transfer::*;
}
