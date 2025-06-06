use bevy::prelude::*;

pub mod save_system;
pub mod versioning;

/// Plugin for save/load functionality with versioning support
#[derive(Default)]
pub struct SaveSystemPlugin;

impl Plugin for SaveSystemPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Will add save/load systems when needed
        // For now, just register the plugin
    }
}

/// Prelude for easy importing
pub mod prelude {
    pub use super::SaveSystemPlugin;
    pub use crate::save_system::{load, save};
    pub use crate::versioning::{is_save_up_to_date, upgrade_save, SAVE_VERSION};
}
