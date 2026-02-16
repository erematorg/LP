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

// NOTE: Matter crate is early-stage placeholder. Blocked on:
// 1. MPM solver stabilization and full parameter exposure (currently in systems/mpm)
// 2. Universal collision physics (momentum, energy, mass conservation contracts)
// 3. Material constitutive models (elasticity, plasticity, viscosity)
// 4. Phase transitions, equations of state (EOS), latent heat
// 5. Energy/matter ledger integration with forces and energy crates
//
// When unblocked, matter will provide:
// - Solid/fluid/gas/plasma state machines with phase transitions
// - Material properties (density, viscosity, elasticity, thermal capacity)
// - Coupling to MPM solver for large-N deformable body simulation

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
