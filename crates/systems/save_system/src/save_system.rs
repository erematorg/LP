use crate::versioning::{is_save_up_to_date, upgrade_save};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

/// Saves data to a file
pub fn save<T: Serialize>(data: &T, path: &str) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Serialization failed: {}", e))?;
    fs::write(path, json).map_err(|e| format!("File write failed: {}", e))?;
    Ok(())
}

/// Loads data from a file, or returns a properly initialized default save if missing
pub fn load<T: for<'de> Deserialize<'de> + Default + Serialize>(path: &str) -> Result<T, String> {
    let json = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("[Warning] Save file not found. Creating a new one.");
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
