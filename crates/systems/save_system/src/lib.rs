use bevy::prelude::*;

pub mod save_system;
pub mod versioning;

pub struct SaveSystemPlugin;

impl Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<save_system::GameTracker>()
            .register_type::<save_system::Saveable>()
            .register_type::<save_system::GameState>()
            .register_type::<save_system::GameEvent>()
            .register_type::<save_system::SaveMetadata>();
    }
}

impl Default for SaveSystemPlugin {
    fn default() -> Self {
        Self
    }
}

pub mod prelude {
    pub use super::SaveSystemPlugin;
    pub use crate::save_system::{
        GameEvent, GameSaveData, GameSnapshot, GameState, GameTracker, SaveMetadata, Saveable,
        WorldSaveExt, get_save_directory, get_save_path, load, load_game_data, save,
        save_game_data,
    };
    pub use crate::versioning::{SAVE_VERSION, is_save_up_to_date, upgrade_save};
}
