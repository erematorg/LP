use serde::{Serialize, Deserialize};
use std::fs;
use serde_json::Value;
use crate::versioning::{is_save_up_to_date, SAVE_VERSION};

/// Saves data to a file
pub fn save<T: Serialize>(data: &T, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("File write failed: {}", e))?;
    Ok(())
}

/// Loads data from a file, or returns a default value if the file is missing
pub fn load<T: for<'de> Deserialize<'de> + Default>(path: &str) -> Result<T, String> {
    let json = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("[Warning] Save file not found. Creating a new one.");
            return Ok(T::default());
        }
    };

    let data: Value = serde_json::from_str(&json)
        .map_err(|e| format!("Deserialization failed: {}", e))?;

    if !is_save_up_to_date(&data) {
        return Err(format!("[Error] Save file is outdated! Expected version: {}", SAVE_VERSION));
    }

    serde_json::from_value(data).map_err(|e| format!("Final deserialization failed: {}", e))
}
