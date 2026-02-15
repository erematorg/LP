use crate::versioning::{is_save_up_to_date, upgrade_save};
use bevy::prelude::ReflectComponent;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::reflect::serde::ReflectSerializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use platform_dirs::AppDirs;

#[derive(Clone, Debug)]
enum SaveDirectory {
    Auto(PathBuf),
    Custom(PathBuf),
}

#[derive(Resource, Clone, Debug)]
pub struct SaveSettings {
    workspace: String,
    directory: SaveDirectory,
    default_file: String,
    autosave_interval: Duration,
    snapshot_limit: usize,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveSettings {
    pub fn new() -> Self {
        let workspace = Self::detect_workspace();
        let env_override = std::env::var("LP_SAVE_DIRECTORY").ok().map(PathBuf::from);
        let directory = env_override
            .map(SaveDirectory::Custom)
            .unwrap_or_else(|| SaveDirectory::Auto(default_save_root(&workspace)));

        Self {
            workspace,
            directory,
            default_file: "game_save.json".to_string(),
            autosave_interval: Duration::from_secs(60),
            snapshot_limit: 10,
        }
    }

    pub fn with_custom_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = SaveDirectory::Custom(directory.into());
        self
    }

    pub fn with_default_file(mut self, file: impl Into<String>) -> Self {
        self.default_file = file.into();
        self
    }

    pub fn with_autosave_interval(mut self, duration: Duration) -> Self {
        self.autosave_interval = duration;
        self
    }

    pub fn with_snapshot_limit(mut self, limit: usize) -> Self {
        self.snapshot_limit = limit.max(1);
        self
    }

    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    pub fn default_file(&self) -> &str {
        &self.default_file
    }

    pub fn autosave_interval(&self) -> Duration {
        self.autosave_interval
    }

    pub fn snapshot_limit(&self) -> usize {
        self.snapshot_limit
    }

    pub fn resolve_directory(&self) -> PathBuf {
        match &self.directory {
            SaveDirectory::Auto(path) | SaveDirectory::Custom(path) => path.clone(),
        }
    }

    pub fn ensure_directory_exists(&self) -> Result<PathBuf, String> {
        let directory = self.resolve_directory();
        if let Err(err) = fs::create_dir_all(&directory) {
            return Err(format!(
                "Failed to create save directory {}: {}",
                directory.display(),
                err
            ));
        }
        Ok(directory)
    }

    pub fn resolve_file(&self, key: &str) -> Result<PathBuf, String> {
        if key.is_empty() {
            return self.resolve_file(self.default_file());
        }

        let path_key = Path::new(key);

        if path_key.is_absolute() {
            return Ok(path_key.to_path_buf());
        }

        if path_key
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(format!(
                "Save path '{}' must not contain parent directory segments ('..')",
                key
            ));
        }

        let mut directory = self.ensure_directory_exists()?;
        directory.push(path_key);
        Ok(directory)
    }

    fn detect_workspace() -> String {
        if let Ok(custom) = std::env::var("LP_SAVE_WORKSPACE") {
            return custom;
        }

        if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
            if let Some(name) = Path::new(&workspace_dir)
                .file_name()
                .and_then(|os| os.to_str())
            {
                return name.to_string();
            }
        }

        "LP".to_string()
    }
}

fn default_save_root(workspace: &str) -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    {
        AppDirs::new(Some(workspace), true)
            .map(|dirs| dirs.data_dir.join("saves"))
            .unwrap_or_else(|| std::env::temp_dir().join(workspace).join("saves"))
    }

    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from(workspace)
    }
}

/// Convenience helper for legacy code paths that still expect a default directory.
pub fn get_save_directory() -> PathBuf {
    let settings = SaveSettings::default();
    settings
        .ensure_directory_exists()
        .unwrap_or_else(|_| settings.resolve_directory())
}

/// Convenience helper for legacy code paths that still expect a resolved path.
pub fn get_save_path(filename: &str) -> PathBuf {
    SaveSettings::default()
        .resolve_file(filename)
        .unwrap_or_else(|_| PathBuf::from(filename))
}

pub fn save<T: Serialize>(data: &T, settings: &SaveSettings, key: &str) -> Result<PathBuf, String> {
    let full_path = settings.resolve_file(key)?;
    if let Some(parent) = full_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return Err(format!(
                "Failed to prepare save directory {}: {}",
                parent.display(),
                err
            ));
        }
    }
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Serialization failed: {}", e))?;
    write_json_atomic(&full_path, &json)?;
    Ok(full_path)
}

fn write_json_atomic(path: &Path, json: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Cannot resolve parent directory for {}", path.display()))?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid save file name for {}", path.display()))?;

    let unique = format!(
        "{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let temp_path = parent.join(format!(".{}.{}.tmp", file_name, unique));

    {
        let mut temp_file = fs::File::create(&temp_path).map_err(|err| {
            format!(
                "Failed to create temp save file {}: {}",
                temp_path.display(),
                err
            )
        })?;

        temp_file.write_all(json.as_bytes()).map_err(|err| {
            format!(
                "Failed to write temp save file {}: {}",
                temp_path.display(),
                err
            )
        })?;
        temp_file.sync_all().map_err(|err| {
            format!(
                "Failed to flush temp save file {}: {}",
                temp_path.display(),
                err
            )
        })?;
    }

    if path.exists() {
        fs::remove_file(path).map_err(|err| {
            format!(
                "Failed to replace existing save {}: {}",
                path.display(),
                err
            )
        })?;
    }

    fs::rename(&temp_path, path).map_err(|err| {
        format!(
            "Failed to move temp save {} to {}: {}",
            temp_path.display(),
            path.display(),
            err
        )
    })
}

pub fn load<T>(settings: &SaveSettings, key: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Default + Serialize,
{
    let full_path = settings.resolve_file(key)?;
    let json = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(_) => {
            let default_data = T::default();
            save(&default_data, settings, key)?;
            return Ok(default_data);
        }
    };

    let mut data: Value =
        serde_json::from_str(&json).map_err(|e| format!("Deserialization failed: {}", e))?;

    if !is_save_up_to_date(&data) {
        eprintln!("[Warning] Save file is outdated! Attempting to upgrade...");
        data = upgrade_save(data);
        save(&data, settings, key)?;
    }

    serde_json::from_value(data).map_err(|e| format!("Final deserialization failed: {}", e))
}

pub fn list_save_files(settings: &SaveSettings) -> Result<Vec<PathBuf>, String> {
    let directory = settings.ensure_directory_exists()?;
    let entries = fs::read_dir(&directory).map_err(|err| {
        format!(
            "Failed to read save directory {}: {}",
            directory.display(),
            err
        )
    })?;

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

pub fn delete_save_file(settings: &SaveSettings, key: &str) -> Result<(), String> {
    let full_path = settings.resolve_file(key)?;
    if full_path.exists() {
        fs::remove_file(&full_path).map_err(|err| {
            format!(
                "Failed to remove save file {}: {}",
                full_path.display(),
                err
            )
        })?;
    }
    Ok(())
}

pub fn load_or_default<T>(settings: &SaveSettings, key: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Default + Serialize,
{
    load(settings, key)
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
        Self::new(Duration::from_secs(60))
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
    pub fn new(auto_save_interval: Duration) -> Self {
        Self {
            state: GameState::default(),
            events: Vec::new(),
            auto_save_timer: Timer::from_seconds(
                auto_save_interval.as_secs_f32(),
                TimerMode::Repeating,
            ),
            snapshots: Vec::new(),
            max_snapshots: 10,
        }
    }

    pub fn from_settings(settings: &SaveSettings) -> Self {
        let mut tracker = Self::new(settings.autosave_interval());
        tracker.max_snapshots = settings.snapshot_limit();
        tracker
    }

    pub fn apply_settings(&mut self, settings: &SaveSettings) {
        self.max_snapshots = settings.snapshot_limit();
        self.auto_save_timer.set_duration(Duration::from_secs_f32(
            settings.autosave_interval().as_secs_f32(),
        ));
        self.auto_save_timer.reset();
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
    pub fn auto_save(&self, world: &mut World, game_time: f64) -> Result<PathBuf, String> {
        let settings = world
            .get_resource::<SaveSettings>()
            .cloned()
            .unwrap_or_default();

        let default_file = settings.default_file().to_string();
        save_game_data(
            world,
            self,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            game_time,
            &settings,
            &default_file,
        )
    }

    /// Update the auto-save timer and return true if it's time to save
    pub fn should_auto_save(&mut self, delta_time: f32) -> bool {
        self.auto_save_timer
            .tick(Duration::from_secs_f32(delta_time));
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
    settings: &SaveSettings,
    key: &str,
) -> Result<PathBuf, String> {
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
                let Some(component_info) = world.components().get_info(*component_id) else {
                    continue;
                };

                let Some(type_id) = component_info.type_id() else {
                    continue;
                };

                let Some(type_registration) = type_registry.get(type_id) else {
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
                    let serializer = ReflectSerializer::new(reflected, &type_registry);
                    if let Ok(value) = serde_json::to_value(&serializer) {
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

    save(&save_data, settings, key)
}

pub fn load_game_data(settings: &SaveSettings, key: &str) -> Result<GameSaveData, String> {
    load::<GameSaveData>(settings, key)
}

/// Extension trait for World to add bevy_save-style convenience methods
pub trait WorldSaveExt {
    fn save_game(&mut self, path: &str) -> Result<(), String>;
    fn load_game(&mut self, path: &str) -> Result<(), String>;
    fn checkpoint_game(&mut self, name: &str) -> Result<(), String>;
    fn rollback_game_checkpoint(&mut self, name: Option<&str>) -> Result<(), String>;
}

impl WorldSaveExt for World {
    fn save_game(&mut self, path: &str) -> Result<(), String> {
        let settings = self
            .get_resource::<SaveSettings>()
            .cloned()
            .unwrap_or_default();

        let key = if path.is_empty() {
            settings.default_file().to_string()
        } else {
            path.to_string()
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let game_time = self
            .get_resource::<Time>()
            .map(|time| time.elapsed_secs_f64())
            .unwrap_or(0.0);

        // Clone tracker to avoid borrow conflicts
        let tracker = self
            .get_resource::<GameTracker>()
            .cloned()
            .ok_or_else(|| "GameTracker resource not found".to_string())?;

        save_game_data(self, &tracker, timestamp, game_time, &settings, &key)?;
        Ok(())
    }

    fn load_game(&mut self, path: &str) -> Result<(), String> {
        let settings = self
            .get_resource::<SaveSettings>()
            .cloned()
            .unwrap_or_default();

        let key = if path.is_empty() {
            settings.default_file().to_string()
        } else {
            path.to_string()
        };

        let save_data = load_game_data(&settings, &key)?;

        // Update GameTracker if it exists
        if let Some(mut tracker) = self.get_resource_mut::<GameTracker>() {
            tracker.state = save_data.game_state;
            tracker.events = save_data.events;
        }

        Ok(())
    }

    fn checkpoint_game(&mut self, name: &str) -> Result<(), String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let snapshot_name = if name.is_empty() {
            format!("checkpoint-{:.3}", timestamp)
        } else {
            name.to_string()
        };

        let mut tracker = self
            .get_resource_mut::<GameTracker>()
            .ok_or_else(|| "GameTracker resource not found".to_string())?;
        tracker.snapshot(snapshot_name, timestamp);
        Ok(())
    }

    fn rollback_game_checkpoint(&mut self, name: Option<&str>) -> Result<(), String> {
        let mut tracker = self
            .get_resource_mut::<GameTracker>()
            .ok_or_else(|| "GameTracker resource not found".to_string())?;

        match name {
            Some(snapshot_name) => tracker.rollback_to(snapshot_name).map(|_| ()),
            None => tracker.rollback().map(|_| ()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
    struct TestSave {
        value: i32,
    }

    #[test]
    fn save_replaces_file_atomically() {
        let unique = format!(
            "lp_save_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let directory = std::env::temp_dir().join(unique);
        let settings = SaveSettings::default().with_custom_directory(&directory);

        let first = TestSave { value: 10 };
        let second = TestSave { value: 42 };

        let path = save(&first, &settings, "slot.json").expect("first save should succeed");
        save(&second, &settings, "slot.json").expect("second save should succeed");

        let loaded: TestSave = load(&settings, "slot.json").expect("load should succeed");
        assert_eq!(loaded, second);
        assert!(path.exists());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn tracker_snapshot_limit_and_rollback_work() {
        let mut tracker = GameTracker::new(Duration::from_secs(5));
        tracker.max_snapshots = 2;

        tracker.state.total_energy = 1.0;
        tracker.snapshot("s1".to_string(), 1.0);

        tracker.state.total_energy = 2.0;
        tracker.snapshot("s2".to_string(), 2.0);

        tracker.state.total_energy = 3.0;
        tracker.snapshot("s3".to_string(), 3.0);

        assert_eq!(tracker.snapshots.len(), 2);
        assert_eq!(tracker.snapshots[0].name, "s2");
        assert_eq!(tracker.snapshots[1].name, "s3");

        tracker.state.total_energy = 99.0;
        tracker
            .rollback_to("s2")
            .expect("rollback to known snapshot should succeed");
        assert_eq!(tracker.state.total_energy, 2.0);
    }
}
