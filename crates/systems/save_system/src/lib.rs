use bevy::prelude::*;

pub mod save_system;
pub mod versioning;

/// Plugin for save/load functionality with versioning support
pub struct SaveSystemPlugin;

impl Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Will add save/load systems when needed
        // For now, just register the plugin
    }
}

/// Prelude for easy importing
pub mod prelude {
    pub use super::SaveSystemPlugin;
    pub use crate::save_system::{save, load};
    pub use crate::versioning::{SAVE_VERSION, is_save_up_to_date, upgrade_save};
}
