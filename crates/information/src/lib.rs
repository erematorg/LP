pub mod fractals;

use bevy::prelude::*;

/// Main plugin for all information-related systems
#[derive(Default)]
pub struct InformationPlugin;

impl Plugin for InformationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Add fractal systems when Bevy integration is implemented
        // For now, just register the plugin to establish the structure
        app.insert_resource(InformationSystemsInitialized);
    }
}

/// Resource to indicate information systems are initialized
#[derive(Resource, Default)]
pub struct InformationSystemsInitialized;

pub mod prelude {
    // Main plugin export
    pub use crate::InformationPlugin;

    // Re-export from fractals module
    pub use crate::fractals::prelude::*;

    // Any future modules would be added here
}
