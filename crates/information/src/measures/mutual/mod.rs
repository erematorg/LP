pub mod calculation;

use bevy::prelude::*;
pub use calculation::*;

/// Plugin for mutual information systems
#[derive(Default)]
pub struct MutualInformationPlugin;

impl Plugin for MutualInformationPlugin {
    fn build(&self, app: &mut App) {
        // For now just register the plugin - future Bevy integration here
        // Could add systems for real-time MI calculation between entities
    }
}

pub mod prelude {
    pub use super::calculation::*;
    pub use super::MutualInformationPlugin;
}