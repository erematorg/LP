pub mod states;

use bevy::prelude::*;

/// Main plugin for all matter-related systems
#[derive(Default)]
pub struct MatterPlugin;

impl Plugin for MatterPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Add state-specific plugins when implementations are complete
        // app.add_plugins((
        //     SolidsPlugin,
        //     FluidsPlugin,
        //     GasesPlugin,
        //     PlasmaPlugin,
        // ));

        // For now, just register the plugin to establish the structure
        app.insert_resource(MatterSystemsInitialized);
    }
}

/// Resource to indicate matter systems are initialized
#[derive(Resource, Default)]
pub struct MatterSystemsInitialized;

pub mod prelude {
    // Main plugin export
    pub use crate::MatterPlugin;

    // Re-export from states module when ready
    //pub use crate::states::prelude::*;
}
