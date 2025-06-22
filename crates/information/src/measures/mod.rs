pub mod divergence;
pub mod mutual;
pub mod shannon;

use bevy::prelude::*;

// Re-export main plugin
pub use mutual::MutualInformationPlugin;

pub mod prelude {
    pub use super::divergence::KLDivergence;
    pub use super::mutual::*;
    pub use super::shannon::Shannon;
}