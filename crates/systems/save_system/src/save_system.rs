use crate::versioning::{is_save_up_to_date, upgrade_save};
use bevy::prelude::ReflectComponent;
use bevy::prelude::*;
use bevy::reflect::{Reflect, ReflectSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the default save directory for the platform
pub fn get_save_directory() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("LP")
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("LP")
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
            })
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("LP")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from(".")
    }
}

/// Get the full path for a save file
pub fn get_save_path(filename: &str) -> PathBuf {
    let save_dir = get_save_directory();

    // Ensure save directory exists
    if let Err(e) = fs::create_dir_all(&save_dir) {
        eprintln!(
            "Warning: Could not create save directory {:?}: {}",
            save_dir, e
        );
        return PathBuf::from(filename); // Fallback to current directory
    }

    save_dir.join(filename)
}

pub fn save<T: Serialize>(data: &T, path: &str) -> Result<(), String> {
    let full_path = get_save_path(path);
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Serialization failed: {}", e))?;
    fs::write(&full_path, json).map_err(|e| format!("File write failed: {}", e))?;
    Ok(())
}

pub fn load<T: for<'de> Deserialize<'de> + Default + Serialize>(path: &str) -> Result<T, String> {
    let full_path = get_save_path(path);
    let json = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(_) => {
            let default_data = T::default();

            if let Err(e) = save(&default_data, path) {
                return Err(format!("Failed to create default save: {}", e));
            }

            return Ok(default_data);
        }
    };

    let mut data: Value =
        serde_json::from_str(&json).map_err(|e| format!("Deserialization failed: {}", e))?;

    if !is_save_up_to_date(&data) {
        eprintln!("[Warning] Save file is outdated! Attempting to upgrade...");
        data = upgrade_save(data);
        save(&data, path)?; // Save upgraded version
    }

    serde_json::from_value(data).map_err(|e| format!("Final deserialization failed: {}", e))
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameSaveData {
    pub version: String,
    pub timestamp: f64,
    pub game_time: f64,
    pub metadata: SaveMetadata,

    pub events: Vec<GameEvent>,
    pub entities: HashMap<String, HashMap<String, Value>>,
    pub game_state: GameState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct SaveMetadata {
    pub created_at: String,
    pub game_version: String,
    pub save_system_version: String,
    pub platform: String,
    pub description: Option<String>,
    pub playtime_seconds: f64,
}

impl Default for SaveMetadata {
    fn default() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            created_at: format!("Unix timestamp: {}", timestamp),
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            save_system_version: crate::versioning::SAVE_VERSION.to_string(),
            platform: std::env::consts::OS.to_string(),
            description: None,
            playtime_seconds: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Reflect)]
pub struct GameState {
    pub total_energy: f32,
    pub entity_count: u32,
    pub environment: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct GameEvent {
    pub event_type: String,
    pub game_time: f64,
    pub entity_id: Option<String>,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Reflect)]
pub struct EntityData {
    pub position: Option<Vec3>,
    pub energy: Option<f32>,
    pub custom: HashMap<String, String>,
}

#[derive(Component, Reflect)]
pub struct Saveable;

#[derive(Resource, Clone)]
pub struct GameTracker {
    pub state: GameState,
    pub events: Vec<GameEvent>,
    pub auto_save_timer: Timer,
    pub snapshots: Vec<GameSnapshot>,
    pub max_snapshots: usize,
}

impl Default for GameTracker {
    fn default() -> Self {
        Self::new(60.0) // Auto-save every 60 seconds by default
    }
}

#[derive(Debug, Clone)]
pub struct GameSnapshot {
    pub state: GameState,
    pub events: Vec<GameEvent>,
    pub timestamp: f64,
    pub name: String,
}

impl GameTracker {
    /// Create a new GameTracker with auto-save enabled
    pub fn new(auto_save_interval_secs: f32) -> Self {
        Self {
            state: GameState::default(),
            events: Vec::new(),
            auto_save_timer: Timer::from_seconds(auto_save_interval_secs, TimerMode::Repeating),
            snapshots: Vec::new(),
            max_snapshots: 10,
        }
    }

    /// Create a snapshot of the current game state
    pub fn snapshot(&mut self, name: String, timestamp: f64) {
        let snapshot = GameSnapshot {
            state: self.state.clone(),
            events: self.events.clone(),
            timestamp,
            name,
        };

        self.snapshots.push(snapshot);

        // Keep only the latest snapshots
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Rollback to the most recent snapshot
    pub fn rollback(&mut self) -> Result<String, String> {
        if let Some(snapshot) = self.snapshots.last() {
            self.state = snapshot.state.clone();
            self.events = snapshot.events.clone();
            Ok(format!("Rolled back to snapshot: {}", snapshot.name))
        } else {
            Err("No snapshots available for rollback".to_string())
        }
    }

    /// Rollback to a specific named snapshot
    pub fn rollback_to(&mut self, name: &str) -> Result<String, String> {
        if let Some(snapshot) = self.snapshots.iter().rev().find(|s| s.name == name) {
            self.state = snapshot.state.clone();
            self.events = snapshot.events.clone();
            Ok(format!("Rolled back to snapshot: {}", name))
        } else {
            Err(format!("Snapshot '{}' not found", name))
        }
    }

    /// Convenience method for auto-saving
    pub fn auto_save(&self, world: &mut World, game_time: f64) -> Result<(), String> {
        save_game_data(
            world,
            self,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            game_time,
        )
    }

    /// Update the auto-save timer and return true if it's time to save
    pub fn should_auto_save(&mut self, delta_time: f32) -> bool {
        self.auto_save_timer
            .tick(std::time::Duration::from_secs_f32(delta_time));
        self.auto_save_timer.just_finished()
    }

    pub fn set_environment(&mut self, name: String, value: f32) {
        self.state.environment.insert(name, value);
    }

    pub fn set_total_energy(&mut self, energy: f32) {
        self.state.total_energy = energy;
    }

    pub fn set_entity_count(&mut self, count: u32) {
        self.state.entity_count = count;
    }

    pub fn log_event(
        &mut self,
        event_type: String,
        entity_id: Option<String>,
        data: String,
        game_time: f64,
    ) {
        self.events.push(GameEvent {
            event_type,
            game_time,
            entity_id,
            data,
        });

        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }
}

pub fn save_game_data(
    world: &mut World,
    tracker: &GameTracker,
    time: f64,
    game_time: f64,
) -> Result<(), String> {
    let mut entities = HashMap::new();

    let mut query = world.query_filtered::<Entity, With<Saveable>>();
    let saveable_entities: Vec<Entity> = query.iter(world).collect();

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();

    for entity in saveable_entities {
        let entity_id = format!("{:?}", entity);
        let mut entity_data = HashMap::new();

        if let Ok(entity_ref) = world.get_entity(entity) {
            for component_id in entity_ref.archetype().components() {
                let Some(component_info) = world.components().get_info(component_id) else {
                    continue;
                };

                let Some(type_id) = component_info.type_id() else {
                    continue;
                };

                let Some(type_registration) = type_registry.get(type_id) else {
                    continue;
                };

                let Some(reflect_serialize) = type_registration.data::<ReflectSerialize>() else {
                    continue;
                };

                let component_name = type_registration.type_info().type_path().to_string();

                if component_name.contains("Saveable") {
                    continue;
                }

                let Some(reflect_component) = type_registration.data::<ReflectComponent>() else {
                    continue;
                };

                if let Some(reflected) = reflect_component.reflect(entity_ref) {
                    let serializable = reflect_serialize.get_serializable(reflected);
                    if let Ok(value) = serde_json::to_value(&*serializable) {
                        entity_data.insert(component_name, value);
                    }
                }
            }
        }

        entities.insert(entity_id, entity_data);
    }

    let metadata = SaveMetadata {
        playtime_seconds: game_time,
        ..Default::default()
    };

    let save_data = GameSaveData {
        version: crate::versioning::SAVE_VERSION.to_string(),
        timestamp: time,
        game_time,
        metadata,
        game_state: tracker.state.clone(),
        events: tracker.events.clone(),
        entities,
    };

    save(&save_data, "game_save.json")
}

pub fn load_game_data() -> Result<GameSaveData, String> {
    load::<GameSaveData>("game_save.json")
}

/// Extension trait for World to add bevy_save-style convenience methods
pub trait WorldSaveExt {
    fn save_game(&mut self, path: &str) -> Result<(), String>;
    fn load_game(&mut self, path: &str) -> Result<(), String>;
}

impl WorldSaveExt for World {
    fn save_game(&mut self, _path: &str) -> Result<(), String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let game_time = self
            .get_resource::<Time>()
            .map(|time| time.elapsed_secs_f64())
            .unwrap_or(0.0);

        // Clone tracker to avoid borrow conflicts
        if let Some(tracker) = self.get_resource::<GameTracker>().cloned() {
            save_game_data(self, &tracker, timestamp, game_time)?;
            Ok(())
        } else {
            Err("GameTracker resource not found".to_string())
        }
    }

    fn load_game(&mut self, path: &str) -> Result<(), String> {
        let save_data = load::<GameSaveData>(path)?;

        // Update GameTracker if it exists
        if let Some(mut tracker) = self.get_resource_mut::<GameTracker>() {
            tracker.state = save_data.game_state;
            tracker.events = save_data.events;
        }

        Ok(())
    }
}
