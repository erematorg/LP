use bevy::prelude::*;

pub use acoustics;
pub use ai;
pub use pbmpm;
pub use save_system;

/// Main systems plugin that bundles all system plugins
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

/// The systems prelude.
///
/// This includes all system plugins for easy importing.
pub mod prelude {
    pub use super::SystemsPlugin;
    
    // Re-export all sub-crate preludes
    pub use acoustics::prelude::*;
    pub use ai::prelude::*;
    pub use pbmpm::prelude::*;
    pub use save_system::prelude::*;
}
