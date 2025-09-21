pub use acoustics;
pub use ai;
pub use pbmpm;
pub use save_system;

use bevy::prelude::*;

/// Systems domain plugin
#[derive(Default)]
pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            acoustics::AcousticsPlugin,
            ai::LPAIPlugin::default(),
            pbmpm::PBMPMPlugin,
            save_system::SaveSystemPlugin,
        ));
    }
}

/// Common systems plugins
pub mod prelude {
    pub use super::SystemsPlugin;

    // Re-export all sub-crate preludes
    pub use acoustics::prelude::*;
    pub use ai::prelude::*;
    pub use pbmpm::prelude::*;
    pub use save_system::prelude::*;
}
