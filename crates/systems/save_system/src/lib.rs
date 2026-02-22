use bevy::prelude::*;

pub mod save_system;
pub mod versioning;

pub struct SaveSystemPlugin {
    settings: save_system::SaveSettings,
}

impl SaveSystemPlugin {
    pub fn new(settings: save_system::SaveSettings) -> Self {
        Self { settings }
    }
}

impl Default for SaveSystemPlugin {
    fn default() -> Self {
        Self {
            settings: save_system::SaveSettings::default(),
        }
    }
}

impl Plugin for SaveSystemPlugin {
    fn build(&self, app: &mut App) {
        {
            let world = app.world_mut();
            let settings = if let Some(existing) = world.get_resource::<save_system::SaveSettings>()
            {
                existing.clone()
            } else {
                let settings = self.settings.clone();
                world.insert_resource(settings.clone());
                settings
            };

            if !world.contains_resource::<save_system::GameTracker>() {
                world.insert_resource(save_system::GameTracker::from_settings(&settings));
            } else if let Some(mut tracker) = world.get_resource_mut::<save_system::GameTracker>() {
                tracker.apply_settings(&settings);
            }
        }

        app.register_type::<save_system::Saveable>()
            .register_type::<save_system::PersistentId>()
            .register_type::<save_system::PersistentIdCounter>()
            .register_type::<save_system::GameState>()
            .register_type::<save_system::GameEvent>()
            .register_type::<save_system::SaveMetadata>();
    }
}

pub mod prelude {
    pub use super::SaveSystemPlugin;
    pub use crate::save_system::{
        GameEvent, GameSaveData, GameSnapshot, GameState, GameTracker, PersistentId,
        PersistentIdCounter, SaveMetadata, SaveSettings, Saveable, WorldSaveExt, delete_save_file,
        get_saved_entity_components, list_save_files, load, load_game_data, save, save_game_data,
    };
    pub use crate::versioning::{SAVE_VERSION, is_save_up_to_date, upgrade_save};
}
