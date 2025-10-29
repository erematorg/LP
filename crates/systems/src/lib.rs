pub use acoustics;
pub use ai;
pub use mpm;
pub use save_system;

use bevy::prelude::*;
use save_system::save_system::SaveSettings;

/// Systems domain plugin
#[derive(Clone, Debug)]
pub struct SystemsPlugin {
    include_ai: bool,
    include_acoustics: bool,
    include_mpm: bool,
    include_save: bool,
    save_settings: SaveSettings,
}

impl Default for SystemsPlugin {
    fn default() -> Self {
        Self {
            include_ai: true,
            include_acoustics: true,
            include_mpm: true,
            include_save: true,
            save_settings: SaveSettings::default(),
        }
    }
}

impl SystemsPlugin {
    /// Enable or disable the AI systems.
    pub fn with_ai(mut self, enabled: bool) -> Self {
        self.include_ai = enabled;
        self
    }

    /// Enable or disable acoustics systems.
    pub fn with_acoustics(mut self, enabled: bool) -> Self {
        self.include_acoustics = enabled;
        self
    }

    /// Enable or disable the Material Point Method placeholder systems.
    pub fn with_mpm(mut self, enabled: bool) -> Self {
        self.include_mpm = enabled;
        self
    }

    /// Enable or disable the save-system integration.
    pub fn with_save_system(mut self, enabled: bool) -> Self {
        self.include_save = enabled;
        self
    }

    /// Provide custom save settings that will be forwarded to the save-system plugin.
    pub fn with_save_settings(mut self, settings: SaveSettings) -> Self {
        self.save_settings = settings;
        self
    }
}

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        if self.include_acoustics {
            app.add_plugins(acoustics::AcousticsPlugin);
        }

        if self.include_ai {
            app.add_plugins(ai::LPAIPlugin::default());
        }

        if self.include_mpm {
            app.add_plugins(mpm::MPMPlugin);
        }

        if self.include_save {
            app.add_plugins(save_system::SaveSystemPlugin::new(self.save_settings.clone()));
        }
    }
}

/// Common systems plugins
pub mod prelude {
    pub use super::SystemsPlugin;

    // Re-export all sub-crate preludes
    pub use acoustics::prelude::*;
    pub use ai::prelude::*;
    pub use mpm::prelude::*;
    pub use save_system::prelude::*;
}
