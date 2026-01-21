pub mod geometry;
pub mod states;

use bevy::prelude::*;

/// Main plugin for all matter-related systems
#[derive(Default)]
pub struct MatterPlugin;

impl Plugin for MatterPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register geometric properties
            .register_type::<geometry::Radius>()
            // Initialize matter systems
            .insert_resource(MatterSystemsInitialized);

        // TODO: Add state-specific plugins when implementations are complete
        // app.add_plugins((
        //     SolidsPlugin,
        //     FluidsPlugin,
        //     GasesPlugin,
        //     PlasmaPlugin,
        // ));
    }
}

/// Resource to indicate matter systems are initialized
#[derive(Resource, Default)]
pub struct MatterSystemsInitialized;

pub mod prelude {
    // Main plugin export
    pub use crate::MatterPlugin;

    // Geometric properties
    pub use crate::geometry::Radius;

    // Re-export from states module when ready
    //pub use crate::states::prelude::*;
}
