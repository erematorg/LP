//! Minimal test to check if plugins can be added without B0001 errors.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
        .add_plugins((
            utils::UtilsPlugin,
            forces::ForcesPlugin,
            energy::waves::WavesPlugin,
            // energy::EnergyPlugin,
            matter::MatterPlugin,
        ))
        .run();
}
