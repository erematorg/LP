use bevy::prelude::*;

pub mod grid;
pub mod particle;
pub mod solver;
pub mod transfer;

/// Plugin for the Material Point Method physics solver
#[derive(Default)]
pub struct MPMPlugin;

impl Plugin for MPMPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Will integrate MPM systems when implementation is ready
        // Plugin structure prepared for upcoming development
    }
}

/// The MPM prelude.
///
/// Common types from this crate re-exported for convenience.
pub mod prelude {
    pub use super::MPMPlugin;

    // TODO: Export main types when implementation is complete
    // pub use crate::grid::*;
    // pub use crate::particles::*;
    // pub use crate::solver::*;
    // pub use crate::transfer::*;
}
